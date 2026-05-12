<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";

  type LaunchMode = "steam" | "executable";

  type InstalledFile = {
    relativePath: string;
    backupPath?: string | null;
  };

  type LauncherConfig = {
    launchMode: LaunchMode;
    siteUrl: string;
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

  let config: LauncherConfig = {
    launchMode: "steam",
    siteUrl: "https://oppw4.prism.am",
    gameFolder: null,
    gameExecutablePath: null,
    modloaderRepo: "Glubus/oppw4-modloader",
    modloaderRelease: null,
    installedFiles: [],
    lastLaunchAt: null
  };
  let detectedGame: DetectedGame | null = null;
  let modloaderStatus = "Unknown";
  let loading = true;
  let busy = false;
  let error = "";
  let message = "";
  let activeTab: "mods" | "launcher" = "mods";
  let browserKey = 0;

  $: hasGameFolder = Boolean(config.gameFolder);
  $: canLaunch = config.launchMode === "steam" || Boolean(config.gameExecutablePath);
  $: isInstalled = config.installedFiles.length > 0;
  $: normalizedSiteUrl = normalizeSiteUrl(config.siteUrl);

  load();

  async function load() {
    loading = true;
    error = "";
    try {
      const state = await invoke<LauncherState>("get_launcher_state");
      applyState(state);
    } catch (err) {
      error = errorMessage(err, "Could not load launcher state");
    } finally {
      loading = false;
    }
  }

  function applyState(state: LauncherState) {
    config = state.config;
    detectedGame = state.detectedGame ?? null;
    modloaderStatus = state.modloaderStatus;
  }

  async function save() {
    const saved = await invoke<LauncherConfig>("save_launcher_config", { config });
    config = saved;
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

  function normalizeSiteUrl(value: string) {
    const trimmed = value.trim();
    if (!trimmed) return "https://oppw4.prism.am";
    if (trimmed.startsWith("http://") || trimmed.startsWith("https://")) return trimmed;
    return `https://${trimmed}`;
  }
</script>

<svelte:head>
  <title>OPPW4 Launcher</title>
</svelte:head>

<main class="app-shell">
  <header class="topbar">
    <div>
      <p class="eyebrow">OPPW4 Mod Launcher</p>
      <h1>OPPW4 Hub</h1>
    </div>
    <nav class="tabs" aria-label="Launcher sections">
      <button class:active={activeTab === "mods"} type="button" on:click={() => (activeTab = "mods")}>Mods</button>
      <button class:active={activeTab === "launcher"} type="button" on:click={() => (activeTab = "launcher")}>Launcher</button>
    </nav>
    <button class="primary compact" disabled={loading || busy || !canLaunch} on:click={launchGame}>
      {busy ? "Working..." : "Launch game"}
    </button>
  </header>

  {#if error}
    <div class="alert error">{error}</div>
  {/if}
  {#if message}
    <div class="alert success">{message}</div>
  {/if}

  {#if activeTab === "mods"}
    <section class="browser-panel">
      <div class="browser-toolbar">
        <div>
          <span class="label">Mods browser</span>
          <p>{normalizedSiteUrl}</p>
        </div>
        <div class="actions">
          <button class="secondary" type="button" on:click={() => (browserKey += 1)}>Reload</button>
          <button class="secondary" type="button" on:click={() => (activeTab = "launcher")}>Launcher settings</button>
        </div>
      </div>
      {#key browserKey}
        <iframe class="site-frame" title="OPPW4 Mods Browser" src={normalizedSiteUrl}></iframe>
      {/key}
    </section>
  {:else}
    <section class="hero">
      <div>
        <p class="eyebrow">Native tools</p>
        <h2>Launch, patch, restore.</h2>
        <p class="subtitle">Manage the dinput8 modloader and start ONE PIECE: PIRATE WARRIORS 4 from Steam or a direct executable.</p>
      </div>
      <button class="primary" disabled={loading || busy || !canLaunch} on:click={launchGame}>
        {busy ? "Working..." : "Launch game"}
      </button>
    </section>

    <section class="status-grid">
      <article class="status-card">
        <span class="label">Launch mode</span>
        <strong>{config.launchMode === "steam" ? "Steam URI" : "Executable"}</strong>
        <p>{config.launchMode === "steam" ? "Uses steam://run/1089090." : config.gameExecutablePath || "No executable selected."}</p>
      </article>
      <article class="status-card">
        <span class="label">Game folder</span>
        <strong>{hasGameFolder ? "Configured" : "Missing"}</strong>
        <p>{config.gameFolder || "Detect Steam or select the folder manually."}</p>
      </article>
      <article class="status-card">
        <span class="label">Modloader</span>
        <strong>{modloaderStatus}</strong>
        <p>{config.modloaderRelease ? `Release ${config.modloaderRelease}` : "GitHub Release install is available once the game folder is set."}</p>
      </article>
    </section>

    <section class="layout">
      <article class="panel">
        <div class="panel-head">
          <div>
            <p class="eyebrow">Game</p>
            <h2>Detection and launch</h2>
          </div>
        </div>

        {#if detectedGame}
          <div class="detected">
            <div>
              <strong>Steam install detected</strong>
              <p>{detectedGame.gameFolder}</p>
            </div>
            <button class="secondary" on:click={useDetectedGame}>Use this install</button>
          </div>
        {:else}
          <div class="detected muted">
            <div>
              <strong>No Steam install detected</strong>
              <p>Select the executable or the game folder manually.</p>
            </div>
            <button class="secondary" on:click={load}>Scan again</button>
          </div>
        {/if}

        <div class="mode-row">
          <button class:active={config.launchMode === "steam"} on:click={() => setLaunchMode("steam")}>Steam</button>
          <button class:active={config.launchMode === "executable"} on:click={() => setLaunchMode("executable")}>Executable</button>
        </div>

        <div class="actions">
          <button class="secondary" on:click={chooseGameFolder}>Select game folder</button>
          <button class="secondary" on:click={chooseExecutable}>Select executable</button>
        </div>

        <label>
          Mods website URL
          <input bind:value={config.siteUrl} on:change={() => saveAndRefresh("Website URL saved.")} placeholder="https://oppw4.prism.am" />
        </label>
      </article>

      <aside class="panel">
        <div class="panel-head">
          <div>
            <p class="eyebrow">Modloader</p>
            <h2>dinput8 install</h2>
          </div>
        </div>

        <label>
          GitHub release repository
          <input bind:value={config.modloaderRepo} on:change={() => saveAndRefresh("Repository saved.")} placeholder="owner/repository" />
        </label>

        <div class="actions vertical">
          <button class="primary" disabled={busy || !hasGameFolder} on:click={installModloader}>
            {isInstalled ? "Update modloader" : "Install modloader"}
          </button>
          <button class="danger" disabled={busy || !isInstalled} on:click={restoreModloader}>Restore original files</button>
        </div>

        {#if config.installedFiles.length}
          <div class="installed-list">
            <span class="label">Installed files</span>
            {#each config.installedFiles as file}
              <p>{file.relativePath}</p>
            {/each}
          </div>
        {/if}
      </aside>
    </section>
  {/if}
</main>
