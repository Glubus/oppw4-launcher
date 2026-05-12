<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { API_BASE, mediaUrl, modTypeLabel, type Skin } from "$lib/api";
  import ChevronIcon from "$lib/components/atoms/ChevronIcon.svelte";
  import LinkKindIcon from "$lib/components/atoms/LinkKindIcon.svelte";
  import StatPill from "$lib/components/atoms/StatPill.svelte";
  import TagChip from "$lib/components/atoms/TagChip.svelte";
  import { toastStore } from "$lib/stores/toasts";
  import { markdownToPlainText } from "$lib/utils/markdown";

  export let skin: Skin;
  export let onVote: (skin: Skin) => void;
  export let canManagePins = false;
  export let onTogglePin: (skin: Skin) => void = () => {};

  type InstallHostedModResult = {
    alreadyUpToDate: boolean;
  };

  let activeImage = 0;
  let installing = false;

  $: isDesktop = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
  $: initials = skin.character.displayName
    .split(/\s+/)
    .slice(0, 2)
    .map((part) => part[0])
    .join("")
    .toUpperCase();
  $: images = skin.images ?? [];
  $: preview = images[activeImage];
  $: creatorName = skin.creditedUsername ?? skin.externalCreatorName ?? "uncredited";
  $: creatorHref = skin.creditedUsername
    ? `/creators/${encodeURIComponent(skin.creditedUsername)}`
    : skin.externalCreatorSlug
      ? `/creators/${encodeURIComponent(skin.externalCreatorSlug)}`
      : null;
  $: obtainHref = skin.links?.[0]
    ? `${API_BASE}/links/${skin.links[0].id}/redirect`
    : skin.files?.[0]
      ? `${API_BASE}/files/${skin.files[0].id}/download`
      : null;
  $: obtainKind = skin.links?.[0]?.kind ?? (skin.files?.[0] ? "zip" : "external");
  $: descriptionPreview = markdownToPlainText(skin.description);
  $: hostedFile = skin.files?.[0] ?? null;

  function previousImage() {
    if (images.length < 2) return;
    activeImage = activeImage === 0 ? images.length - 1 : activeImage - 1;
  }

  function nextImage() {
    if (images.length < 2) return;
    activeImage = activeImage === images.length - 1 ? 0 : activeImage + 1;
  }

  async function installHostedMod() {
    if (!hostedFile || installing) return;
    installing = true;
    try {
      const result = await invoke<InstallHostedModResult>("install_hosted_mod", { input: { fileId: hostedFile.id, fileName: hostedFile.fileName } });
      toastStore.push(result.alreadyUpToDate ? "Already up to date." : `${skin.title} installed.`, "success");
    } catch (err) {
      toastStore.push(err instanceof Error ? err.message : typeof err === "string" ? err : "Could not install mod.", "error");
    } finally {
      installing = false;
    }
  }
</script>

<article
  class="group overflow-hidden rounded-lg border border-white/10 bg-card/92 shadow-[0_18px_55px_rgba(0,0,0,0.34)] backdrop-blur-md transition duration-200 hover:-translate-y-0.5 hover:border-white/30"
>
  <div class="relative aspect-[16/11] overflow-hidden bg-muted">
    {#if preview}
      <img class="h-full w-full object-cover transition duration-300 group-hover:scale-[1.035]" src={mediaUrl(preview.url)} alt={preview.alt || skin.title} />
    {:else}
      <div class="absolute inset-0 bg-[linear-gradient(135deg,hsl(var(--primary)/.22),hsl(var(--accent)/.18))]"></div>
      <div class="absolute left-5 top-5 rounded-md border border-white/30 bg-white/12 px-4 py-3 text-4xl font-black text-white shadow-xl backdrop-blur">
        {initials}
      </div>
    {/if}
    <div class="absolute inset-x-0 bottom-0 h-28 bg-gradient-to-t from-background/88 to-transparent"></div>
    <a
      class="absolute inset-0 z-10 cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring"
      href={`/skins/${skin.slug}`}
      aria-label={`Open ${skin.title}`}
    ></a>

    <div class="absolute left-3 top-3 z-20 flex flex-wrap gap-2">
      <span class="rounded-full border border-white/25 bg-black/45 px-2.5 py-1 text-[0.68rem] font-black uppercase tracking-wide text-white backdrop-blur">{skin.character.isDlc ? "DLC" : "Base"}</span>
      <span class="rounded-full border border-white/25 bg-black/45 px-2.5 py-1 text-[0.68rem] font-black uppercase tracking-wide text-white backdrop-blur">{modTypeLabel(skin.modType)}</span>
      {#if skin.isPinned}
        <span class="rounded-full border border-white/25 bg-black/45 px-2.5 py-1 text-[0.68rem] font-black uppercase tracking-wide text-white backdrop-blur">Pinned</span>
      {/if}
      {#if images.length > 1}
        <span class="rounded-full border border-white/25 bg-black/45 px-2.5 py-1 text-[0.68rem] font-black text-white backdrop-blur">{activeImage + 1}/{images.length}</span>
      {/if}
      {#if skin.videos?.length}
        <span class="rounded-full border border-white/25 bg-black/45 px-2.5 py-1 text-[0.68rem] font-black uppercase tracking-wide text-white backdrop-blur">Video</span>
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
      <p class="truncate text-xs font-black uppercase tracking-[0.18em] text-muted-foreground">{skin.character.displayName} / {modTypeLabel(skin.modType)}</p>
      <a class="mt-1 block focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring" href={`/skins/${skin.slug}`}>
        <h2 class="line-clamp-2 text-2xl font-black leading-tight text-foreground">{skin.title}</h2>
      </a>
      <p class="mt-1 text-xs font-bold text-muted-foreground">v{skin.version}</p>
    </div>

    <p class="line-clamp-2 min-h-11 text-sm leading-5 text-muted-foreground">{descriptionPreview || "No description yet."}</p>

    <div class="flex flex-wrap gap-2 text-xs">
      <StatPill label="views" value={skin.viewedCount} tone="neutral" />
      <StatPill label="redirects" value={skin.redirectionCount} tone="neutral" />
    </div>

    <div class="flex items-center justify-between gap-3 border-t border-white/10 pt-4 text-sm">
      {#if creatorHref}
        <a class="min-w-0 truncate font-bold text-primary hover:underline" href={creatorHref}>
          {creatorName}
        </a>
      {:else}
        <span class="min-w-0 truncate font-bold text-muted-foreground">{creatorName}</span>
      {/if}
      <button
        class="inline-flex shrink-0 items-center overflow-hidden rounded-md border border-white/14 bg-background/60 text-sm font-black transition hover:border-primary/50 hover:bg-white/10"
        title="Upvote"
        aria-label={`Upvote ${skin.title}`}
        on:click={() => onVote(skin)}
      >
        <span class="border-r border-white/10 px-2.5 py-1.5 text-primary">▲</span>
        <span class="min-w-9 px-2.5 py-1.5 text-center">{skin.voteCount}</span>
      </button>
    </div>

    {#if skin.tags.length}
      <div class="flex flex-wrap gap-2 overflow-hidden">
        {#each skin.tags.slice(0, 3) as tag}
          <TagChip label={tag} />
        {/each}
      </div>
    {/if}

    <div class="grid grid-cols-2 gap-2">
      <a class="pointer-events-auto inline-flex h-10 items-center justify-center rounded-md border border-input bg-background/70 px-4 py-2 text-sm font-bold text-foreground backdrop-blur hover:bg-accent" href={`/skins/${skin.slug}`}>View</a>
      {#if isDesktop && hostedFile}
        <button
          class="pointer-events-auto inline-flex h-10 items-center justify-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-bold text-primary-foreground hover:bg-primary/90 disabled:cursor-not-allowed disabled:opacity-60"
          type="button"
          disabled={installing}
          on:click={installHostedMod}
        >
          <LinkKindIcon kind="zip" />
          {installing ? "Installing..." : "Install"}
        </button>
      {:else if obtainHref}
        <a class="pointer-events-auto inline-flex h-10 items-center justify-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-bold text-primary-foreground hover:bg-primary/90" href={obtainHref}>
          <LinkKindIcon kind={obtainKind} />
          Obtain
        </a>
      {:else}
        <span class="inline-flex h-10 items-center justify-center rounded-md border border-border bg-background/50 px-4 py-2 text-sm font-bold text-muted-foreground">No link</span>
      {/if}
    </div>

    {#if canManagePins}
      <button
        class="inline-flex h-9 items-center justify-center rounded-md border border-white/12 bg-background/55 px-3 text-sm font-bold text-foreground hover:bg-white/10"
        type="button"
        on:click={() => onTogglePin(skin)}
      >
        {skin.isPinned ? "Unpin from profile" : "Pin to profile"}
      </button>
    {/if}
  </div>
</article>
