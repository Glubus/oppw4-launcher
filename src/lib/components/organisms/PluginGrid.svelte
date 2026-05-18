<script lang="ts">
  import type { Plugin } from "$lib/api";
  import PluginCard from "$lib/components/molecules/PluginCard.svelte";

  export let plugins: Plugin[] = [];
  export let loading = false;
</script>

{#if loading}
  <section class="grid gap-5 md:grid-cols-2 xl:grid-cols-3">
    {#each Array(6) as _}
      <div class="skeleton h-[300px] rounded-lg"></div>
    {/each}
  </section>
{:else if plugins.length}
  <section class="grid gap-5 md:grid-cols-2 xl:grid-cols-3">
    {#each plugins as plugin}
      <PluginCard {plugin} />
    {/each}
  </section>
{:else}
  <section class="rounded-lg border border-dashed border-white/18 bg-card/72 p-12 text-center backdrop-blur-md">
    <p class="text-2xl font-black">No plugins found</p>
    <p class="mt-2 text-base-content/60">Try another plugin name, tag, or creator.</p>
  </section>
{/if}
