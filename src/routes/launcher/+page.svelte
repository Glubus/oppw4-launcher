<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import AppHeader from "$lib/components/organisms/AppHeader.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import Label from "$lib/components/ui/Label.svelte";

  type LaunchMode = "steam" | "executable";

  type InstalledFile = {
    relativePath: string;
    backupPath?: string | null;
  };

  type LauncherConfig = {
    launchMode: LaunchMode;
    gameFolder?: string | null;
    gameExecutablePath?: string | null;
    modloaderRepo: string;
    modloaderRelease?: string | null;
    installedFiles: InstalledFile[];
    lastLaunchAt?: string | null;
  };

  type DetectedGame = {
    gameFolder: string;
    executablePath?: string | null;
    source: string;
  };

  type LauncherState = {
    config: LauncherConfig;
    detectedGame?: DetectedGame | null;
    modloaderStatus: string;
    latestRelease?: ReleaseInfo | null;
    needsPatcherUpdate: boolean;
    installedMods: InstalledMod[];
  };

  type ReleaseInfo = {
    tagName: string;
    name?: string | null;
    body?: string | null;
    htmlUrl: string;
    prerelease: boolean;
    assetName?: string | null;
  };

  type InstalledMod = {
    name: string;
    kind: string;
    path: string;
  };

  const defaultConfig: LauncherConfig = {
    launchMode: "steam",
    gameFolder: null,
    gameExecutablePath: null,
    modloaderRepo: "Glubus/oppw4-patcher",
    modloaderRelease: null,
    installedFiles: [],
    lastLaunchAt: null
  };

  let config = defaultConfig;
  let detectedGame: DetectedGame | null = null;
  let modloaderStatus = "Unknown";
  let loading = true;
  let busy = false;
  let error = "";
  let message = "";
  let isDesktop = false;
  let installedMods: InstalledMod[] = [];
  let latestRelease: ReleaseInfo | null = null;
  let needsPatcherUpdate = false;
  let activePanel: "mods" | "settings" = "mods";

  $: hasGameFolder = Boolean(config.gameFolder);
  $: canLaunch = config.launchMode === "steam" || Boolean(config.gameExecutablePath);
  $: isInstalled = config.installedFiles.length > 0;
  $: currentRelease = config.modloaderRelease || "Not installed";
  $: latestReleaseLabel = latestRelease?.tagName || "Unknown";
  $: patcherTone = !isInstalled ? "Missing" : needsPatcherUpdate ? "Update available" : "Up to date";

  onMount(async () => {
    isDesktop = "__TAURI_INTERNALS__" in window;
    if (!isDesktop) {
      loading = false;
      return;
    }
    await load();
  });

  async function load() {
    loading = true;
    error = "";
    try {
      const state = await invoke<LauncherState>("get_launcher_state");
      config = state.config;
      detectedGame = state.detectedGame ?? null;
      modloaderStatus = state.modloaderStatus;
      installedMods = state.installedMods ?? [];
      latestRelease = state.latestRelease ?? null;
      needsPatcherUpdate = state.needsPatcherUpdate;
    } catch (err) {
      error = errorMessage(err, "Could not load launcher state");
    } finally {
      loading = false;
    }
  }

  async function save() {
    config = await invoke<LauncherConfig>("save_launcher_config", { config });
  }

  async function saveAndRefresh(success: string) {
    error = "";
    message = "";
    try {
      await save();
      await load();
      message = success;
    } catch (err) {
      error = errorMessage(err, "Could not save config");
    }
  }

  async function chooseGameFolder() {
    const selected = await open({ directory: true, multiple: false, title: "Select OPPW4 game folder" });
    if (typeof selected === "string") {
      config = { ...config, gameFolder: selected };
      await saveAndRefresh("Game folder saved.");
    }
  }

  async function chooseExecutable() {
    const selected = await open({
      directory: false,
      multiple: false,
      title: "Select OPPW4 executable",
      filters: [{ name: "Executable", extensions: ["exe", "AppImage", "sh", "x86_64"] }]
    });
    if (typeof selected === "string") {
      const parent = selected.replace(/[\\/][^\\/]+$/, "");
      config = { ...config, gameExecutablePath: selected, gameFolder: config.gameFolder ?? parent };
      await saveAndRefresh("Executable saved.");
    }
  }

  async function useDetectedGame() {
    if (!detectedGame) return;
    config = {
      ...config,
      gameFolder: detectedGame.gameFolder,
      gameExecutablePath: detectedGame.executablePath ?? config.gameExecutablePath
    };
    await saveAndRefresh("Detected Steam install saved.");
  }

  async function setLaunchMode(mode: LaunchMode) {
    config = { ...config, launchMode: mode };
    await saveAndRefresh(`Launch mode set to ${mode}.`);
  }

  async function launchGame() {
    busy = true;
    error = "";
    message = "";
    try {
      await save();
      await invoke("launch_game");
      await load();
      message = "Launch request sent.";
    } catch (err) {
      error = errorMessage(err, "Could not launch game");
    } finally {
      busy = false;
    }
  }

  async function installModloader() {
    busy = true;
    error = "";
    message = "";
    try {
      await save();
      config = await invoke<LauncherConfig>("install_modloader");
      await load();
      message = "Patcher installed.";
    } catch (err) {
      error = errorMessage(err, "Could not install patcher");
    } finally {
      busy = false;
    }
  }

  async function restoreModloader() {
    busy = true;
    error = "";
    message = "";
    try {
      config = await invoke<LauncherConfig>("restore_modloader");
      await load();
      message = "Patcher restored.";
    } catch (err) {
      error = errorMessage(err, "Could not restore patcher");
    } finally {
      busy = false;
    }
  }

  function errorMessage(err: unknown, fallback: string) {
    return err instanceof Error ? err.message : typeof err === "string" ? err : fallback;
  }

  function releaseBodyPreview(value?: string | null) {
    const clean = (value ?? "").trim();
    if (!clean) return "No patch notes were provided for this release.";
    return clean.length > 900 ? `${clean.slice(0, 900)}...` : clean;
  }
</script>

<svelte:head>
  <title>Launcher | OPPW4 Skin Hub</title>
</svelte:head>

<AppHeader />

<main class="mx-auto grid max-w-7xl gap-5 px-4 py-6">
  <section class="flex flex-col gap-4 rounded-lg border border-white/10 bg-card/86 p-5 shadow-[0_18px_60px_rgba(0,0,0,0.25)] backdrop-blur-md md:flex-row md:items-end md:justify-between">
    <div>
      <p class="text-xs font-black uppercase tracking-[0.22em] text-primary/90">Desktop launcher</p>
      <h1 class="mt-1 text-4xl font-black tracking-tight">Launch and manage mods</h1>
      <p class="mt-2 max-w-2xl text-sm leading-6 text-muted-foreground">The launcher checks your game folder, keeps the patcher readable, lists installed mods, and starts OPPW4 without making you touch files manually.</p>
    </div>
    <Button size="lg" disabled={!isDesktop || loading || busy || !canLaunch} on:click={launchGame}>
      {busy ? "Working..." : "Launch game"}
    </Button>
  </section>

  {#if !isDesktop}
    <Card class="p-6">
      <h2 class="text-xl font-black">Desktop-only tools</h2>
      <p class="mt-2 text-sm leading-6 text-muted-foreground">This page is part of the launcher build. Native game detection, file install, and launch actions are available inside the Tauri app.</p>
    </Card>
  {:else}
    {#if error}
      <div class="rounded-xl border border-destructive/40 bg-destructive/15 px-4 py-3 text-sm text-red-100 shadow-lg">{error}</div>
    {/if}
    {#if message}
      <div class="rounded-xl border border-emerald-400/30 bg-emerald-500/15 px-4 py-3 text-sm font-bold text-emerald-200 shadow-lg">{message}</div>
    {/if}

    <section class="grid gap-5 md:grid-cols-3">
      <Card class="p-5">
        <p class="text-xs font-black uppercase tracking-[0.18em] text-muted-foreground">Patcher status</p>
        <p class="mt-3 text-2xl font-black">{patcherTone}</p>
        <p class="mt-2 text-sm leading-6 text-muted-foreground">Installed: <span class="font-bold text-foreground">{currentRelease}</span><br />Latest: <span class="font-bold text-foreground">{latestReleaseLabel}</span></p>
      </Card>
      <Card class="p-5">
        <p class="text-xs font-black uppercase tracking-[0.18em] text-muted-foreground">Game folder</p>
        <p class="mt-3 text-2xl font-black">{hasGameFolder ? "Configured" : "Missing"}</p>
        <p class="mt-2 break-words text-sm leading-6 text-muted-foreground">{config.gameFolder || "Detect Steam or select the folder manually."}</p>
      </Card>
      <Card class="p-5">
        <p class="text-xs font-black uppercase tracking-[0.18em] text-muted-foreground">Installed mods</p>
        <p class="mt-3 text-2xl font-black">{installedMods.length}</p>
        <p class="mt-2 text-sm leading-6 text-muted-foreground">Folders and zip files found in your selected <span class="font-bold">mods/</span> folder.</p>
      </Card>
    </section>

    <section class="grid items-start gap-5 lg:grid-cols-[minmax(0,1fr)_390px]">
      <div class="grid gap-5">
        <Card class="p-5">
          <div class="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
            <div>
              <h2 class="text-xl font-black">Patcher</h2>
              <p class="mt-2 text-sm leading-6 text-muted-foreground">The patcher is the dinput8 DLL loaded when the game starts. Keep it updated, then launch the game normally.</p>
            </div>
            <div class="flex flex-wrap gap-2">
              <Button disabled={busy || !hasGameFolder || !latestRelease} on:click={installModloader}>{isInstalled ? "Update patcher" : "Install patcher"}</Button>
              <Button variant="outline" disabled={busy} on:click={load}>Check updates</Button>
            </div>
          </div>

          <div class="mt-5 grid gap-3 md:grid-cols-2">
            <div class="rounded-lg border border-white/10 bg-background/45 p-4">
              <p class="text-xs font-black uppercase tracking-[0.16em] text-muted-foreground">Current</p>
              <p class="mt-2 text-lg font-black">{currentRelease}</p>
              <p class="mt-1 text-sm text-muted-foreground">{modloaderStatus}</p>
            </div>
            <div class="rounded-lg border border-white/10 bg-background/45 p-4">
              <p class="text-xs font-black uppercase tracking-[0.16em] text-muted-foreground">Latest GitHub</p>
              <p class="mt-2 text-lg font-black">{latestReleaseLabel}</p>
              <p class="mt-1 text-sm text-muted-foreground">{latestRelease?.assetName || "No compatible asset found"}</p>
            </div>
          </div>

          <div class="mt-5 rounded-lg border border-white/10 bg-background/45 p-4">
            <div class="flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
              <div>
                <p class="text-sm font-black">Patch notes</p>
                <p class="mt-1 text-xs text-muted-foreground">{latestRelease?.name || latestRelease?.tagName || "No release loaded"}</p>
              </div>
              {#if latestRelease?.htmlUrl}
                <a class="text-sm font-bold text-primary hover:underline" href={latestRelease.htmlUrl} target="_blank" rel="noreferrer">Open release</a>
              {/if}
            </div>
            <pre class="mt-3 max-h-56 overflow-auto whitespace-pre-wrap rounded-md bg-black/25 p-3 text-xs leading-5 text-muted-foreground">{releaseBodyPreview(latestRelease?.body)}</pre>
          </div>
        </Card>

        <Card class="p-3">
          <div class="grid grid-cols-2 overflow-hidden rounded-lg border border-white/10 bg-background/45">
            <button class="h-11 font-black {activePanel === 'mods' ? 'bg-primary/20 text-primary' : 'text-muted-foreground hover:bg-white/8'}" type="button" on:click={() => (activePanel = "mods")}>Installed mods</button>
            <button class="h-11 font-black {activePanel === 'settings' ? 'bg-primary/20 text-primary' : 'text-muted-foreground hover:bg-white/8'}" type="button" on:click={() => (activePanel = "settings")}>Settings</button>
          </div>

          {#if activePanel === "mods"}
            <div class="p-2 pt-5">
              <div class="mb-5 flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
                <div>
                  <h2 class="text-xl font-black">Installed mods</h2>
                  <p class="mt-2 text-sm leading-6 text-muted-foreground">Everything here is already in <span class="font-bold">mods/</span>. Put folders or zip files there and refresh.</p>
                </div>
                <Button variant="outline" size="sm" on:click={load}>Refresh</Button>
              </div>

              {#if !hasGameFolder}
                <p class="rounded-lg border border-white/12 bg-background/45 p-4 text-sm text-muted-foreground">Select a game folder in Settings to scan installed mods.</p>
              {:else if installedMods.length}
                <div class="grid gap-2">
                  {#each installedMods as mod}
                    <div class="grid gap-1 rounded-md border border-white/10 bg-background/45 px-3 py-2">
                      <div class="flex items-center justify-between gap-3">
                        <p class="min-w-0 truncate text-sm font-black">{mod.name}</p>
                        <span class="shrink-0 rounded-md border border-white/12 px-2 py-1 text-[11px] font-black uppercase text-muted-foreground">{mod.kind}</span>
                      </div>
                      <p class="break-words text-xs text-muted-foreground">{mod.path}</p>
                    </div>
                  {/each}
                </div>
              {:else}
                <p class="rounded-lg border border-white/12 bg-background/45 p-4 text-sm text-muted-foreground">No installed mods found. Create a <span class="font-bold">mods/</span> folder next to the game executable and add mod folders or zip files.</p>
              {/if}
            </div>
          {:else}
            <div class="grid gap-5 p-2 pt-5">
              <div>
                <h2 class="text-xl font-black">Settings</h2>
                <p class="mt-2 text-sm leading-6 text-muted-foreground">Only change this if the launcher picked the wrong game folder or if you are testing another patcher repo.</p>
              </div>

              {#if detectedGame}
                <div class="flex flex-col gap-3 rounded-lg border border-primary/20 bg-primary/10 p-4 sm:flex-row sm:items-center sm:justify-between">
                  <div>
                    <p class="font-black">Steam install detected</p>
                    <p class="mt-1 break-words text-sm text-muted-foreground">{detectedGame.gameFolder}</p>
                  </div>
                  <Button variant="outline" on:click={useDetectedGame}>Use this install</Button>
                </div>
              {/if}

              <div class="grid gap-4 rounded-lg border border-white/10 bg-background/45 p-4">
                <p class="font-black">Launch method</p>
                <div class="grid grid-cols-2 overflow-hidden rounded-lg border border-white/10 bg-background/45">
                  <button class="h-11 font-black {config.launchMode === 'steam' ? 'bg-primary/20 text-primary' : 'text-muted-foreground hover:bg-white/8'}" type="button" on:click={() => setLaunchMode("steam")}>Steam</button>
                  <button class="h-11 font-black {config.launchMode === 'executable' ? 'bg-primary/20 text-primary' : 'text-muted-foreground hover:bg-white/8'}" type="button" on:click={() => setLaunchMode("executable")}>Executable</button>
                </div>
                <div class="flex flex-wrap gap-2">
                  <Button variant="outline" on:click={chooseGameFolder}>Select game folder</Button>
                  <Button variant="outline" on:click={chooseExecutable}>Select executable</Button>
                </div>
              </div>

              <Label>
                Patcher GitHub repository
                <Input bind:value={config.modloaderRepo} on:change={() => saveAndRefresh("Repository saved.")} placeholder="owner/repository" />
              </Label>
            </div>
          {/if}
        </Card>
      </div>

      <Card class="p-5">
        <div class="mb-5">
          <h2 class="text-xl font-black">What this does</h2>
          <p class="mt-2 text-sm leading-6 text-muted-foreground">For normal users: install or update the patcher, keep mods inside <span class="font-bold">mods/</span>, then launch the game.</p>
        </div>

        <div class="grid gap-4">
          <Button disabled={busy || !hasGameFolder || !latestRelease} on:click={installModloader}>{isInstalled ? "Update patcher" : "Install patcher"}</Button>
          <Button variant="destructive" disabled={busy || !isInstalled} on:click={restoreModloader}>Restore original files</Button>
          <Button variant="outline" disabled={!canLaunch || busy} on:click={launchGame}>Launch game</Button>
        </div>

        {#if config.installedFiles.length}
          <div class="mt-5 border-t border-white/10 pt-4">
            <p class="text-xs font-black uppercase tracking-[0.18em] text-muted-foreground">Installed files</p>
            <div class="mt-3 grid gap-2">
              {#each config.installedFiles as file}
                <p class="break-words rounded-md bg-background/45 px-3 py-2 text-xs font-bold text-muted-foreground">{file.relativePath}</p>
              {/each}
            </div>
          </div>
        {/if}
      </Card>
    </section>
  {/if}
</main>
