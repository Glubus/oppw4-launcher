<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { get } from "svelte/store";
  import { apiFetch, type PublicUser, type Skin } from "$lib/api";
  import { session } from "$lib/stores/session";
  import AppHeader from "$lib/components/organisms/AppHeader.svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import SkinGrid from "$lib/components/organisms/SkinGrid.svelte";
  import { toastStore } from "$lib/stores/toasts";

  let user: PublicUser | null = null;
  let skins: Skin[] = [];
  let loading = true;
  let error = "";
  let isLinkedUser = true;

  onMount(load);

  async function load() {
    loading = true;
    error = "";
    try {
      const current = get(session);
      let data: { user: PublicUser; skins: Skin[] };
      try {
        data = await apiFetch<{ user: PublicUser; skins: Skin[] }>(`/users/${$page.params.username}`, {}, current?.token);
        isLinkedUser = true;
      } catch {
        data = await apiFetch<{ user: PublicUser; skins: Skin[] }>(`/creators/external/${$page.params.username}`);
        isLinkedUser = false;
      }
      user = data.user;
      skins = data.skins;
    } catch (err) {
      error = err instanceof Error ? err.message : "Creator not found";
    } finally {
      loading = false;
    }
  }

  async function vote(skin: Skin) {
    const current = get(session);
    if (!current) {
      toastStore.push("Log in to upvote.", "error");
      return;
    }
    try {
      const result = await apiFetch<{ voteCount: number }>(`/skins/${skin.id}/vote`, { method: "POST" }, current.token);
      skin.voteCount = result.voteCount;
      skins = skins;
    } catch (err) {
      error = err instanceof Error ? err.message : "Could not vote";
    }
  }

  async function togglePin(skin: Skin) {
    const current = get(session);
    if (!current) return;

    try {
      await apiFetch(`/me/pinned-skins/${skin.id}`, { method: skin.isPinned ? "DELETE" : "POST" }, current.token);
      toastStore.push(skin.isPinned ? "Skin unpinned." : "Skin pinned.", "success");
      await load();
    } catch (err) {
      toastStore.push(err instanceof Error ? err.message : "Could not update pinned skins.", "error");
    }
  }
</script>

<svelte:head>
  <title>{user ? `${user.username} | OPPW4 Skin Hub` : "Creator | OPPW4 Skin Hub"}</title>
</svelte:head>

<AppHeader />

<main class="mx-auto grid max-w-7xl gap-5 px-4 py-6">
  {#if error}
    <div class="rounded-xl border border-destructive/40 bg-destructive/15 px-4 py-3 text-sm text-red-100 shadow-lg">{error}</div>
  {/if}

  {#if user}
    <Card class="bg-card/70 p-5 backdrop-blur">
      <p class="text-xs font-black uppercase tracking-[0.22em] text-primary">Creator</p>
      <div class="mt-2 flex flex-col gap-4 md:flex-row md:items-end md:justify-between">
        <div>
          <h1 class="text-4xl font-black tracking-tight">{isLinkedUser ? `@${user.username}` : user.username}</h1>
          {#if !isLinkedUser}
            <p class="mt-2 max-w-2xl text-sm text-muted-foreground">This creator page is not linked to an account yet.</p>
          {/if}
          <div class="mt-4 grid grid-cols-3 overflow-hidden rounded-lg border border-white/10 bg-background/45">
            <div class="border-r border-white/10 px-4 py-3">
              <p class="text-xs text-muted-foreground">Mods</p>
              <p class="text-2xl font-black">{user.stats?.modCount ?? skins.length}</p>
            </div>
            <div class="border-r border-white/10 px-4 py-3">
              <p class="text-xs text-muted-foreground">Downloads</p>
              <p class="text-2xl font-black">{user.stats?.downloadCount ?? 0}</p>
            </div>
            <div class="px-4 py-3">
              <p class="text-xs text-muted-foreground">Upvotes</p>
              <p class="text-2xl font-black">{user.stats?.upvoteCount ?? 0}</p>
            </div>
          </div>
        </div>
        {#if user.socialLinks.length}
          <div class="flex flex-wrap gap-2">
            {#each user.socialLinks as link}
              <a class="rounded-md border border-border bg-background/70 px-3 py-2 text-sm font-bold text-primary backdrop-blur hover:bg-accent" href={link.url} target="_blank" rel="noreferrer">{link.label}</a>
            {/each}
          </div>
        {/if}
      </div>
    </Card>
  {/if}

  <SkinGrid {skins} {loading} onVote={vote} canManagePins={Boolean(user?.canManagePins && isLinkedUser)} onTogglePin={togglePin} />
</main>
