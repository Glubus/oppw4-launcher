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

  let creator: PublicUser | null = null;
  let skins: Skin[] = [];
  let loading = true;
  let error = "";

  onMount(load);

  async function load() {
    loading = true;
    error = "";
    try {
      const data = await apiFetch<{ user: PublicUser; skins: Skin[] }>(`/creators/external/${$page.params.name}`);
      creator = data.user;
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
</script>

<svelte:head>
  <title>{creator ? `${creator.username} | OPPW4 Skin Hub` : "Creator | OPPW4 Skin Hub"}</title>
</svelte:head>

<AppHeader />

<main class="mx-auto grid max-w-7xl gap-5 px-4 py-6">
  {#if error}
    <div class="rounded-xl border border-destructive/40 bg-destructive/15 px-4 py-3 text-sm text-red-100 shadow-lg">{error}</div>
  {/if}

  {#if creator}
    <Card class="bg-card/70 p-5 backdrop-blur">
      <p class="text-xs font-black uppercase tracking-[0.22em] text-primary">Creator</p>
      <div class="mt-2 flex flex-col gap-4 md:flex-row md:items-end md:justify-between">
        <div>
          <h1 class="text-4xl font-black tracking-tight">{creator.username}</h1>
          <p class="mt-2 max-w-2xl text-sm text-muted-foreground">This creator page is not linked to an account yet.</p>
          <div class="mt-4 grid grid-cols-3 overflow-hidden rounded-lg border border-white/10 bg-background/45">
            <div class="border-r border-white/10 px-4 py-3">
              <p class="text-xs text-muted-foreground">Mods</p>
              <p class="text-2xl font-black">{creator.stats?.modCount ?? skins.length}</p>
            </div>
            <div class="border-r border-white/10 px-4 py-3">
              <p class="text-xs text-muted-foreground">Downloads</p>
              <p class="text-2xl font-black">{creator.stats?.downloadCount ?? 0}</p>
            </div>
            <div class="px-4 py-3">
              <p class="text-xs text-muted-foreground">Upvotes</p>
              <p class="text-2xl font-black">{creator.stats?.upvoteCount ?? 0}</p>
            </div>
          </div>
        </div>
        {#if creator.socialLinks.length}
          <div class="flex flex-wrap gap-2">
            {#each creator.socialLinks as link}
              <a class="rounded-md border border-border bg-background/70 px-3 py-2 text-sm font-bold text-primary backdrop-blur hover:bg-accent" href={link.url} target="_blank" rel="noreferrer">{link.label}</a>
            {/each}
          </div>
        {/if}
      </div>
    </Card>
  {/if}

  <SkinGrid {skins} {loading} onVote={vote} />
</main>
