<script lang="ts">
  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import { apiFetch, type Character, type Skin } from "$lib/api";
  import { session } from "$lib/stores/session";
  import AppHeader from "$lib/components/organisms/AppHeader.svelte";
  import FilterBar from "$lib/components/molecules/FilterBar.svelte";
  import SkinGrid from "$lib/components/organisms/SkinGrid.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import { toastStore } from "$lib/stores/toasts";
  import { invoke } from "@tauri-apps/api/core";

  const PAGE_SIZE = 12;

  type SkinListResponse = {
    skins: Skin[];
    totalCount: number;
    pagination: {
      page: number;
      limit: number;
      total: number;
      hasMore: boolean;
    };
  };

  type InstalledMod = {
    path: string;
  };

  let skins: Skin[] = [];
  let visibleSkins: Skin[] = [];
  let characters: Character[] = [];
  let query = "";
  let character = "";
  let modType = "";
  let sort = "recent";
  let showAlreadyInstalled = false;
  let isDesktop = false;
  let installedSkinIds = new Set<string>();
  let loading = true;
  let loadingMore = false;
  let error = "";
  let page = 1;
  let total = 0;
  let hasMore = false;
  let sentinel: HTMLDivElement;
  let observer: IntersectionObserver;

  onMount(() => {
    isDesktop = "__TAURI_INTERNALS__" in window;
    void loadCharacters();
    void load(true);
    observer = new IntersectionObserver((entries) => {
      if (entries[0]?.isIntersecting) void loadMore();
    }, { rootMargin: "520px" });
    observer.observe(sentinel);
    return () => observer.disconnect();
  });

  async function loadCharacters() {
    try {
      const data = await apiFetch<{ characters: Character[] }>("/characters");
      characters = data.characters;
    } catch (err) {
      error = err instanceof Error ? err.message : "Could not load characters";
    }
  }

  async function load(reset = true) {
    if (reset) {
      page = 1;
      loading = true;
    } else {
      loadingMore = true;
    }
    error = "";
    try {
      const nextPage = reset ? 1 : page + 1;
      const params = new URLSearchParams();
      if (query) params.set("q", query);
      if (character) params.set("character", character);
      if (modType) params.set("modType", modType);
      params.set("sort", sort);
      params.set("limit", `${PAGE_SIZE}`);
      params.set("page", `${nextPage}`);
      const skinData = await apiFetch<SkinListResponse>(`/skins?${params}`);
      await refreshInstalledSkinState(skinData.skins);
      skins = reset ? skinData.skins : [...skins, ...skinData.skins];
      visibleSkins = filteredSkins(skins);
      page = skinData.pagination.page;
      total = skinData.totalCount;
      hasMore = skinData.pagination.hasMore;
    } catch (err) {
      error = err instanceof Error ? err.message : "Could not load skins";
    } finally {
      loading = false;
      loadingMore = false;
    }
  }

  async function loadMore() {
    if (loading || loadingMore || !hasMore) return;
    await load(false);
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
      visibleSkins = filteredSkins(skins);
    } catch (err) {
      error = err instanceof Error ? err.message : "Vote impossible";
    }
  }

  async function refreshInstalledSkinState(items: Skin[]) {
    if (!isDesktop) return;
    const installedIds = new Set(installedSkinIds);
    await Promise.all(items.map(async (skin) => {
      try {
        const installed = await invoke<InstalledMod | null>("installed_mod_for_skin", {
          input: { modId: skin.id, slug: skin.slug }
        });
        if (installed) installedIds.add(skin.id);
        else installedIds.delete(skin.id);
      } catch {
        // The public browser can still work when native mod detection is unavailable.
      }
    }));
    installedSkinIds = installedIds;
  }

  function filteredSkins(items: Skin[]) {
    if (!isDesktop || showAlreadyInstalled) return items;
    return items.filter((skin) => !installedSkinIds.has(skin.id));
  }

  function updateFilters() {
    void load(true);
  }

  function updateInstalledVisibility() {
    visibleSkins = filteredSkins(skins);
  }
</script>

<svelte:head>
  <title>OPPW4 Skin Hub</title>
</svelte:head>

<AppHeader />

<main class="mx-auto grid max-w-7xl gap-5 px-4 py-6">
  <section class="flex flex-col gap-4 rounded-lg border border-white/10 bg-card/86 p-5 shadow-[0_18px_60px_rgba(0,0,0,0.25)] backdrop-blur-md md:flex-row md:items-end md:justify-between">
    <div>
      <p class="text-xs font-black uppercase tracking-[0.22em] text-primary/90">Skin index</p>
      <h1 class="mt-1 text-4xl font-black tracking-tight">Community skins</h1>
      <p class="mt-2 max-w-2xl text-sm leading-6 text-muted-foreground">Search by character, creator, tag, popularity, views, or external obtain redirects.</p>
    </div>
    <div class="grid grid-cols-2 overflow-hidden rounded-lg border border-white/10 bg-background/45">
      <div class="border-r border-white/10 px-5 py-3">
        <div class="text-xs text-muted-foreground">Skins</div>
        <div class="text-2xl font-bold">{total}</div>
      </div>
      <div class="px-5 py-3">
        <div class="text-xs text-muted-foreground">Roster</div>
        <div class="text-2xl font-bold">{characters.length}</div>
      </div>
    </div>
  </section>
  <FilterBar bind:query bind:character bind:modType bind:sort bind:showAlreadyInstalled {characters} canFilterInstalled={isDesktop} onChange={updateFilters} onInstalledVisibilityChange={updateInstalledVisibility} />

  {#if error}
    <div class="rounded-xl border border-destructive/40 bg-destructive/15 px-4 py-3 text-sm text-red-100 shadow-lg">
      <span>{error}</span>
    </div>
  {/if}

  <SkinGrid skins={visibleSkins} {loading} onVote={vote} />

  <div bind:this={sentinel} class="flex min-h-16 items-center justify-center">
    {#if loadingMore}
      <span class="text-sm font-bold text-muted-foreground">Loading more mods...</span>
    {:else if hasMore}
      <Button variant="outline" on:click={loadMore}>Load more</Button>
    {:else if visibleSkins.length}
      <span class="text-sm font-bold text-muted-foreground">All mods loaded.</span>
    {/if}
  </div>
</main>
