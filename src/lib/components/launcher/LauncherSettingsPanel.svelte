<script lang="ts">
  import Button from "$lib/components/ui/Button.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import Label from "$lib/components/ui/Label.svelte";
  import LauncherHealthPanel from "./LauncherHealthPanel.svelte";
  import type { DetectedGame, HealthCheckItem, LaunchMode, LauncherConfig, LauncherUpdateInfo, ReleaseInfo } from "./types";

  export let config: LauncherConfig;
  export let detectedGame: DetectedGame | null = null;
  export let hasGameFolder = false;
  export let healthItems: HealthCheckItem[] = [];
  export let modloaderStatus = "Missing";
  export let latestRelease: ReleaseInfo | null = null;
  export let needsPatcherUpdate = false;
  export let launcherUpdate: LauncherUpdateInfo | null = null;
  export let checkingLauncherUpdate = false;
  export let installingLauncherUpdate = false;
  export let busy = false;
  export let onUseDetected: () => void = () => {};
  export let onSetLaunchMode: (mode: LaunchMode) => void = () => {};
  export let onChooseGameFolder: () => void = () => {};
  export let onChooseExecutable: () => void = () => {};
  export let onRepositoryChange: () => void = () => {};
  export let onRunHealth: () => void = () => {};
  export let onExportDiagnostics: () => void = () => {};
  export let onDebugLogsChange: () => void = () => {};
  export let onOverlapWarningChange: () => void = () => {};
  export let onCheckLauncherUpdate: () => void = () => {};
  export let onInstallLauncherUpdate: () => void = () => {};

  type SettingsTab = "game" | "patcher" | "launcher";

  let activeTab: SettingsTab = "game";

  const settingsTabs: Array<{ id: SettingsTab; label: string }> = [
    { id: "game", label: "Game" },
    { id: "patcher", label: "Patcher" },
    { id: "launcher", label: "Launcher" },
  ];

  const normalizeVersion = (version: string) => version.trim().replace(/^v/i, "");

  $: launcherVersionsDiffer = launcherUpdate
    ? normalizeVersion(launcherUpdate.currentVersion) !== normalizeVersion(launcherUpdate.latestVersion)
    : false;
  $: launcherUpdateText = launcherUpdate?.available
    ? `Version ${launcherUpdate.latestVersion} is available.`
    : launcherUpdate
      ? `Current build matches GitHub release ${launcherUpdate.latestVersion}.`
      : "Check GitHub releases for a launcher build.";
  $: patcherStatusTone = modloaderStatus.toLowerCase().includes("installed") && !needsPatcherUpdate
    ? "bg-emerald-500 shadow-[0_0_0_4px_rgba(16,185,129,0.16)]"
    : needsPatcherUpdate
      ? "bg-amber-400 shadow-[0_0_0_4px_rgba(251,191,36,0.16)]"
      : "bg-destructive shadow-[0_0_0_4px_rgba(239,68,68,0.16)]";
  $: usesExecutableOverride = config.launchMode === "executable";
</script>

<div class="grid gap-5 p-2 pt-5">
  <div>
    <h2 class="text-xl font-black">Settings</h2>
    <p class="mt-1 text-sm text-muted-foreground">Game paths, patcher tools, diagnostics, updates, and logs.</p>
  </div>

  <div class="flex gap-1 overflow-x-auto border-b border-white/10">
    {#each settingsTabs as tab}
      <button
        class="h-10 shrink-0 border-b-2 px-4 text-sm font-black transition {activeTab === tab.id
          ? 'border-primary text-primary'
          : 'border-transparent text-muted-foreground hover:text-foreground'}"
        type="button"
        on:click={() => (activeTab = tab.id)}
      >
        {tab.label}
      </button>
    {/each}
  </div>

  {#if activeTab === "game"}
    <section class="grid gap-5">
      <section class="grid gap-3 border-b border-white/10 pb-5">
        <div>
          <h3 class="font-black">Launch method</h3>
          <p class="mt-1 text-sm text-muted-foreground">Steam works for most installs. Enable executable override only for custom setups.</p>
        </div>
        <div class="grid grid-cols-2 overflow-hidden rounded-lg border border-white/10 bg-background/45">
          <button class="h-11 font-black {config.launchMode === 'steam' ? 'bg-primary/20 text-primary' : 'text-muted-foreground hover:bg-white/8'}" type="button" on:click={() => onSetLaunchMode("steam")}>Steam</button>
          <button class="h-11 font-black {config.launchMode === 'executable' ? 'bg-primary/20 text-primary' : 'text-muted-foreground hover:bg-white/8'}" type="button" on:click={() => onSetLaunchMode("executable")}>Executable</button>
        </div>
      </section>

      <section class="grid gap-3">
        <label class="flex items-center gap-3 rounded-md border border-white/10 bg-background/45 px-3 py-2 text-sm font-bold text-foreground">
          <input
            class="sr-only"
            type="checkbox"
            checked={usesExecutableOverride}
            on:change={(event) => onSetLaunchMode(event.currentTarget.checked ? "executable" : "steam")}
          />
          <span class="grid h-4 w-4 shrink-0 place-items-center rounded border border-white/25 {usesExecutableOverride ? 'border-primary bg-primary text-primary-foreground' : 'bg-background/70'}" aria-hidden="true">
            {#if usesExecutableOverride}
              <svg class="h-3 w-3" viewBox="0 0 16 16" fill="none">
                <path d="M3.25 8.25L6.5 11.5L12.75 4.5" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
              </svg>
            {/if}
          </span>
          Override Steam launch with a custom executable
        </label>

        <div class="grid gap-3 transition {usesExecutableOverride ? '' : 'pointer-events-none opacity-45 grayscale'}">
          <div class="grid min-h-16 gap-3 sm:grid-cols-[minmax(0,1fr)_auto] sm:items-start">
            <div class="min-w-0">
              <h3 class="font-black">Game install</h3>
              <p class="mt-1 text-sm text-muted-foreground">
                {hasGameFolder ? "Local mod management is ready." : "Select the game folder to enable local mod management."}
              </p>
            </div>
            <div class="flex flex-wrap gap-2 sm:justify-end">
              <Button variant="outline" on:click={onChooseGameFolder}>{config.gameFolder ? "Change game folder" : "Select game folder"}</Button>
              <Button variant="outline" disabled={!usesExecutableOverride} on:click={onChooseExecutable}>{config.gameExecutablePath ? "Change executable" : "Select executable"}</Button>
            </div>
          </div>

          <div class="grid gap-3 lg:grid-cols-2">
            <div class="min-w-0 rounded-md border border-white/10 bg-background/45 p-3">
              <p class="text-xs font-black uppercase tracking-[0.16em] text-muted-foreground">Game folder</p>
              <p class="mt-2 break-words text-sm font-bold text-foreground">{config.gameFolder || "Not selected"}</p>
            </div>
            <div class="min-w-0 rounded-md border border-white/10 bg-background/45 p-3 {usesExecutableOverride ? '' : 'pointer-events-none'}">
              <p class="text-xs font-black uppercase tracking-[0.16em] text-muted-foreground">Executable</p>
              <p class="mt-2 break-words text-sm font-bold text-foreground">{config.gameExecutablePath || "Steam launch / not selected"}</p>
            </div>
          </div>

          {#if detectedGame}
            <div class="flex flex-col gap-3 rounded-md border border-primary/20 bg-primary/10 p-3 sm:flex-row sm:items-center sm:justify-between">
              <div class="min-w-0">
                <p class="font-black">Steam install detected</p>
                <p class="mt-1 break-words text-sm text-muted-foreground">{detectedGame.gameFolder}</p>
              </div>
              <Button variant="outline" on:click={onUseDetected}>Use this install</Button>
            </div>
          {/if}
        </div>
      </section>
    </section>
  {:else if activeTab === "patcher"}
    <section class="grid gap-5">
      <section class="grid gap-3 border-b border-white/10 pb-5">
        <div class="grid min-h-16 gap-3 sm:grid-cols-[minmax(0,1fr)_auto] sm:items-start">
          <div class="min-w-0">
            <h3 class="font-black">Patcher status</h3>
            <p class="mt-1 text-sm text-muted-foreground">
              {needsPatcherUpdate ? "A different patcher release is available." : "Installed patcher state from the local game folder."}
            </p>
          </div>
          <span class="mt-1 h-3 w-3 shrink-0 justify-self-end rounded-full {patcherStatusTone}" title={modloaderStatus} aria-label={modloaderStatus}></span>
        </div>
        <div class="grid gap-3 lg:grid-cols-2">
          <div class="min-w-0 rounded-md border border-white/10 bg-background/45 p-3">
            <p class="text-xs font-black uppercase tracking-[0.16em] text-muted-foreground">Installed release</p>
            <p class="mt-2 break-words text-sm font-bold text-foreground">{config.modloaderRelease || "Not installed"}</p>
          </div>
          <div class="min-w-0 rounded-md border border-white/10 bg-background/45 p-3">
            <p class="text-xs font-black uppercase tracking-[0.16em] text-muted-foreground">Latest release</p>
            <p class="mt-2 break-words text-sm font-bold text-foreground">{latestRelease?.tagName || "Not checked"}</p>
          </div>
        </div>
      </section>

      <section class="grid gap-3 border-b border-white/10 pb-5">
        <details>
          <summary class="flex h-10 cursor-pointer list-none items-center justify-between rounded-md border border-white/12 bg-background/55 px-3 text-sm font-black text-foreground hover:bg-white/10">
            Advanced patcher source
            <span class="text-muted-foreground">⌄</span>
          </summary>
          <div class="mt-3 grid gap-2">
            <Label>
              Patcher GitHub repository
              <Input bind:value={config.modloaderRepo} on:change={onRepositoryChange} placeholder="owner/repository" />
            </Label>
            <p class="text-xs leading-5 text-muted-foreground">Expected format: owner/repository.</p>
          </div>
        </details>
      </section>

      <LauncherHealthPanel items={healthItems} {busy} onRun={onRunHealth} onExport={onExportDiagnostics} />
    </section>
  {:else}
    <section class="grid gap-5">
      <section class="grid gap-3 border-b border-white/10 pb-5">
        <div class="grid min-h-16 gap-3 sm:grid-cols-[minmax(0,1fr)_auto] sm:items-start">
          <div>
            <h3 class="font-black">Launcher update</h3>
            <p class="mt-1 text-sm text-muted-foreground">
              {#if launcherUpdate && launcherVersionsDiffer && !launcherUpdate.available}
                Current build {launcherUpdate.currentVersion} differs from GitHub release {launcherUpdate.latestVersion}, but <span class="font-black text-foreground">no installable asset is available</span> for this platform.
              {:else}
                {launcherUpdateText}
              {/if}
            </p>
          </div>
          <div class="flex flex-wrap gap-2 sm:justify-end">
            <Button variant="outline" disabled={checkingLauncherUpdate || installingLauncherUpdate} on:click={onCheckLauncherUpdate}>{checkingLauncherUpdate ? "Checking..." : "Check update"}</Button>
            {#if launcherUpdate?.available}
              <Button disabled={installingLauncherUpdate} on:click={onInstallLauncherUpdate}>{installingLauncherUpdate ? "Downloading..." : "Install update"}</Button>
            {/if}
          </div>
        </div>
        {#if launcherUpdate?.assetName}
          <p class="rounded-md border border-white/10 bg-background/45 p-3 text-sm text-muted-foreground">
            Asset: <span class="font-bold text-foreground">{launcherUpdate.assetName}</span>
          </p>
        {/if}
        {#if launcherUpdate}
          <div class="grid gap-3 lg:grid-cols-2">
            <div class="min-w-0 rounded-md border border-white/10 bg-background/45 p-3">
              <p class="text-xs font-black uppercase tracking-[0.16em] text-muted-foreground">Current build</p>
              <p class="mt-2 break-words text-sm font-bold text-foreground">{launcherUpdate.currentVersion}</p>
            </div>
            <div class="min-w-0 rounded-md border border-white/10 bg-background/45 p-3">
              <p class="text-xs font-black uppercase tracking-[0.16em] text-muted-foreground">GitHub release</p>
              <p class="mt-2 break-words text-sm font-bold text-foreground">{launcherUpdate.latestVersion}</p>
            </div>
          </div>
        {/if}
      </section>

      <section class="grid gap-3">
        <div class="flex flex-col gap-3 border-b border-white/10 pb-5 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <h3 class="font-black">Overlap warnings</h3>
            <p class="mt-1 text-sm text-muted-foreground">Warn before launching when multiple active mods may target the same character and mod type.</p>
          </div>
          <Button variant={config.warnOnPotentialOverlap ? "default" : "outline"} on:click={() => {
            config.warnOnPotentialOverlap = !config.warnOnPotentialOverlap;
            onOverlapWarningChange();
          }}>
            {config.warnOnPotentialOverlap ? "Disable overlap warnings" : "Enable overlap warnings"}
          </Button>
        </div>

        <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <h3 class="font-black">Launcher logs</h3>
            <p class="mt-1 text-sm text-muted-foreground">Important actions are written next to the launcher executable in logs/. Enable debug logs for detailed traces.</p>
          </div>
          <Button variant={config.debugLogs ? "default" : "outline"} on:click={() => {
            config.debugLogs = !config.debugLogs;
            onDebugLogsChange();
          }}>
            {config.debugLogs ? "Disable debug logs" : "Enable debug logs"}
          </Button>
        </div>
      </section>
    </section>
  {/if}
</div>
