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

  let showCreator = false;
</script>

<div class="grid gap-5 p-2 pt-5">
  <div class="flex flex-col gap-3 lg:flex-row lg:items-end lg:justify-between">
    <div>
      <h2 class="text-xl font-black">Profiles</h2>
      <p class="mt-2 text-sm leading-6 text-muted-foreground">Build presets for enabled mods, then switch setups before launching the game.</p>
    </div>
    <Button disabled={busy} on:click={() => (showCreator = true)}>Add new profile</Button>
  </div>

  {#if profiles.length}
    <section class="grid w-full gap-5 md:grid-cols-2 xl:grid-cols-3">
      {#each profiles as profile}
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
