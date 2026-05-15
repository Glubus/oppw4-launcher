<script lang="ts">
  import Button from "$lib/components/ui/Button.svelte";
  import type { PotentialOverlapGroup } from "./types";

  export let groups: PotentialOverlapGroup[] = [];
  export let launchDisabled = false;
  export let dontWarnAgain = false;
  export let onCancel: () => void = () => {};
  export let onLaunch: (dontWarnAgain: boolean) => void = () => {};
</script>

<div class="fixed inset-0 z-50 grid place-items-center p-4">
  <button class="absolute inset-0 bg-black/70 backdrop-blur-sm" type="button" aria-label="Cancel launch" on:click={onCancel}></button>
  <div class="relative grid max-h-[86vh] w-full max-w-2xl gap-4 overflow-auto rounded-lg border border-amber-300/35 bg-background p-5 shadow-2xl" role="dialog" aria-modal="true" aria-label="Potential mod overlap">
    <div>
      <p class="text-xs font-black uppercase tracking-[0.18em] text-amber-300">Warning</p>
      <h2 class="mt-1 text-2xl font-black">Potential mod overlap</h2>
      <p class="mt-2 text-sm leading-6 text-muted-foreground">Multiple active mods may target the same skin area. You can launch anyway if this setup is intentional.</p>
    </div>

    <div class="grid gap-3">
      {#each groups as group}
        <section class="rounded-md border border-amber-300/25 bg-amber-400/10 p-3">
          <p class="text-sm font-black text-amber-100">{group.characterLabel} / {group.modType}</p>
          <ul class="mt-2 grid gap-1 text-sm text-muted-foreground">
            {#each group.mods as mod}
              <li class="truncate">- {mod.name}</li>
            {/each}
          </ul>
        </section>
      {/each}
    </div>

    <label class="flex items-center gap-3 rounded-md border border-white/10 bg-background/45 px-3 py-2 text-sm font-bold text-foreground">
      <input class="sr-only" type="checkbox" bind:checked={dontWarnAgain} />
      <span class="grid h-4 w-4 shrink-0 place-items-center rounded border border-white/25 {dontWarnAgain ? 'border-amber-300 bg-amber-300 text-black' : 'bg-background/70'}" aria-hidden="true">
        {#if dontWarnAgain}
          <svg class="h-3 w-3" viewBox="0 0 16 16" fill="none">
            <path d="M3.25 8.25L6.5 11.5L12.75 4.5" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        {/if}
      </span>
      Don't warn me about potential overlaps
    </label>

    <div class="flex flex-wrap justify-end gap-2">
      <Button variant="outline" on:click={onCancel}>Cancel</Button>
      <Button disabled={launchDisabled} on:click={() => onLaunch(dontWarnAgain)}>
        {launchDisabled ? "Working..." : "Launch anyway"}
      </Button>
    </div>
  </div>
</div>
