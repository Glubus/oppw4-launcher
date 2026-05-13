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
</script>

<section class="flex flex-col gap-4 rounded-lg border border-white/10 bg-card/86 p-5 shadow-[0_18px_60px_rgba(0,0,0,0.25)] backdrop-blur-md md:flex-row md:items-end md:justify-between">
  <div>
    <p class="text-xs font-black uppercase tracking-[0.22em] text-primary/90">Desktop launcher</p>
    <h1 class="mt-1 text-4xl font-black tracking-tight">Launch and manage mods</h1>
    <p class="mt-2 max-w-2xl text-sm leading-6 text-muted-foreground">
      Current: <span class="font-bold text-foreground">{currentRelease}</span>
      {#if latestReleaseLabel}
        · Latest: <span class="font-bold text-foreground">{latestReleaseLabel}</span>
      {/if}
      {#if latestReleaseDate}
        · Released <span class="font-bold text-foreground">{latestReleaseDate}</span>
      {/if}
    </p>
    <p class="mt-1 text-xs font-bold uppercase tracking-wide text-muted-foreground">{modloaderStatus}</p>
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
