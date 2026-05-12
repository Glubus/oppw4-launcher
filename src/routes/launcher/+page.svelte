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
  };

  const defaultConfig: LauncherConfig = {
    launchMode: "steam",
    gameFolder: null,
    gameExecutablePath: null,
    modloaderRepo: "Glubus/oppw4-modloader",
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

  $: hasGameFolder = Boolean(config.gameFolder);
  $: canLaunch = config.launchMode === "steam" || Boolean(config.gameExecutablePath);
  $: isInstalled = config.installedFiles.length > 0;

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
      message = "Modloader installed.";
    } catch (err) {
      error = errorMessage(err, "Could not install modloader");
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
      message = "Modloader restored.";
    } catch (err) {
      error = errorMessage(err, "Could not restore modloader");
    } finally {
      busy = false;
    }
  }

  function errorMessage(err: unknown, fallback: string) {
    return err instanceof Error ? err.message : typeof err === "string" ? err : fallback;
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
      <h1 class="mt-1 text-4xl font-black tracking-tight">Launch, patch, restore</h1>
      <p class="mt-2 max-w-2xl text-sm leading-6 text-muted-foreground">Detect OPPW4, install the dinput8 modloader, restore backups, and launch through Steam or a direct executable.</p>
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
        <p class="text-xs font-black uppercase tracking-[0.18em] text-muted-foreground">Launch mode</p>
        <p class="mt-3 text-2xl font-black">{config.launchMode === "steam" ? "Steam URI" : "Executable"}</p>
        <p class="mt-2 break-words text-sm leading-6 text-muted-foreground">{config.launchMode === "steam" ? "Uses steam://run/1089090." : config.gameExecutablePath || "No executable selected."}</p>
      </Card>
      <Card class="p-5">
        <p class="text-xs font-black uppercase tracking-[0.18em] text-muted-foreground">Game folder</p>
        <p class="mt-3 text-2xl font-black">{hasGameFolder ? "Configured" : "Missing"}</p>
        <p class="mt-2 break-words text-sm leading-6 text-muted-foreground">{config.gameFolder || "Detect Steam or select the folder manually."}</p>
      </Card>
      <Card class="p-5">
        <p class="text-xs font-black uppercase tracking-[0.18em] text-muted-foreground">Modloader</p>
        <p class="mt-3 text-2xl font-black">{modloaderStatus}</p>
        <p class="mt-2 break-words text-sm leading-6 text-muted-foreground">{config.modloaderRelease ? `Release ${config.modloaderRelease}` : "Install from a GitHub Release zip once the game folder is set."}</p>
      </Card>
    </section>

    <section class="grid items-start gap-5 lg:grid-cols-[minmax(0,1fr)_380px]">
      <Card class="p-5">
        <div class="mb-5">
          <h2 class="text-xl font-black">Game detection</h2>
          <p class="mt-2 text-sm leading-6 text-muted-foreground">Steam installs are detected automatically when possible. Non-Steam installs can use a direct executable path.</p>
        </div>

        <div class="grid gap-4">
          {#if detectedGame}
            <div class="flex flex-col gap-3 rounded-lg border border-primary/20 bg-primary/10 p-4 sm:flex-row sm:items-center sm:justify-between">
              <div>
                <p class="font-black">Steam install detected</p>
                <p class="mt-1 break-words text-sm text-muted-foreground">{detectedGame.gameFolder}</p>
              </div>
              <Button variant="outline" on:click={useDetectedGame}>Use this install</Button>
            </div>
          {:else}
            <div class="flex flex-col gap-3 rounded-lg border border-white/12 bg-background/45 p-4 sm:flex-row sm:items-center sm:justify-between">
              <div>
                <p class="font-black">No Steam install detected</p>
                <p class="mt-1 text-sm text-muted-foreground">Select the executable or game folder manually.</p>
              </div>
              <Button variant="outline" on:click={load}>Scan again</Button>
            </div>
          {/if}

          <div class="grid grid-cols-2 overflow-hidden rounded-lg border border-white/10 bg-background/45">
            <button class="h-11 font-black {config.launchMode === 'steam' ? 'bg-primary/20 text-primary' : 'text-muted-foreground hover:bg-white/8'}" type="button" on:click={() => setLaunchMode("steam")}>Steam</button>
            <button class="h-11 font-black {config.launchMode === 'executable' ? 'bg-primary/20 text-primary' : 'text-muted-foreground hover:bg-white/8'}" type="button" on:click={() => setLaunchMode("executable")}>Executable</button>
          </div>

          <div class="flex flex-wrap gap-2">
            <Button variant="outline" on:click={chooseGameFolder}>Select game folder</Button>
            <Button variant="outline" on:click={chooseExecutable}>Select executable</Button>
          </div>
        </div>
      </Card>

      <Card class="p-5">
        <div class="mb-5">
          <h2 class="text-xl font-black">dinput8 modloader</h2>
          <p class="mt-2 text-sm leading-6 text-muted-foreground">Downloads the latest release zip, backs up replaced files, then writes the modloader into the selected game folder.</p>
        </div>

        <div class="grid gap-4">
          <Label>
            GitHub release repository
            <Input bind:value={config.modloaderRepo} on:change={() => saveAndRefresh("Repository saved.")} placeholder="owner/repository" />
          </Label>
          <Button disabled={busy || !hasGameFolder} on:click={installModloader}>{isInstalled ? "Update modloader" : "Install modloader"}</Button>
          <Button variant="destructive" disabled={busy || !isInstalled} on:click={restoreModloader}>Restore original files</Button>
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
