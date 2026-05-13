<script lang="ts">
  import { MOD_TYPE_OPTIONS } from "$lib/api";

  export let value = "";
  export let onChange: (value: string) => void = () => {};

  let details: HTMLDetailsElement;

  $: selected = MOD_TYPE_OPTIONS.find((item) => item.value === value) ?? MOD_TYPE_OPTIONS[0];

  function select(next: string) {
    value = next;
    if (details) details.open = false;
    onChange(next);
  }
</script>

<details class="relative z-40 w-full" bind:this={details}>
  <summary class="flex h-10 w-full cursor-pointer list-none items-center justify-between rounded-md border border-white/12 bg-background/55 px-3 text-sm font-medium text-foreground shadow-sm outline-none transition-colors hover:bg-white/10 focus-visible:ring-2 focus-visible:ring-ring">
    <span class="truncate">{selected.label}</span>
    <span class="text-muted-foreground">⌄</span>
  </summary>
  <div class="absolute z-50 mt-2 w-full rounded-lg border border-white/12 bg-popover/95 p-2 text-popover-foreground shadow-2xl backdrop-blur-md">
    {#each MOD_TYPE_OPTIONS as item}
      <button
        class="flex h-8 w-full items-center justify-start rounded-md px-2 text-sm hover:bg-white/10"
        class:bg-accent={item.value === value}
        class:text-accent-foreground={item.value === value}
        type="button"
        on:click={() => select(item.value)}
      >
        {item.label}
      </button>
    {/each}
  </div>
</details>
