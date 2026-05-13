<script lang="ts">
  import Button from "$lib/components/ui/Button.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import LauncherProfileCard from "$lib/components/molecules/LauncherProfileCard.svelte";
  import ProfileIcon from "./ProfileIcon.svelte";
  import { profileColors, profileIcons, profileModCount, profilePreviewMods } from "./helpers";
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

  let newProfileIcon = "sparkles";
  let newProfileColor = "violet";
</script>

<div class="grid gap-5 p-2 pt-5">
  <div class="flex flex-col gap-3 lg:flex-row lg:items-end lg:justify-between">
    <div>
      <h2 class="text-xl font-black">Profiles</h2>
      <p class="mt-2 text-sm leading-6 text-muted-foreground">Build presets for enabled mods, then switch setups before launching the game.</p>
    </div>
    <div class="flex flex-wrap items-end gap-2">
      <Input class="w-56" bind:value={profileName} placeholder="Profile name" />
      <div class="flex gap-1 rounded-lg border border-white/10 bg-background/45 p-1">
        {#each profileIcons as icon}
          <button class="grid h-9 w-9 place-items-center rounded-md border border-transparent text-muted-foreground hover:bg-white/10 hover:text-foreground {newProfileIcon === icon.value ? 'bg-white/10 text-foreground' : ''}" type="button" title={icon.label} on:click={() => (newProfileIcon = icon.value)}>
            <ProfileIcon name={icon.value} className="h-4 w-4" />
          </button>
        {/each}
      </div>
      <div class="flex gap-1 rounded-lg border border-white/10 bg-background/45 p-1">
        {#each profileColors as color}
          <button class="h-9 w-9 rounded-md border-2" style={`background: linear-gradient(135deg, ${color.from}, ${color.to}); border-color: ${newProfileColor === color.value ? color.text : color.border};`} type="button" title={color.label} on:click={() => (newProfileColor = color.value)}></button>
        {/each}
      </div>
      <Button disabled={!profileName.trim() || busy} on:click={() => onCreateWithStyle(newProfileIcon, newProfileColor)}>Create</Button>
      <Button variant="outline" disabled={!profileName.trim() || !installedMods.length || busy} on:click={() => onSaveEnabledWithStyle(newProfileIcon, newProfileColor)}>Save enabled mods</Button>
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
