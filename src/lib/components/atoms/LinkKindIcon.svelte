<script lang="ts">
  export let kind = "external";
  export let className = "h-4 w-4";
  export { className as class };

  $: normalizedKind = kind?.toLowerCase() ?? "external";
</script>

{#if normalizedKind === "patreon"}
  <svg class={className} viewBox="0 0 24 24" fill="none" aria-hidden="true">
    <path d="M6 4v16" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
    <circle cx="15" cy="9" r="5" fill="currentColor" />
  </svg>
{:else if normalizedKind === "kofi"}
  <svg class={className} viewBox="0 0 24 24" fill="none" aria-hidden="true">
    <path d="M5 8h11v6a5 5 0 0 1-5 5H9a4 4 0 0 1-4-4V8Z" stroke="currentColor" stroke-width="2" stroke-linejoin="round" />
    <path d="M16 10h2.5a2.5 2.5 0 0 1 0 5H16" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
    <path d="M9 11.5c.9-1.1 2.2-1 3 0 .8-1 2.1-1.1 3 0" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
  </svg>
{:else if normalizedKind === "drive"}
  <svg class={className} viewBox="0 0 24 24" fill="none" aria-hidden="true">
    <path d="M7 18h10a4 4 0 0 0 .8-7.9A6 6 0 0 0 6.4 8.2 5 5 0 0 0 7 18Z" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
  </svg>
{:else if normalizedKind === "nexus"}
  <svg class={className} viewBox="0 0 24 24" fill="none" aria-hidden="true">
    <path d="M12 3 20 7.5v9L12 21l-8-4.5v-9L12 3Z" stroke="currentColor" stroke-width="2" stroke-linejoin="round" />
    <path d="M9 15V9l6 6V9" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
  </svg>
{:else if normalizedKind === "mirror"}
  <svg class={className} viewBox="0 0 24 24" fill="none" aria-hidden="true">
    <path d="M8 7h8M8 12h8M8 17h5" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
    <rect x="5" y="3" width="14" height="18" rx="3" stroke="currentColor" stroke-width="2" />
  </svg>
{:else if normalizedKind === "file" || normalizedKind === "zip"}
  <svg class={className} viewBox="0 0 24 24" fill="none" aria-hidden="true">
    <path d="M12 4v10m0 0 4-4m-4 4-4-4" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
    <path d="M5 18h14" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
  </svg>
{:else}
  <svg class={className} viewBox="0 0 24 24" fill="none" aria-hidden="true">
    <path d="M10 13a5 5 0 0 0 7.1 0l2-2a5 5 0 0 0-7.1-7.1l-1.1 1.1" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
    <path d="M14 11a5 5 0 0 0-7.1 0l-2 2A5 5 0 0 0 12 20.1l1.1-1.1" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
  </svg>
{/if}
