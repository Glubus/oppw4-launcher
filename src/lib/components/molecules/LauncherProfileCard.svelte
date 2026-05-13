<script lang="ts">
  import ChevronIcon from "$lib/components/atoms/ChevronIcon.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import { profileColor, profileIcon } from "$lib/components/launcher/helpers";
  import ProfileIcon from "$lib/components/launcher/ProfileIcon.svelte";
  import type { ModProfile } from "$lib/components/launcher/types";

  type InstalledMod = {
    name: string;
    coverDataUrl?: string | null;
  };

  export let profile: ModProfile;
  export let previewMods: InstalledMod[] = [];
  export let availableCount = 0;
  export let busy = false;
  export let onOpen: (profile: ModProfile) => void = () => {};
  export let onApply: (profile: ModProfile) => void = () => {};
  export let onDelete: (profile: ModProfile) => void = () => {};

  let activeImage = 0;

  $: images = previewMods.filter((mod) => mod.coverDataUrl);
  $: if (activeImage >= images.length) activeImage = 0;
  $: preview = images[activeImage];
  $: icon = profileIcon(profile);
  $: color = profileColor(profile);

  function previousImage() {
    if (images.length < 2) return;
    activeImage = activeImage === 0 ? images.length - 1 : activeImage - 1;
  }

  function nextImage() {
    if (images.length < 2) return;
    activeImage = activeImage === images.length - 1 ? 0 : activeImage + 1;
  }
</script>

<article class="group overflow-hidden rounded-lg border border-white/10 bg-card/92 shadow-[0_18px_55px_rgba(0,0,0,0.34)] backdrop-blur-md transition duration-200 hover:-translate-y-0.5 hover:border-white/30">
  <div class="relative aspect-[16/11] overflow-hidden bg-muted">
    {#if preview?.coverDataUrl}
      <img class="h-full w-full object-cover transition duration-300 group-hover:scale-[1.035]" src={preview.coverDataUrl} alt={preview.name} />
    {:else}
      <div class="absolute inset-0" style={`background: linear-gradient(135deg, ${color.from}, ${color.to});`}></div>
    {/if}

    <div class="absolute inset-x-0 bottom-0 h-28 bg-gradient-to-t from-background/88 to-transparent"></div>
    <button class="absolute inset-0 z-10 cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring" type="button" on:click={() => onOpen(profile)} aria-label={`Open ${profile.name}`}></button>
    <div class="absolute right-3 top-3 z-20 grid h-11 w-11 place-items-center rounded-md border shadow-xl backdrop-blur" style={`background: linear-gradient(135deg, ${color.from}, ${color.to}); border-color: ${color.border}; color: ${color.text};`}>
      <ProfileIcon name={icon.value} className="h-5 w-5" />
    </div>
    <div class="absolute left-3 top-3 z-20 flex flex-wrap gap-2">
      <span class="rounded-full border border-white/25 bg-black/45 px-2.5 py-1 text-[0.68rem] font-black uppercase tracking-wide text-white backdrop-blur">Profile</span>
      <span class="rounded-full border border-white/25 bg-black/45 px-2.5 py-1 text-[0.68rem] font-black uppercase tracking-wide text-white backdrop-blur">{availableCount}/{profile.enabledModKeys.length} available</span>
      {#if images.length > 1}
        <span class="rounded-full border border-white/25 bg-black/45 px-2.5 py-1 text-[0.68rem] font-black text-white backdrop-blur">{activeImage + 1}/{images.length}</span>
      {/if}
    </div>

    {#if images.length > 1}
      <div class="absolute inset-x-3 top-1/2 z-20 flex -translate-y-1/2 items-center justify-between opacity-0 transition group-hover:opacity-100">
        <button class="grid h-9 w-9 place-items-center rounded-full border border-white/25 bg-black/50 text-white backdrop-blur hover:bg-black/70" type="button" on:click={previousImage} aria-label="Previous image">
          <ChevronIcon direction="left" class="h-5 w-5" />
        </button>
        <button class="grid h-9 w-9 place-items-center rounded-full border border-white/25 bg-black/50 text-white backdrop-blur hover:bg-black/70" type="button" on:click={nextImage} aria-label="Next image">
          <ChevronIcon direction="right" class="h-5 w-5" />
        </button>
      </div>
    {/if}
  </div>

  <div class="grid gap-4 p-4">
    <div class="min-w-0">
      <p class="truncate text-xs font-black uppercase tracking-[0.18em] text-muted-foreground">Preset / {profile.enabledModKeys.length} linked mod{profile.enabledModKeys.length === 1 ? "" : "s"}</p>
      <button class="mt-1 block text-left focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring" type="button" on:click={() => onOpen(profile)}>
        <h3 class="line-clamp-2 text-2xl font-black leading-tight text-foreground">{profile.name}</h3>
      </button>
    </div>

    <p class="line-clamp-2 min-h-11 text-sm leading-5 text-muted-foreground">
      Opens with {availableCount} locally available mod{availableCount === 1 ? "" : "s"}.
    </p>

    <div class="grid grid-cols-2 gap-2">
      <Button disabled={busy} on:click={() => onApply(profile)}>Apply</Button>
      <Button variant="outline" disabled={busy} on:click={() => onDelete(profile)}>Delete</Button>
    </div>
  </div>
</article>
