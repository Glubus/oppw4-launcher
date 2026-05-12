<script lang="ts">
  import type { Character } from "$lib/api";

  export let characters: Character[] = [];
  export let value = "";
  export let placeholder = "All characters";
  export let valueKey: "slug" | "id" = "slug";
  export let includeAll = true;
  export let onChange: () => void = () => {};

  let search = "";
  let details: HTMLDetailsElement;

  $: selected = characters.find((item) => String(item[valueKey]) === String(value));
  $: filtered = characters.filter((item) => {
    const needle = search.trim().toLowerCase();
    if (!needle) return true;
    return `${item.displayName} ${item.pack}`.toLowerCase().includes(needle);
  });

  function select(next: string) {
    value = next;
    search = "";
    if (details) details.open = false;
    onChange();
  }
</script>

<details class="relative z-40 w-full" bind:this={details}>
  <summary class="flex h-10 w-full cursor-pointer list-none items-center justify-between rounded-md border border-white/12 bg-background/55 px-3 text-sm font-medium text-foreground shadow-sm outline-none transition-colors hover:bg-white/10 focus-visible:ring-2 focus-visible:ring-ring">
    <span class="truncate">{selected?.displayName ?? placeholder}</span>
    <span class="text-muted-foreground">⌄</span>
  </summary>
  <div class="absolute z-50 mt-2 w-full rounded-lg border border-white/12 bg-popover/95 p-2 text-popover-foreground shadow-2xl backdrop-blur-md">
    <input class="mb-2 h-9 w-full rounded-md border border-white/12 bg-background/55 px-3 text-sm outline-none placeholder:text-muted-foreground focus-visible:ring-2 focus-visible:ring-ring" bind:value={search} placeholder="Search character..." />
    <div class="max-h-72 overflow-y-auto">
      {#if includeAll}
        <button class="flex h-8 w-full items-center justify-start rounded-md px-2 text-sm hover:bg-white/10 {value === '' ? 'bg-accent text-accent-foreground' : ''}" type="button" on:click={() => select("")}>
          {placeholder}
        </button>
      {/if}
      {#each filtered as item}
        <button class="flex h-8 w-full items-center justify-between gap-2 rounded-md px-2 text-sm hover:bg-white/10" class:bg-accent={String(item[valueKey]) === String(value)} class:text-accent-foreground={String(item[valueKey]) === String(value)} type="button" on:click={() => select(String(item[valueKey]))}>
          <span class="truncate">{item.displayName}</span>
          <span class="rounded-full border border-border px-1.5 py-0.5 text-[0.65rem] text-muted-foreground">{item.isDlc ? "DLC" : "Base"}</span>
        </button>
      {/each}
      {#if filtered.length === 0}
        <p class="px-3 py-4 text-sm text-muted-foreground">No character found.</p>
      {/if}
    </div>
  </div>
</details>
