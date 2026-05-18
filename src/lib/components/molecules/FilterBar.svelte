<script lang="ts">
  import type { Character } from "$lib/api";
  import CharacterCombobox from "$lib/components/molecules/CharacterCombobox.svelte";
  import ModTypeCombobox from "$lib/components/molecules/ModTypeCombobox.svelte";
  import SortCombobox from "$lib/components/molecules/SortCombobox.svelte";

  export let characters: Character[] = [];
  export let query = "";
  export let contentKind: "mod" | "plugin" = "mod";
  export let character = "";
  export let modType = "";
  export let sort = "recent";
  export let showAlreadyInstalled = false;
  export let canFilterInstalled = false;
  export let onChange: () => void;
  export let onContentKindChange: () => void = onChange;
  export let onInstalledVisibilityChange: () => void = () => {};

  $: pluginMode = contentKind === "plugin";

  function selectContentKind(next: "mod" | "plugin") {
    if (contentKind === next) return;
    contentKind = next;
    character = "";
    modType = "";
    onContentKindChange();
  }

  function updateModType(value: string) {
    modType = value;
    onChange();
  }

  function updateCharacter(value: string) {
    character = value;
    onChange();
  }

  function updateSort(value: string) {
    sort = value;
    onChange();
  }

  function updateShowAlreadyInstalled() {
    showAlreadyInstalled = !showAlreadyInstalled;
    onInstalledVisibilityChange();
  }
</script>

<aside class="relative z-30 grid content-start gap-4 rounded-lg border border-white/10 bg-card/86 p-4 shadow-[0_18px_50px_rgba(0,0,0,0.22)] backdrop-blur-md lg:sticky lg:top-24">
  <div>
    <p class="text-xs font-black uppercase tracking-[0.18em] text-muted-foreground">Browse</p>
    <div class="mt-3 grid grid-cols-2 gap-1 rounded-md border border-white/10 bg-background/45 p-1">
      <button class="h-10 rounded px-3 text-sm font-black {contentKind === 'mod' ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:bg-white/10 hover:text-foreground'}" type="button" on:click={() => selectContentKind("mod")}>Mods</button>
      <button class="h-10 rounded px-3 text-sm font-black {contentKind === 'plugin' ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:bg-white/10 hover:text-foreground'}" type="button" on:click={() => selectContentKind("plugin")}>Plugins</button>
    </div>
  </div>

  <label class="grid gap-2 text-sm font-bold">
    Search
    <span class="input input-bordered flex items-center gap-2 bg-background/60">
      <span class="font-black text-primary">⌕</span>
      <input bind:value={query} on:input={onChange} placeholder="Name, creator, tag..." />
    </span>
  </label>

  {#if !pluginMode}
    <div class="grid gap-2">
      <p class="text-sm font-bold">Mod type</p>
      <ModTypeCombobox value={modType} onChange={updateModType} />
    </div>

    <div class="grid gap-2">
      <p class="text-sm font-bold">Character</p>
      <CharacterCombobox {characters} value={character} placeholder="All characters" valueKey="slug" includeAll={true} onChange={updateCharacter} />
    </div>
  {/if}

  <div class="grid gap-2">
    <p class="text-sm font-bold">Sort</p>
    <SortCombobox value={sort} onChange={updateSort} />
  </div>

  {#if canFilterInstalled}
    <label class="flex min-h-12 items-center gap-2 rounded-md border border-input bg-background/60 px-3 py-2 text-sm font-bold text-foreground">
      <input class="h-4 w-4 accent-primary" type="checkbox" checked={showAlreadyInstalled} on:change={updateShowAlreadyInstalled} />
      <span>Show already installed</span>
    </label>
  {/if}
</aside>
