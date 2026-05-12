<script lang="ts">
  import Button from "$lib/components/ui/Button.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import Label from "$lib/components/ui/Label.svelte";
  import type { DetectedGame, LaunchMode, LauncherConfig } from "./types";

  export let config: LauncherConfig;
  export let detectedGame: DetectedGame | null = null;
  export let hasGameFolder = false;
  export let onUseDetected: () => void = () => {};
  export let onSetLaunchMode: (mode: LaunchMode) => void = () => {};
  export let onChooseGameFolder: () => void = () => {};
  export let onChooseExecutable: () => void = () => {};
  export let onRepositoryChange: () => void = () => {};
</script>

<div class="grid gap-5 p-2 pt-5">
  <div>
    <h2 class="text-xl font-black">Settings</h2>
    <p class="mt-2 text-sm leading-6 text-muted-foreground">Only change this if the launcher picked the wrong game folder or if you are testing another patcher repo.</p>
  </div>

  <div class="grid gap-3 rounded-lg border border-white/10 bg-background/45 p-4">
    <div class="flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
      <div>
        <p class="font-black">Current game install</p>
        <p class="mt-1 text-sm text-muted-foreground">{hasGameFolder ? "Ready to scan installed mods." : "Select the game folder to enable local mod management."}</p>
      </div>
      <span class="w-fit rounded-full border border-white/15 bg-white/8 px-3 py-1 text-xs font-black uppercase tracking-wide text-muted-foreground">
        {config.launchMode === "steam" ? "Steam" : "Executable"}
      </span>
    </div>

    <div class="grid gap-3 md:grid-cols-2">
      <div class="min-w-0 rounded-md border border-white/10 bg-card/60 p-3">
        <p class="text-xs font-black uppercase tracking-[0.16em] text-muted-foreground">Game folder</p>
        <p class="mt-2 break-words text-sm font-bold text-foreground">{config.gameFolder || "Not selected"}</p>
      </div>
      <div class="min-w-0 rounded-md border border-white/10 bg-card/60 p-3">
        <p class="text-xs font-black uppercase tracking-[0.16em] text-muted-foreground">Executable</p>
        <p class="mt-2 break-words text-sm font-bold text-foreground">{config.gameExecutablePath || "Steam launch / not selected"}</p>
      </div>
    </div>
  </div>

  {#if detectedGame}
    <div class="flex flex-col gap-3 rounded-lg border border-primary/20 bg-primary/10 p-4 sm:flex-row sm:items-center sm:justify-between">
      <div>
        <p class="font-black">Steam install detected</p>
        <p class="mt-1 break-words text-sm text-muted-foreground">{detectedGame.gameFolder}</p>
      </div>
      <Button variant="outline" on:click={onUseDetected}>Use this install</Button>
    </div>
  {/if}

  <div class="grid gap-4 rounded-lg border border-white/10 bg-background/45 p-4">
    <p class="font-black">Launch method</p>
    <div class="grid grid-cols-2 overflow-hidden rounded-lg border border-white/10 bg-background/45">
      <button class="h-11 font-black {config.launchMode === 'steam' ? 'bg-primary/20 text-primary' : 'text-muted-foreground hover:bg-white/8'}" type="button" on:click={() => onSetLaunchMode("steam")}>Steam</button>
      <button class="h-11 font-black {config.launchMode === 'executable' ? 'bg-primary/20 text-primary' : 'text-muted-foreground hover:bg-white/8'}" type="button" on:click={() => onSetLaunchMode("executable")}>Executable</button>
    </div>
    <div class="flex flex-wrap gap-2">
      <Button variant="outline" on:click={onChooseGameFolder}>{config.gameFolder ? "Change game folder" : "Select game folder"}</Button>
      <Button variant="outline" on:click={onChooseExecutable}>{config.gameExecutablePath ? "Change executable" : "Select executable"}</Button>
    </div>
  </div>

  <Label>
    Patcher GitHub repository
    <Input bind:value={config.modloaderRepo} on:change={onRepositoryChange} placeholder="owner/repository" />
  </Label>
</div>
