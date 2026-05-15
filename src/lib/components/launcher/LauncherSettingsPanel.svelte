<script lang="ts">
  import Button from "$lib/components/ui/Button.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import Label from "$lib/components/ui/Label.svelte";
  import LauncherHealthPanel from "./LauncherHealthPanel.svelte";
  import type { DetectedGame, HealthCheckItem, LaunchMode, LauncherConfig } from "./types";

  export let config: LauncherConfig;
  export let detectedGame: DetectedGame | null = null;
  export let hasGameFolder = false;
  export let healthItems: HealthCheckItem[] = [];
  export let busy = false;
  export let onUseDetected: () => void = () => {};
  export let onSetLaunchMode: (mode: LaunchMode) => void = () => {};
  export let onChooseGameFolder: () => void = () => {};
  export let onChooseExecutable: () => void = () => {};
  export let onRepositoryChange: () => void = () => {};
  export let onRunHealth: () => void = () => {};
  export let onExportDiagnostics: () => void = () => {};
  export let onDebugLogsChange: () => void = () => {};
</script>

<div class="grid gap-5 p-2 pt-5">
  <div>
    <h2 class="text-xl font-black">Settings</h2>
    <p class="mt-1 text-sm text-muted-foreground">Game paths, launch mode, patcher source, and diagnostics.</p>
  </div>

  <section class="grid gap-5 border-b border-white/10 pb-5">
    <div class="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
      <div class="min-w-0">
        <h3 class="font-black">Game install</h3>
        <p class="mt-1 text-sm text-muted-foreground">
          {hasGameFolder ? "Local mod management is ready." : "Select the game folder to enable local mod management."}
        </p>
      </div>
      <div class="flex flex-wrap gap-2">
        <Button variant="outline" on:click={onChooseGameFolder}>{config.gameFolder ? "Change game folder" : "Select game folder"}</Button>
        <Button variant="outline" on:click={onChooseExecutable}>{config.gameExecutablePath ? "Change executable" : "Select executable"}</Button>
      </div>
    </div>

    <div class="grid gap-3 lg:grid-cols-2">
      <div class="min-w-0 rounded-md border border-white/10 bg-background/45 p-3">
        <p class="text-xs font-black uppercase tracking-[0.16em] text-muted-foreground">Game folder</p>
        <p class="mt-2 break-words text-sm font-bold text-foreground">{config.gameFolder || "Not selected"}</p>
      </div>
      <div class="min-w-0 rounded-md border border-white/10 bg-background/45 p-3">
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
  </section>

  <section class="grid gap-3 border-b border-white/10 pb-5">
    <div>
      <h3 class="font-black">Launch method</h3>
      <p class="mt-1 text-sm text-muted-foreground">Steam works for most installs. Use executable only for custom setups.</p>
    </div>
    <div class="grid grid-cols-2 overflow-hidden rounded-lg border border-white/10 bg-background/45">
      <button class="h-11 font-black {config.launchMode === 'steam' ? 'bg-primary/20 text-primary' : 'text-muted-foreground hover:bg-white/8'}" type="button" on:click={() => onSetLaunchMode("steam")}>Steam</button>
      <button class="h-11 font-black {config.launchMode === 'executable' ? 'bg-primary/20 text-primary' : 'text-muted-foreground hover:bg-white/8'}" type="button" on:click={() => onSetLaunchMode("executable")}>Executable</button>
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

  <section class="grid gap-3">
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
</div>
