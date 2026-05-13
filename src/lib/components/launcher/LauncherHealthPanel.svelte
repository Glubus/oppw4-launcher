<script lang="ts">
  import Button from "$lib/components/ui/Button.svelte";
  import type { HealthCheckItem } from "./types";

  export let items: HealthCheckItem[] = [];
  export let busy = false;
  export let onRun: () => void = () => {};
  export let onExport: () => void = () => {};

  function levelClass(level: HealthCheckItem["level"]) {
    if (level === "ok") return "border-emerald-300/30 bg-emerald-400/10 text-emerald-100";
    if (level === "warn") return "border-amber-300/35 bg-amber-400/10 text-amber-100";
    return "border-red-300/35 bg-red-400/10 text-red-100";
  }
</script>

<section class="grid gap-3 rounded-lg border border-white/10 bg-background/45 p-4">
  <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
    <div>
      <p class="font-black">Health check</p>
      <p class="mt-1 text-sm text-muted-foreground">Checks patcher state, mods folder, metadata, dependencies and latest loader log.</p>
    </div>
    <div class="flex flex-wrap gap-2">
      <Button variant="outline" disabled={busy} on:click={onRun}>Run check</Button>
      <Button variant="outline" disabled={busy} on:click={onExport}>Export diagnostics</Button>
    </div>
  </div>

  {#if items.length}
    <div class="grid gap-2">
      {#each items as item}
        <div class={`rounded-md border p-3 ${levelClass(item.level)}`}>
          <div class="flex items-center justify-between gap-3">
            <p class="font-black">{item.title}</p>
            <span class="text-xs font-black uppercase tracking-wide">{item.level}</span>
          </div>
          <p class="mt-1 text-sm leading-5 opacity-85">{item.detail}</p>
        </div>
      {/each}
    </div>
  {:else}
    <p class="rounded-md border border-white/10 bg-card/60 p-3 text-sm text-muted-foreground">No health check run yet.</p>
  {/if}
</section>
