<script lang="ts">
  import type { Character } from "$lib/api";
  import CharacterCombobox from "$lib/components/molecules/CharacterCombobox.svelte";
  import ModTypeCombobox from "$lib/components/molecules/ModTypeCombobox.svelte";
  import SortCombobox from "$lib/components/molecules/SortCombobox.svelte";

  export let characters: Character[] = [];
  export let query = "";
  export let pluginMode = false;
  export let character = "";
  export let modType = "";
  export let sort = "recent";
  export let showAlreadyInstalled = false;
  export let canFilterInstalled = false;
  export let onChange: () => void;
  export let onInstalledVisibilityChange: () => void = () => {};

  function updateShowAlreadyInstalled() {
    showAlreadyInstalled = !showAlreadyInstalled;
    onInstalledVisibilityChange();
  }
</script>

<section class="relative z-30 grid gap-3 overflow-visible rounded-lg border border-white/10 bg-card/86 p-3 shadow-[0_18px_50px_rgba(0,0,0,0.22)] backdrop-blur-md {pluginMode ? 'lg:grid-cols-[1fr_180px]' : canFilterInstalled ? 'lg:grid-cols-[1fr_240px_190px_160px_210px]' : 'lg:grid-cols-[1fr_260px_210px_180px]'}">
  <label class="input input-bordered flex items-center gap-2 bg-background/60">
    <span class="font-black text-primary">⌕</span>
    <input bind:value={query} on:input={onChange} placeholder="Search skin, creator, tag..." />
  </label>

  {#if !pluginMode}
    <ModTypeCombobox bind:value={modType} {onChange} />

    <CharacterCombobox {characters} bind:value={character} placeholder="All characters" valueKey="slug" includeAll={true} {onChange} />
  {/if}

  <SortCombobox bind:value={sort} {onChange} />

  {#if canFilterInstalled && !pluginMode}
    <label class="flex h-10 items-center gap-2 rounded-md border border-input bg-background/60 px-3 text-sm font-bold text-foreground">
      <input class="h-4 w-4 accent-primary" type="checkbox" checked={showAlreadyInstalled} on:change={updateShowAlreadyInstalled} />
      <span class="truncate">Show already installed</span>
    </label>
  {/if}
</section>
