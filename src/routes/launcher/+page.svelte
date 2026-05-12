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
    enabled: boolean;
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
  $: updateLabel = !isInstalled ? "Install patcher" : needsPatcherUpdate ? "Update patcher" : "";

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

  async function toggleInstalledMod(mod: InstalledMod) {
    busy = true;
    error = "";
    message = "";
    try {
      await invoke("set_mod_enabled", { input: { path: mod.path, enabled: !mod.enabled } });
      await load();
      message = `${mod.name} ${mod.enabled ? "disabled" : "enabled"}.`;
    } catch (err) {
      error = errorMessage(err, "Could not update mod state");
    } finally {
      busy = false;
    }
  }

  function errorMessage(err: unknown, fallback: string) {
    return err instanceof Error ? err.message : typeof err === "string" ? err : fallback;
  }

  function modInitials(name: string) {
    return name
      .replace(/\.(zip|disabled)$/i, "")
      .split(/[\s._-]+/)
      .filter(Boolean)
      .slice(0, 2)
      .map((part) => part[0])
      .join("")
      .toUpperCase() || "MOD";
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
      <p class="mt-2 max-w-2xl text-sm leading-6 text-muted-foreground">Installed patcher: <span class="font-bold text-foreground">{currentRelease}</span> · Latest: <span class="font-bold text-foreground">{latestReleaseLabel}</span></p>
    </div>
    <div class="flex flex-wrap gap-2">
      {#if updateLabel}
        <Button size="lg" disabled={!isDesktop || busy || !hasGameFolder || !latestRelease} on:click={installModloader}>{updateLabel}</Button>
      {/if}
      <Button size="lg" disabled={!isDesktop || loading || busy || !canLaunch} on:click={launchGame}>
        {busy ? "Working..." : "Launch game"}
      </Button>
    </div>
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

    <Card class="p-3">
      <div class="grid grid-cols-2 overflow-hidden rounded-lg border border-white/10 bg-background/45">
        <button class="h-11 font-black {activePanel === 'mods' ? 'bg-primary/20 text-primary' : 'text-muted-foreground hover:bg-white/8'}" type="button" on:click={() => (activePanel = "mods")}>Mods</button>
        <button class="h-11 font-black {activePanel === 'settings' ? 'bg-primary/20 text-primary' : 'text-muted-foreground hover:bg-white/8'}" type="button" on:click={() => (activePanel = "settings")}>Settings</button>
      </div>

      {#if activePanel === "mods"}
        <div class="p-2 pt-5">
          <div class="mb-5 flex items-center justify-between gap-3">
            <div>
              <h2 class="text-xl font-black">Installed mods</h2>
              <p class="mt-1 text-sm text-muted-foreground">{installedMods.length} found in your mods folder.</p>
            </div>
            <Button variant="outline" size="sm" on:click={load}>Refresh</Button>
          </div>

          {#if !hasGameFolder}
            <p class="rounded-lg border border-white/12 bg-background/45 p-4 text-sm text-muted-foreground">Select a game folder in Settings to scan installed mods.</p>
          {:else if installedMods.length}
            <section class="grid gap-5 md:grid-cols-2 xl:grid-cols-3">
              {#each installedMods as mod}
                <article class="group overflow-hidden rounded-lg border border-white/10 bg-card/92 shadow-[0_18px_55px_rgba(0,0,0,0.34)] backdrop-blur-md transition duration-200 hover:-translate-y-0.5 hover:border-white/30">
                  <div class="relative aspect-[16/11] overflow-hidden bg-muted">
                    <div class="absolute inset-0 bg-[linear-gradient(135deg,hsl(var(--primary)/.22),hsl(var(--accent)/.18))]"></div>
                    <div class="absolute left-5 top-5 rounded-md border border-white/30 bg-white/12 px-4 py-3 text-4xl font-black text-white shadow-xl backdrop-blur">
                      {modInitials(mod.name)}
                    </div>
                    <div class="absolute inset-x-0 bottom-0 h-28 bg-gradient-to-t from-background/88 to-transparent"></div>
                    <div class="absolute left-3 top-3 z-20 flex flex-wrap gap-2">
                      <span class="rounded-full border border-white/25 bg-black/45 px-2.5 py-1 text-[0.68rem] font-black uppercase tracking-wide text-white backdrop-blur">Installed</span>
                      <span class="rounded-full border border-white/25 bg-black/45 px-2.5 py-1 text-[0.68rem] font-black uppercase tracking-wide text-white backdrop-blur">{mod.kind}</span>
                      <span class="rounded-full border border-white/25 bg-black/45 px-2.5 py-1 text-[0.68rem] font-black uppercase tracking-wide text-white backdrop-blur">{mod.enabled ? "Enabled" : "Disabled"}</span>
                    </div>
                  </div>

                  <div class="grid gap-4 p-4">
                    <div class="min-w-0">
                      <p class="truncate text-xs font-black uppercase tracking-[0.18em] text-muted-foreground">Local mod / {mod.kind}</p>
                      <h2 class="line-clamp-2 text-2xl font-black leading-tight text-foreground">{mod.name}</h2>
                      <p class="mt-1 break-words text-xs font-bold text-muted-foreground">{mod.path}</p>
                    </div>

                    <div class="grid grid-cols-2 gap-2">
                      <Button variant="outline" on:click={load}>Refresh</Button>
                      <Button variant={mod.enabled ? "destructive" : "default"} disabled={busy} on:click={() => toggleInstalledMod(mod)}>
                        {mod.enabled ? "Disable" : "Enable"}
                      </Button>
                    </div>
                  </div>
                </article>
              {/each}
            </section>
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
  {/if}
</main>
