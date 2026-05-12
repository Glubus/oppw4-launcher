<script lang="ts">
  import type { Skin } from "$lib/api";
  import SkinCard from "$lib/components/molecules/SkinCard.svelte";

  export let skins: Skin[] = [];
  export let loading = false;
  export let onVote: (skin: Skin) => void;
  export let canManagePins = false;
  export let onTogglePin: (skin: Skin) => void = () => {};
</script>

{#if loading}
  <section class="grid gap-5 md:grid-cols-2 xl:grid-cols-3">
    {#each Array(6) as _}
      <div class="skeleton h-[430px] rounded-lg"></div>
    {/each}
  </section>
{:else if skins.length}
  <section class="grid gap-5 md:grid-cols-2 xl:grid-cols-3">
    {#each skins as skin}
      <SkinCard {skin} {onVote} {canManagePins} {onTogglePin} />
    {/each}
  </section>
{:else}
  <section class="rounded-lg border border-dashed border-white/18 bg-card/72 p-12 text-center backdrop-blur-md">
    <p class="text-2xl font-black">No skins found</p>
    <p class="mt-2 text-base-content/60">Try another character, tag, or creator name.</p>
  </section>
{/if}
