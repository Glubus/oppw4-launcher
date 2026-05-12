<script lang="ts">
  import type { Character } from "$lib/api";
  import CharacterCombobox from "$lib/components/molecules/CharacterCombobox.svelte";
  import ModTypeCombobox from "$lib/components/molecules/ModTypeCombobox.svelte";
  import SortCombobox from "$lib/components/molecules/SortCombobox.svelte";

  export let characters: Character[] = [];
  export let query = "";
  export let character = "";
  export let modType = "";
  export let sort = "recent";
  export let onChange: () => void;
</script>

<section class="relative z-30 grid gap-3 overflow-visible rounded-lg border border-white/10 bg-card/86 p-3 shadow-[0_18px_50px_rgba(0,0,0,0.22)] backdrop-blur-md lg:grid-cols-[1fr_260px_210px_180px]">
  <label class="input input-bordered flex items-center gap-2 bg-background/60">
    <span class="font-black text-primary">⌕</span>
    <input bind:value={query} on:input={onChange} placeholder="Search skin, creator, tag..." />
  </label>

  <ModTypeCombobox bind:value={modType} {onChange} />

  <CharacterCombobox {characters} bind:value={character} placeholder="All characters" valueKey="slug" includeAll={true} {onChange} />

  <SortCombobox bind:value={sort} {onChange} />
</section>
