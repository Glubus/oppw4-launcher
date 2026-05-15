<script lang="ts">
  import Button from "$lib/components/ui/Button.svelte";
  import LauncherProfileCard from "$lib/components/molecules/LauncherProfileCard.svelte";
  import ProfileCreateModal from "./ProfileCreateModal.svelte";
  import { profileModCount, profilePreviewMods } from "./helpers";
  import type { InstalledMod, ModProfile } from "./types";

  export let profiles: ModProfile[] = [];
  export let installedMods: InstalledMod[] = [];
  export let profileName = "";
  export let busy = false;
  export let onOpen: (profile: ModProfile) => void = () => {};
  export let onApply: (profile: ModProfile) => void = () => {};
  export let onDelete: (profile: ModProfile) => void = () => {};
  export let onCreateWithStyle: (icon: string, color: string) => void = () => {};
  export let onSaveEnabledWithStyle: (icon: string, color: string) => void = () => {};

  const availabilityOptions = [
    { value: "", label: "All profiles" },
    { value: "available", label: "Fully available" },
    { value: "partial", label: "Missing mods" },
    { value: "empty", label: "Empty" }
  ];

  let showCreator = false;
  let profileSearch = "";
  let profileAvailability = "";
  let availabilityDetails: HTMLDetailsElement;

  $: filteredProfiles = profiles.filter(matchesProfileFilters);

  function matchesProfileFilters(profile: ModProfile) {
    const query = profileSearch.trim().toLowerCase();
    const availableCount = profileModCount(profile, installedMods);
    const totalCount = profile.enabledModKeys.length;
    const matchesQuery = !query || profile.name.toLowerCase().includes(query);
    const matchesAvailability =
      !profileAvailability ||
      (profileAvailability === "available" && totalCount > 0 && availableCount === totalCount) ||
      (profileAvailability === "partial" && totalCount > 0 && availableCount < totalCount) ||
      (profileAvailability === "empty" && totalCount === 0);
    return matchesQuery && matchesAvailability;
  }

  function selectAvailability(next: string) {
    profileAvailability = next;
    if (availabilityDetails) availabilityDetails.open = false;
  }

  function resetProfileFilters() {
    profileSearch = "";
    profileAvailability = "";
  }
</script>

<div class="grid gap-5 p-2 pt-5">
  <div class="flex flex-col gap-3 lg:flex-row lg:items-end lg:justify-between">
    <div>
      <h2 class="text-xl font-black">Profiles</h2>
      <p class="mt-1 text-sm text-muted-foreground">Build presets for enabled mods, then switch setups before launching the game.</p>
    </div>
    <Button disabled={busy} on:click={() => (showCreator = true)}>Add new profile</Button>
  </div>

  <section class="relative z-30 grid min-w-0 gap-3 overflow-visible border-b border-white/10 pb-5 sm:grid-cols-[minmax(0,1fr)_minmax(0,190px)_minmax(0,90px)]">
    <label class="input input-bordered flex min-w-0 items-center gap-2 bg-background/60">
      <span class="font-black text-primary">⌕</span>
      <input class="min-w-0" bind:value={profileSearch} placeholder="Search profile..." />
    </label>

    <details class="relative z-40 min-w-0 w-full" bind:this={availabilityDetails}>
      <summary class="flex h-10 w-full cursor-pointer list-none items-center justify-between rounded-md border border-white/12 bg-background/55 px-3 text-sm font-medium text-foreground shadow-sm outline-none transition-colors hover:bg-white/10 focus-visible:ring-2 focus-visible:ring-ring">
        <span class="truncate">{availabilityOptions.find((item) => item.value === profileAvailability)?.label ?? "All profiles"}</span>
        <span class="text-muted-foreground">⌄</span>
      </summary>
      <div class="absolute z-50 mt-2 w-full rounded-lg border border-white/12 bg-popover/95 p-2 text-popover-foreground shadow-2xl backdrop-blur-md">
        {#each availabilityOptions as item}
          <button class="flex h-8 w-full items-center justify-start rounded-md px-2 text-sm hover:bg-white/10" class:bg-accent={item.value === profileAvailability} class:text-accent-foreground={item.value === profileAvailability} type="button" on:click={() => selectAvailability(item.value)}>
            {item.label}
          </button>
        {/each}
      </div>
    </details>

    <Button class="w-full" variant="outline" type="button" on:click={resetProfileFilters}>Reset</Button>
  </section>

  {#if profiles.length}
    <section class="grid w-full gap-5 md:grid-cols-2 xl:grid-cols-3">
      {#each filteredProfiles as profile}
        <LauncherProfileCard
          {profile}
          previewMods={profilePreviewMods(profile, installedMods)}
          availableCount={profileModCount(profile, installedMods)}
          {busy}
          {onOpen}
          onApply={onApply}
          onDelete={onDelete}
        />
      {/each}
    </section>
    {#if !filteredProfiles.length}
      <p class="rounded-lg border border-white/12 bg-background/45 p-4 text-sm text-muted-foreground">No profiles match this search.</p>
    {/if}
  {:else}
    <p class="rounded-lg border border-white/12 bg-background/45 p-4 text-sm text-muted-foreground">No profiles yet. Create one here, then add mods from the Mods tab.</p>
  {/if}
</div>

{#if showCreator}
  <ProfileCreateModal
    bind:profileName
    installedCount={installedMods.length}
    {busy}
    onClose={() => (showCreator = false)}
    onCreate={onCreateWithStyle}
    onSaveEnabled={onSaveEnabledWithStyle}
  />
{/if}
