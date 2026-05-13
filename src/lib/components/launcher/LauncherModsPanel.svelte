<script lang="ts">
  import CharacterCombobox from "$lib/components/molecules/CharacterCombobox.svelte";
  import ModTypeCombobox from "$lib/components/molecules/ModTypeCombobox.svelte";
  import SortCombobox from "$lib/components/molecules/SortCombobox.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import InstalledModCard from "./InstalledModCard.svelte";
  import { localCharacters, sortInstalledMods } from "./helpers";
  import type { InstalledMod, ModProfile, UpdateSkinMap } from "./types";

  export let installedMods: InstalledMod[] = [];
  export let profiles: ModProfile[] = [];
  export let updateSkins: UpdateSkinMap = {};
  export let hasGameFolder = false;
  export let busy = false;
  export let checkingUpdates = false;
  export let updatingAll = false;
  export let updateCount = 0;
  export let onImportZip: () => void = () => {};
  export let onUpdateAll: () => void = () => {};
  export let onToggleMod: (mod: InstalledMod) => void = () => {};
  export let onRemoveMod: (mod: InstalledMod) => void = () => {};
  export let onAddToProfile: (profile: ModProfile, mod: InstalledMod) => void = () => {};

  const statusOptions = [
    { value: "", label: "All status" },
    { value: "enabled", label: "Enabled" },
    { value: "disabled", label: "Disabled" },
    { value: "to_update", label: "To update" }
  ];

  let modSearch = "";
  let modCharacter = "";
  let modType = "";
  let modStatus = "";
  let modSort = "recent";
  let statusDetails: HTMLDetailsElement;

  $: installedCharacters = localCharacters(installedMods);
  $: filteredInstalledMods = sortInstalledMods(installedMods.filter(matchesModFilters), modSort);

  function matchesModFilters(mod: InstalledMod) {
    const query = modSearch.trim().toLowerCase();
    const matchesQuery = !query || [mod.name, mod.version, mod.characterName, mod.characterSlug, mod.modType, mod.slug].some((part) => part?.toLowerCase().includes(query));
    const matchesCharacter = !modCharacter || mod.characterSlug === modCharacter;
    const matchesType = !modType || mod.modType === modType;
    const matchesStatus = !modStatus || (modStatus === "enabled" ? mod.enabled : modStatus === "disabled" ? !mod.enabled : Boolean(updateSkins[mod.path]));
    return matchesQuery && matchesCharacter && matchesType && matchesStatus;
  }

  function resetInstalledFilters() {
    modSearch = "";
    modCharacter = "";
    modType = "";
    modStatus = "";
    modSort = "recent";
  }

  function selectStatus(next: string) {
    modStatus = next;
    if (statusDetails) statusDetails.open = false;
  }

  function selectModType(value: string) {
    modType = value;
  }

  function selectCharacter(value: string) {
    modCharacter = value;
  }

  function selectSort(value: string) {
    modSort = value;
  }
</script>

<div class="p-2 pt-5">
  <div class="mb-5 flex items-center justify-between gap-3">
    <div>
      <h2 class="text-xl font-black">Installed mods</h2>
      <p class="mt-1 text-sm text-muted-foreground">{filteredInstalledMods.length}/{installedMods.length} found in your mods folder.</p>
    </div>
    <div class="flex flex-wrap gap-2">
      <Button variant="outline" size="sm" disabled={busy || !hasGameFolder} on:click={onImportZip}>Import ZIP</Button>
      {#if updateCount}
        <Button size="sm" disabled={updatingAll} on:click={onUpdateAll}>{updatingAll ? "Updating..." : `Update all (${updateCount})`}</Button>
      {/if}
      {#if checkingUpdates}
        <span class="inline-flex h-9 items-center rounded-md border border-white/12 bg-background/55 px-3 text-xs font-black uppercase tracking-wide text-muted-foreground">Checking updates...</span>
      {/if}
    </div>
  </div>

  <section class="relative z-30 grid gap-3 overflow-visible rounded-lg border border-white/10 bg-card/86 p-3 shadow-[0_18px_50px_rgba(0,0,0,0.22)] backdrop-blur-md lg:grid-cols-[1fr_210px_240px_180px_160px_auto]">
    <label class="input input-bordered flex items-center gap-2 bg-background/60">
      <span class="font-black text-primary">⌕</span>
      <input bind:value={modSearch} placeholder="Search mod, character, version..." />
    </label>
    <ModTypeCombobox value={modType} onChange={selectModType} />
    <CharacterCombobox characters={installedCharacters} value={modCharacter} placeholder="All characters" valueKey="slug" includeAll={true} onChange={selectCharacter} />

    <details class="relative z-40 w-full" bind:this={statusDetails}>
      <summary class="flex h-10 w-full cursor-pointer list-none items-center justify-between rounded-md border border-white/12 bg-background/55 px-3 text-sm font-medium text-foreground shadow-sm outline-none transition-colors hover:bg-white/10 focus-visible:ring-2 focus-visible:ring-ring">
        <span class="truncate">{statusOptions.find((item) => item.value === modStatus)?.label ?? "All status"}</span>
        <span class="text-muted-foreground">⌄</span>
      </summary>
      <div class="absolute z-50 mt-2 w-full rounded-lg border border-white/12 bg-popover/95 p-2 text-popover-foreground shadow-2xl backdrop-blur-md">
        {#each statusOptions as item}
          <button class="flex h-8 w-full items-center justify-start rounded-md px-2 text-sm hover:bg-white/10" class:bg-accent={item.value === modStatus} class:text-accent-foreground={item.value === modStatus} type="button" on:click={() => selectStatus(item.value)}>
            {item.label}
          </button>
        {/each}
      </div>
    </details>

    <SortCombobox value={modSort} onChange={selectSort} />
    <Button variant="outline" type="button" on:click={resetInstalledFilters}>Reset</Button>
  </section>

  {#if !hasGameFolder}
    <p class="rounded-lg border border-white/12 bg-background/45 p-4 text-sm text-muted-foreground">Select a game folder in Settings to scan installed mods.</p>
  {:else if filteredInstalledMods.length}
    <section class="mt-5 grid gap-5 md:grid-cols-2 xl:grid-cols-3">
      {#each filteredInstalledMods as mod}
        <InstalledModCard {mod} {profiles} {updateSkins} {busy} onToggle={onToggleMod} onRemove={onRemoveMod} onAddToProfile={onAddToProfile} />
      {/each}
    </section>
  {:else}
    <p class="mt-5 rounded-lg border border-white/12 bg-background/45 p-4 text-sm text-muted-foreground">{installedMods.length ? "No installed mods match this search." : "No installed mods found. Create a mods/ folder next to the game executable and add mod folders or zip files."}</p>
  {/if}
</div>
