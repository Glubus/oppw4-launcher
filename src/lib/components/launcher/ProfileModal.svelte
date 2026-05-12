<script lang="ts">
  import Button from "$lib/components/ui/Button.svelte";
  import { modInitials, modPageHref } from "./helpers";
  import type { InstalledMod, ModProfile, UpdateSkinMap } from "./types";

  export let profile: ModProfile;
  export let mods: InstalledMod[] = [];
  export let updateSkins: UpdateSkinMap = {};
  export let busy = false;
  export let onApply: (profile: ModProfile) => void = () => {};
  export let onClose: () => void = () => {};
  export let onToggleMod: (mod: InstalledMod) => void = () => {};
</script>

<div class="fixed inset-0 z-50 grid place-items-center p-4">
  <button class="absolute inset-0 bg-black/70 backdrop-blur-sm" type="button" aria-label="Close profile" on:click={onClose}></button>
  <div class="relative max-h-[86vh] w-full max-w-6xl overflow-hidden rounded-lg border border-white/12 bg-background shadow-2xl" role="dialog" aria-modal="true" aria-label={`${profile.name} profile`}>
    <div class="flex flex-col gap-3 border-b border-white/10 p-4 sm:flex-row sm:items-center sm:justify-between">
      <div>
        <p class="text-xs font-black uppercase tracking-[0.18em] text-primary">Profile</p>
        <h2 class="text-2xl font-black">{profile.name}</h2>
        <p class="mt-1 text-sm text-muted-foreground">{mods.length}/{profile.enabledModKeys.length} linked mods available locally.</p>
      </div>
      <div class="flex flex-wrap gap-2">
        <Button disabled={busy} on:click={() => onApply(profile)}>Apply</Button>
        <Button variant="outline" on:click={onClose}>Close</Button>
      </div>
    </div>

    <div class="max-h-[calc(86vh-110px)] overflow-auto p-4">
      {#if mods.length}
        <section class="grid gap-5 md:grid-cols-2 xl:grid-cols-3">
          {#each mods as mod}
            {@const href = modPageHref(mod)}
            <article class="group overflow-hidden rounded-lg border border-white/10 bg-card/92 shadow-[0_18px_55px_rgba(0,0,0,0.28)] backdrop-blur-md transition duration-200 hover:-translate-y-0.5 hover:border-white/30 {!mod.enabled ? 'grayscale opacity-60' : ''}">
              <div class="relative aspect-[16/11] overflow-hidden bg-muted">
                {#if mod.coverDataUrl}
                  <img class="h-full w-full object-cover transition duration-300 group-hover:scale-[1.035] {!mod.enabled ? 'brightness-75' : ''}" src={mod.coverDataUrl} alt={mod.name} />
                {:else}
                  <div class="absolute inset-0 bg-[linear-gradient(135deg,hsl(var(--primary)/.22),hsl(var(--accent)/.18))]"></div>
                  <div class="absolute left-5 top-5 rounded-md border border-white/30 bg-white/12 px-4 py-3 text-4xl font-black text-white shadow-xl backdrop-blur">{modInitials(mod.name)}</div>
                {/if}
                <div class="absolute inset-x-0 bottom-0 h-28 bg-gradient-to-t from-background/88 to-transparent"></div>
                <div class="absolute left-3 top-3 z-20 flex flex-wrap gap-2">
                  <span class="rounded-full border border-white/25 bg-black/45 px-2.5 py-1 text-[0.68rem] font-black uppercase tracking-wide text-white backdrop-blur">Profile</span>
                  <span class="rounded-full border border-white/25 bg-black/45 px-2.5 py-1 text-[0.68rem] font-black uppercase tracking-wide text-white backdrop-blur">{mod.enabled ? "Enabled" : "Disabled"}</span>
                  {#if updateSkins[mod.path]}
                    <span class="rounded-full border border-amber-300/50 bg-amber-400/20 px-2.5 py-1 text-[0.68rem] font-black uppercase tracking-wide text-amber-100 backdrop-blur">To update</span>
                  {/if}
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
                      <h3 class="line-clamp-2 text-2xl font-black leading-tight text-foreground">{mod.name}</h3>
                    </a>
                  {:else}
                    <h3 class="line-clamp-2 text-2xl font-black leading-tight text-foreground">{mod.name}</h3>
                  {/if}
                  {#if mod.version}
                    <p class="mt-1 text-xs font-bold text-primary">v{mod.version}</p>
                  {/if}
                </div>

                <Button variant={mod.enabled ? "destructive" : "default"} disabled={busy} on:click={() => onToggleMod(mod)}>
                  {mod.enabled ? "Disable" : "Enable"}
                </Button>
              </div>
            </article>
          {/each}
        </section>
      {:else}
        <p class="rounded-lg border border-white/12 bg-card/70 p-4 text-sm text-muted-foreground">No linked mods from this profile are currently installed.</p>
      {/if}
    </div>
  </div>
</div>
