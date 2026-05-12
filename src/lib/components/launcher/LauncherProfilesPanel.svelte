<script lang="ts">
  import Button from "$lib/components/ui/Button.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import LauncherProfileCard from "$lib/components/molecules/LauncherProfileCard.svelte";
  import { profileModCount, profilePreviewMods } from "./helpers";
  import type { InstalledMod, ModProfile } from "./types";

  export let profiles: ModProfile[] = [];
  export let installedMods: InstalledMod[] = [];
  export let profileName = "";
  export let busy = false;
  export let onCreate: () => void = () => {};
  export let onSaveEnabled: () => void = () => {};
  export let onOpen: (profile: ModProfile) => void = () => {};
  export let onApply: (profile: ModProfile) => void = () => {};
  export let onDelete: (profile: ModProfile) => void = () => {};
</script>

<div class="grid gap-5 p-2 pt-5">
  <div class="flex flex-col gap-3 lg:flex-row lg:items-end lg:justify-between">
    <div>
      <h2 class="text-xl font-black">Profiles</h2>
      <p class="mt-2 text-sm leading-6 text-muted-foreground">Build presets for enabled mods, then switch setups before launching the game.</p>
    </div>
    <div class="flex flex-wrap gap-2">
      <Input class="w-56" bind:value={profileName} placeholder="Profile name" />
      <Button disabled={!profileName.trim() || busy} on:click={onCreate}>Create</Button>
      <Button variant="outline" disabled={!profileName.trim() || !installedMods.length || busy} on:click={onSaveEnabled}>Save enabled mods</Button>
    </div>
  </div>

  {#if profiles.length}
    <section class="grid gap-5 md:grid-cols-2 xl:grid-cols-3">
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
