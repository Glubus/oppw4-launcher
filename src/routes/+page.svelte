<script lang="ts">
  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import { apiFetch, type Character, type Skin } from "$lib/api";
  import { session } from "$lib/stores/session";
  import AppHeader from "$lib/components/organisms/AppHeader.svelte";
  import FilterBar from "$lib/components/molecules/FilterBar.svelte";
  import SkinGrid from "$lib/components/organisms/SkinGrid.svelte";
  import { toastStore } from "$lib/stores/toasts";

  let skins: Skin[] = [];
  let characters: Character[] = [];
  let query = "";
  let character = "";
  let modType = "";
  let sort = "recent";
  let loading = true;
  let error = "";

  onMount(load);

  async function load() {
    loading = true;
    error = "";
    try {
      const params = new URLSearchParams();
      if (query) params.set("q", query);
      if (character) params.set("character", character);
      if (modType) params.set("modType", modType);
      params.set("sort", sort);
      const [skinData, characterData] = await Promise.all([
        apiFetch<{ skins: Skin[] }>(`/skins?${params}`),
        apiFetch<{ characters: Character[] }>("/characters")
      ]);
      skins = skinData.skins;
      characters = characterData.characters;
    } catch (err) {
      error = err instanceof Error ? err.message : "Could not load skins";
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
      error = err instanceof Error ? err.message : "Vote impossible";
    }
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
        <div class="text-2xl font-bold">{skins.length}</div>
      </div>
      <div class="px-5 py-3">
        <div class="text-xs text-muted-foreground">Roster</div>
        <div class="text-2xl font-bold">{characters.length}</div>
      </div>
    </div>
  </section>
  <FilterBar bind:query bind:character bind:modType bind:sort {characters} onChange={load} />

  {#if error}
    <div class="rounded-xl border border-destructive/40 bg-destructive/15 px-4 py-3 text-sm text-red-100 shadow-lg">
      <span>{error}</span>
    </div>
  {/if}

  <SkinGrid {skins} {loading} onVote={vote} />
</main>
