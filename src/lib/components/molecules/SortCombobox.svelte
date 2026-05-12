<script lang="ts">
  export let value = "recent";
  export let onChange: () => void = () => {};

  let details: HTMLDetailsElement;

  const options = [
    { value: "recent", label: "Recent drops" },
    { value: "popular", label: "Most upvoted" },
    { value: "viewed", label: "Most viewed" }
  ];

  $: selected = options.find((item) => item.value === value) ?? options[0];

  function select(next: string) {
    value = next;
    if (details) details.open = false;
    onChange();
  }
</script>

<details class="relative z-40 w-full" bind:this={details}>
  <summary class="flex h-10 w-full cursor-pointer list-none items-center justify-between rounded-md border border-white/12 bg-background/55 px-3 text-sm font-medium text-foreground shadow-sm outline-none transition-colors hover:bg-white/10 focus-visible:ring-2 focus-visible:ring-ring">
    <span class="truncate">{selected.label}</span>
    <span class="text-muted-foreground">⌄</span>
  </summary>
  <div class="absolute z-50 mt-2 w-full rounded-lg border border-white/12 bg-popover/95 p-2 text-popover-foreground shadow-2xl backdrop-blur-md">
    {#each options as item}
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
