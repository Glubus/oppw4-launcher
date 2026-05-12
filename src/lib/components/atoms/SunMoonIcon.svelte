<script lang="ts">
  export let mode: "sun" | "moon" = "moon";
  export let className = "h-4 w-4";
  export { className as class };
</script>

<svg class={className} viewBox="0 0 24 24" fill="none" aria-hidden="true">
  {#if mode === "sun"}
    <circle cx="12" cy="12" r="4" stroke="currentColor" stroke-width="2" />
    <path d="M12 2v2m0 16v2M4 12H2m20 0h-2M5 5l1.5 1.5M17.5 17.5 19 19M19 5l-1.5 1.5M6.5 17.5 5 19" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
  {:else}
    <path d="M20 14.5A7.5 7.5 0 0 1 9.5 4 8.5 8.5 0 1 0 20 14.5Z" stroke="currentColor" stroke-width="2" stroke-linejoin="round" />
  {/if}
</svg>
