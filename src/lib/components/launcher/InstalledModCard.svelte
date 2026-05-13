<script lang="ts">
  import Button from "$lib/components/ui/Button.svelte";
  import { modInitials, modPageHref, profileHasMod } from "./helpers";
  import type { InstalledMod, ModProfile, UpdateSkinMap } from "./types";

  export let mod: InstalledMod;
  export let profiles: ModProfile[] = [];
  export let updateSkins: UpdateSkinMap = {};
  export let busy = false;
  export let onToggle: (mod: InstalledMod) => void = () => {};
  export let onRemove: (mod: InstalledMod) => void = () => {};
  export let onAddToProfile: (profile: ModProfile, mod: InstalledMod) => void = () => {};

  $: href = modPageHref(mod);
  $: updateSkin = updateSkins[mod.path];
</script>

<article class="group overflow-hidden rounded-lg border border-white/10 bg-card/92 shadow-[0_18px_55px_rgba(0,0,0,0.34)] backdrop-blur-md transition duration-200 hover:-translate-y-0.5 hover:border-white/30 {!mod.enabled ? 'grayscale opacity-60' : ''}">
  <div class="relative aspect-[16/11] overflow-hidden bg-muted">
    {#if mod.coverDataUrl}
      <img class="h-full w-full object-cover transition duration-300 group-hover:scale-[1.035] {!mod.enabled ? 'brightness-75' : ''}" src={mod.coverDataUrl} alt={mod.name} />
    {:else}
      <div class="absolute inset-0 bg-[linear-gradient(135deg,hsl(var(--primary)/.22),hsl(var(--accent)/.18))]"></div>
      <div class="absolute left-5 top-5 rounded-md border border-white/30 bg-white/12 px-4 py-3 text-4xl font-black text-white shadow-xl backdrop-blur">
        {modInitials(mod.name)}
      </div>
    {/if}
    <div class="absolute inset-x-0 bottom-0 h-28 bg-gradient-to-t from-background/88 to-transparent"></div>
    <div class="absolute left-3 top-3 z-20 flex flex-wrap gap-2">
      <span class="rounded-full border border-white/25 bg-black/45 px-2.5 py-1 text-[0.68rem] font-black uppercase tracking-wide text-white backdrop-blur">Installed</span>
      {#if updateSkin}
        <span class="rounded-full border border-amber-300/50 bg-amber-400/20 px-2.5 py-1 text-[0.68rem] font-black uppercase tracking-wide text-amber-100 backdrop-blur">To update</span>
      {/if}
      <span class="rounded-full border border-white/25 bg-black/45 px-2.5 py-1 text-[0.68rem] font-black uppercase tracking-wide text-white backdrop-blur">{mod.kind}</span>
      <span class="rounded-full border border-white/25 bg-black/45 px-2.5 py-1 text-[0.68rem] font-black uppercase tracking-wide text-white backdrop-blur">{mod.enabled ? "Enabled" : "Disabled"}</span>
    </div>
    {#if href}
      <a class="absolute inset-0 z-10 cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring" href={href} aria-label={`Open ${mod.name}`}></a>
    {/if}
  </div>

  <div class="grid gap-4 p-4">
    <div class="min-w-0">
      <p class="truncate text-xs font-black uppercase tracking-[0.18em] text-muted-foreground">{mod.characterName || "Local mod"} / {mod.modType || mod.kind}</p>
      {#if href}
        <a class="mt-1 block focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring" href={href}>
          <h2 class="line-clamp-2 text-2xl font-black leading-tight text-foreground">{mod.name}</h2>
        </a>
      {:else}
        <h2 class="line-clamp-2 text-2xl font-black leading-tight text-foreground">{mod.name}</h2>
      {/if}
      {#if mod.version}
        <p class="mt-1 text-xs font-bold text-primary">v{mod.version}</p>
      {/if}
      {#if updateSkin}
        <p class="mt-1 text-xs font-bold text-amber-300">Latest v{updateSkin.version}</p>
      {/if}
      {#if mod.dependencies.length}
        <p class="mt-2 line-clamp-1 text-xs font-bold text-muted-foreground">Needs {mod.dependencies.join(", ")}</p>
      {/if}
      {#if mod.changelog}
        <details class="mt-2 rounded-md border border-white/10 bg-background/45 p-2 text-xs text-muted-foreground">
          <summary class="cursor-pointer font-black text-foreground">Changelog</summary>
          <p class="mt-1 whitespace-pre-wrap leading-5">{mod.changelog}</p>
        </details>
      {/if}
    </div>

    <div class="grid grid-cols-2 gap-2">
      {#if profiles.length}
        <details class="relative z-30">
          <summary class="flex h-10 w-full cursor-pointer list-none items-center justify-center rounded-md border border-white/12 bg-background/55 px-3 text-sm font-bold text-foreground shadow-sm transition-colors hover:bg-white/10">
            Add to profile
          </summary>
          <div class="absolute bottom-12 left-0 z-50 grid min-w-44 gap-1 rounded-lg border border-white/12 bg-popover/95 p-2 text-popover-foreground shadow-2xl backdrop-blur-md">
            {#each profiles as profile}
              <button class="flex h-8 w-full items-center justify-between gap-3 rounded-md px-2 text-left text-sm hover:bg-white/10 disabled:cursor-not-allowed disabled:opacity-55" type="button" disabled={profileHasMod(profile, mod) || busy} on:click={() => onAddToProfile(profile, mod)}>
                <span class="truncate">{profile.name}</span>
                {#if profileHasMod(profile, mod)}
                  <span class="text-xs font-black text-primary">Added</span>
                {/if}
              </button>
            {/each}
          </div>
        </details>
      {:else}
        <Button variant="outline" disabled={true}>No profile</Button>
      {/if}
      <Button variant={mod.enabled ? "destructive" : "default"} disabled={busy} on:click={() => onToggle(mod)}>
        {mod.enabled ? "Disable" : "Enable"}
      </Button>
      <Button class="col-span-2" variant="outline" disabled={busy} on:click={() => onRemove(mod)}>Remove</Button>
    </div>
  </div>
</article>
