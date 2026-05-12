<script lang="ts">
  export let direction: "left" | "right" = "right";
  export let className = "h-4 w-4";
  export { className as class };
</script>

<svg class={className} viewBox="0 0 24 24" fill="none" aria-hidden="true">
  {#if direction === "left"}
    <path d="m15 18-6-6 6-6" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" />
  {:else}
    <path d="m9 18 6-6-6-6" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" />
  {/if}
</svg>
