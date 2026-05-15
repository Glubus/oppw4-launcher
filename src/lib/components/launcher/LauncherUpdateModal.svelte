<script lang="ts">
  import Button from "$lib/components/ui/Button.svelte";
  import type { LauncherUpdateInfo } from "./types";

  export let update: LauncherUpdateInfo;
  export let installing = false;
  export let onInstall: () => void = () => {};
  export let onDismiss: () => void = () => {};
</script>

<div class="fixed inset-0 z-50 grid place-items-center bg-black/62 p-4 backdrop-blur-sm">
  <section class="w-full max-w-md rounded-lg border border-white/12 bg-card p-5 shadow-2xl">
    <p class="text-xs font-black uppercase tracking-[0.2em] text-primary/90">Launcher update</p>
    <h2 class="mt-2 text-2xl font-black">Version {update.latestVersion} is available</h2>
    <p class="mt-2 text-sm leading-6 text-muted-foreground">
      Current version: <span class="font-bold text-foreground">{update.currentVersion}</span>
      {#if update.assetName}
        · Asset: <span class="font-bold text-foreground">{update.assetName}</span>
      {/if}
    </p>
    {#if update.publishedAt}
      <p class="mt-1 text-sm text-muted-foreground">Published {update.publishedAt}</p>
    {/if}

    <div class="mt-5 grid gap-2 sm:grid-cols-2">
      <Button disabled={installing} on:click={onInstall}>{installing ? "Downloading..." : "Update now"}</Button>
      <Button variant="outline" disabled={installing} on:click={onDismiss}>Not now</Button>
    </div>
  </section>
</div>
