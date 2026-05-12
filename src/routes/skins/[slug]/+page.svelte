<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { API_BASE, apiFetch, mediaUrl, modTypeLabel, type Session, type Skin } from "$lib/api";
  import { session } from "$lib/stores/session";
  import LinkKindIcon from "$lib/components/atoms/LinkKindIcon.svelte";
  import TagChip from "$lib/components/atoms/TagChip.svelte";
  import MarkdownContent from "$lib/components/molecules/MarkdownContent.svelte";
  import AppHeader from "$lib/components/organisms/AppHeader.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import { videoEmbedUrl } from "$lib/utils/video";

  let skin: Skin | null = null;
  let current: Session | null = null;
  let error = "";
  let statusMessage = "";
  let activeMedia = 0;

  session.subscribe((value) => (current = value));

  $: creatorName = skin?.creditedUsername ?? skin?.externalCreatorName ?? "uncredited";
  $: creatorHref = skin?.creditedUsername
    ? `/creators/${encodeURIComponent(skin.creditedUsername)}`
    : skin?.externalCreatorSlug
      ? `/creators/${encodeURIComponent(skin.externalCreatorSlug)}`
      : null;
  $: images = skin?.images ?? [];
  $: videos = skin?.videos ?? [];
  $: mediaItems = [
    ...videos.map((video) => ({ type: "video" as const, video })),
    ...images.map((image) => ({ type: "image" as const, image }))
  ];
  $: activeItem = mediaItems[activeMedia];
  $: activeEmbed = activeItem?.type === "video" ? videoEmbedUrl(activeItem.video.url) : null;
  $: canEdit = Boolean(
    current && skin && (
      skin.submittedByUserId === current.user.id ||
      skin.creditedUserId === current.user.id ||
      current.user.roles.some((role) => role === "ROLE_ADMIN" || role === "ROLE_MODERATOR")
    )
  );
  $: canModerate = Boolean(current?.user.roles.some((role) => role === "ROLE_ADMIN" || role === "ROLE_MODERATOR"));

  onMount(async () => {
    try {
      const data = await apiFetch<{ skin: Skin }>(`/skins/${$page.params.slug}`, {}, current?.token);
      skin = data.skin;
    } catch (err) {
      error = err instanceof Error ? err.message : "Skin not found";
    }
  });

  async function updateStatus(nextStatus: string) {
    if (!current || !skin) return;
    error = "";
    statusMessage = "";
    try {
      await apiFetch(`/admin/skins/${skin.id}/status`, {
        method: "PATCH",
        body: JSON.stringify({ status: nextStatus })
      }, current.token);
      skin = { ...skin, status: nextStatus };
      statusMessage = `Status updated to ${nextStatus}.`;
    } catch (err) {
      error = err instanceof Error ? err.message : "Could not update status";
    }
  }
</script>

<svelte:head>
  <title>{skin ? `${skin.title} | OPPW4 Skin Hub` : "Skin | OPPW4 Skin Hub"}</title>
</svelte:head>

<AppHeader />

<main class="mx-auto grid max-w-7xl gap-5 px-4 py-6">
  {#if error}
    <div class="rounded-xl border border-destructive/40 bg-destructive/15 px-4 py-3 text-sm text-red-100 shadow-lg">{error}</div>
  {:else if !skin}
    <Card class="p-10 text-center text-sm text-muted-foreground">Loading skin...</Card>
  {:else}
    <section class="grid gap-5 lg:grid-cols-[minmax(0,1fr)_340px]">
      <div class="grid gap-5">
        <Card class="overflow-hidden bg-card/70">
          {#if activeItem}
            <div class="relative aspect-video overflow-hidden bg-black">
              {#if activeItem.type === "video"}
                {#if activeEmbed}
                  <iframe
                    class="absolute inset-0 h-full w-full"
                    src={activeEmbed}
                    title={activeItem.video.label || skin.title}
                    allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share"
                    allowfullscreen
                  ></iframe>
                {:else}
                  <div class="grid h-full place-items-center p-6 text-center">
                    <a class="text-lg font-black text-primary hover:underline" href={activeItem.video.url} target="_blank" rel="noreferrer">{activeItem.video.label}</a>
                  </div>
                {/if}
              {:else}
                <img class="absolute inset-0 h-full w-full scale-110 object-cover opacity-30 blur-2xl" src={mediaUrl(activeItem.image.url)} alt="" aria-hidden="true" />
                <div class="absolute inset-0 bg-black/52"></div>
                <img class="relative z-10 h-full w-full object-contain" src={mediaUrl(activeItem.image.url)} alt={activeItem.image.alt || skin.title} />
              {/if}
            </div>
            {#if mediaItems.length > 1}
              <div class="grid grid-cols-4 gap-2 border-t border-white/10 p-3 sm:grid-cols-6">
                {#each mediaItems as item, index}
                  <button class="overflow-hidden rounded-md border bg-background/50 {index === activeMedia ? 'border-primary' : 'border-white/10'}" type="button" on:click={() => (activeMedia = index)}>
                    {#if item.type === "video"}
                      <div class="grid aspect-video place-items-center bg-black text-xs font-black uppercase tracking-wide text-white">Video</div>
                    {:else}
                      <img class="aspect-video w-full object-cover" src={mediaUrl(item.image.url)} alt={item.image.alt || skin.title} />
                    {/if}
                  </button>
                {/each}
              </div>
            {/if}
          {:else}
            <div class="grid aspect-video place-items-center bg-muted text-5xl font-black text-muted-foreground">{skin.character.displayName.slice(0, 2).toUpperCase()}</div>
          {/if}
        </Card>

        <Card class="p-5">
          <p class="text-xs font-black uppercase tracking-[0.22em] text-primary/90">{skin.character.displayName}</p>
          <div class="mt-2 flex flex-wrap items-end gap-3">
            <h1 class="text-4xl font-black tracking-tight">{skin.title}</h1>
            <span class="rounded-md border border-white/12 bg-background/55 px-2 py-1 text-xs font-black text-muted-foreground">v{skin.version}</span>
            <span class="rounded-md border border-primary/30 bg-primary/10 px-2 py-1 text-xs font-black text-primary">{modTypeLabel(skin.modType)}</span>
          </div>
          <div class="mt-4">
            <MarkdownContent value={skin.description} />
          </div>
          {#if skin.tags.length}
            <div class="mt-5 flex flex-wrap gap-2">
              {#each skin.tags as tag}
                <TagChip label={tag} />
              {/each}
            </div>
          {/if}
        </Card>
      </div>

      <aside class="grid content-start gap-4">
        <Card class="p-5">
          <h2 class="text-sm font-black uppercase tracking-[0.18em] text-muted-foreground">Creator</h2>
          {#if creatorHref}
            <a class="mt-3 block text-xl font-black text-primary hover:underline" href={creatorHref}>
              {creatorName}
            </a>
          {:else}
            <p class="mt-3 text-xl font-black text-muted-foreground">{creatorName}</p>
          {/if}
          {#if skin.creditedSocialLinks?.length}
            <div class="mt-4 flex flex-wrap gap-2">
              {#each skin.creditedSocialLinks as link}
                <a class="inline-flex items-center gap-2 rounded-md border border-white/12 bg-background/55 px-3 py-2 text-sm font-bold text-primary hover:bg-white/10" href={link.url} target="_blank" rel="noreferrer">
                  <LinkKindIcon kind={link.kind} />
                  {link.label}
                </a>
              {/each}
            </div>
          {/if}
        </Card>

        {#if canEdit}
          <Card class="p-5">
            <h2 class="text-sm font-black uppercase tracking-[0.18em] text-muted-foreground">Manage</h2>
            <Button class="mt-4 w-full" href={`/skins/${skin.slug}/edit`}>Edit mod</Button>
          </Card>
        {/if}

        {#if canModerate}
          <Card class="p-5">
            <h2 class="text-sm font-black uppercase tracking-[0.18em] text-muted-foreground">Moderation</h2>
            <div class="mt-3 flex items-center justify-between gap-3">
              <span class="text-sm text-muted-foreground">Current status</span>
              <span class="rounded-md border border-white/12 bg-background/55 px-2 py-1 text-xs font-black capitalize">{skin.status}</span>
            </div>
            <div class="mt-4 grid gap-2">
              {#if skin.status !== "published"}
                <Button type="button" class="w-full" on:click={() => updateStatus("published")}>Publish</Button>
              {/if}
              {#if skin.status !== "pending"}
                <Button type="button" variant="outline" class="w-full" on:click={() => updateStatus("pending")}>Move to pending</Button>
              {/if}
              {#if skin.status !== "rejected"}
                <Button type="button" variant="destructive" class="w-full" on:click={() => updateStatus("rejected")}>Reject</Button>
              {/if}
            </div>
            {#if statusMessage}
              <p class="mt-3 text-sm font-bold text-primary">{statusMessage}</p>
            {/if}
          </Card>
        {/if}

        <Card class="p-5">
          <h2 class="text-sm font-black uppercase tracking-[0.18em] text-muted-foreground">Obtain</h2>
          <div class="mt-4 grid gap-3">
            {#each skin.links ?? [] as link}
              <Button href={`${API_BASE}/links/${link.id}/redirect`} class="w-full" variant="outline">
                <LinkKindIcon kind={link.kind} />
                {link.label}
              </Button>
            {/each}
            {#each skin.files ?? [] as file}
              <Button href={`${API_BASE}/files/${file.id}/download`} class="w-full">
                <LinkKindIcon kind="zip" />
                {file.fileName}
              </Button>
            {/each}
            {#if !(skin.links?.length || skin.files?.length)}
              <p class="text-sm text-muted-foreground">No obtain link yet.</p>
            {/if}
          </div>
        </Card>

        <Card class="grid grid-cols-3 gap-3 p-4 text-center">
          <div class="rounded-md bg-background/45 p-3">
            <p class="text-xs text-muted-foreground">Views</p>
            <p class="text-2xl font-black">{skin.viewedCount}</p>
          </div>
          <div class="rounded-md bg-background/45 p-3">
            <p class="text-xs text-muted-foreground">Redirects</p>
            <p class="text-2xl font-black">{skin.redirectionCount}</p>
          </div>
          <div class="rounded-md bg-background/45 p-3">
            <p class="text-xs text-muted-foreground">Votes</p>
            <p class="text-2xl font-black">{skin.voteCount}</p>
          </div>
        </Card>
      </aside>
    </section>
  {/if}
</main>
