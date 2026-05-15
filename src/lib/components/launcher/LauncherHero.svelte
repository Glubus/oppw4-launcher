<script lang="ts">
  import Button from "$lib/components/ui/Button.svelte";

  export let currentRelease = "Not installed";
  export let latestReleaseLabel = "";
  export let latestReleaseDate = "";
  export let modloaderStatus = "Missing";
  export let updateLabel = "";
  export let isDesktop = false;
  export let busy = false;
  export let loading = false;
  export let hasGameFolder = false;
  export let canLaunch = false;
  export let hasLatestRelease = false;
  export let onInstall: () => void = () => {};
  export let onLaunch: () => void = () => {};
  export let onCheck: () => void = () => {};

  $: showStatus = modloaderStatus.toLowerCase() !== "installed";
</script>

<section class="grid gap-4 rounded-lg border border-white/10 bg-card/86 p-5 shadow-[0_18px_60px_rgba(0,0,0,0.25)] backdrop-blur-md lg:grid-cols-[minmax(0,1fr)_auto] lg:items-end">
  <div class="min-w-0">
    <p class="text-xs font-black uppercase tracking-[0.22em] text-primary/90">Desktop launcher</p>
    <h1 class="mt-1 text-4xl font-black tracking-tight">Launch and manage mods</h1>
    <p class="mt-2 max-w-2xl text-sm leading-6 text-muted-foreground">
      Installed: <span class="font-bold text-foreground">{currentRelease}</span>
      {#if latestReleaseLabel}
        · Latest: <span class="font-bold text-foreground">{latestReleaseLabel}</span>
      {/if}
      {#if latestReleaseDate}
        · Released <span class="font-bold text-foreground">{latestReleaseDate}</span>
      {/if}
    </p>
    {#if showStatus}
      <p class="mt-1 text-xs font-bold uppercase tracking-wide text-muted-foreground">{modloaderStatus}</p>
    {/if}
    <ol class="mt-4 grid max-w-3xl gap-2 text-sm sm:grid-cols-3">
      <li class="rounded-md border border-white/10 bg-background/45 px-3 py-2">
        <span class="font-black text-primary">1.</span>
        <span class="ml-1 font-bold text-foreground">Select your game folder</span>
      </li>
      <li class="rounded-md border border-white/10 bg-background/45 px-3 py-2">
        <span class="font-black text-primary">2.</span>
        <span class="ml-1 font-bold text-foreground">Install or update the patcher</span>
      </li>
      <li class="rounded-md border border-white/10 bg-background/45 px-3 py-2">
        <span class="font-black text-primary">3.</span>
        <span class="ml-1 font-bold text-foreground">Install mods here; web links open in your browser</span>
      </li>
    </ol>
  </div>
  <div class="flex flex-wrap gap-2">
    <Button variant="outline" size="lg" disabled={!isDesktop || busy || !hasGameFolder} on:click={onCheck}>Check patcher</Button>
    {#if updateLabel}
      <Button size="lg" disabled={!isDesktop || busy || !hasGameFolder || !hasLatestRelease} on:click={onInstall}>{updateLabel}</Button>
    {/if}
    <Button size="lg" disabled={!isDesktop || loading || busy || !canLaunch} on:click={onLaunch}>
      {busy ? "Working..." : "Launch game"}
    </Button>
  </div>
</section>
