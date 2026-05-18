<script lang="ts">
  import { onMount } from "svelte";
  import { apiFetch, type Plugin, type PluginDocPage } from "$lib/api";
  import MarkdownContent from "$lib/components/molecules/MarkdownContent.svelte";
  import AppHeader from "$lib/components/organisms/AppHeader.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Card from "$lib/components/ui/Card.svelte";

  export let pluginSlug = "";
  export let pageSlug: string | null = null;

  let plugin: Plugin | null = null;
  let pages: PluginDocPage[] = [];
  let currentPage: PluginDocPage | null = null;
  let error = "";

  $: creatorName = plugin?.creatorName ?? plugin?.creditedUsername ?? plugin?.externalCreatorName ?? "uncredited";
  $: moduleName = plugin?.luaModuleName ?? plugin?.slug.replaceAll("-", "_") ?? "";

  onMount(loadDocs);

  async function loadDocs() {
    try {
      if (pageSlug) {
        const data = await apiFetch<{ plugin: Plugin; page: PluginDocPage; pages: PluginDocPage[] }>(`/plugins/${pluginSlug}/docs/${pageSlug}`);
        plugin = data.plugin;
        pages = data.pages;
        currentPage = data.page;
        return;
      }

      const data = await apiFetch<{ plugin: Plugin; pages: PluginDocPage[] }>(`/plugins/${pluginSlug}/docs`);
      plugin = data.plugin;
      pages = data.pages;
      currentPage = data.pages[0] ?? {
        id: "empty",
        slug: "overview",
        title: "Overview",
        body: "No documentation has been published for this plugin yet.",
        sortOrder: 0,
        createdAt: "",
        updatedAt: ""
      };
    } catch (err) {
      error = err instanceof Error ? err.message : "Plugin docs not found";
    }
  }
</script>

<svelte:head>
  <title>{plugin && currentPage ? `${currentPage.title} - ${plugin.title} Docs | OPPW4` : "Plugin Docs | OPPW4"}</title>
</svelte:head>

<AppHeader />

<main class="mx-auto grid max-w-7xl gap-5 px-4 py-6">
  {#if error}
    <div class="rounded-xl border border-destructive/40 bg-destructive/15 px-4 py-3 text-sm text-red-100 shadow-lg">{error}</div>
  {:else if !plugin || !currentPage}
    <Card class="p-10 text-center text-sm text-muted-foreground">Loading docs...</Card>
  {:else}
    <section class="grid items-start gap-5 lg:grid-cols-[280px_minmax(0,1fr)]">
      <aside class="grid content-start gap-4 rounded-lg border border-white/10 bg-card/86 p-4 shadow-[0_18px_50px_rgba(0,0,0,0.22)] backdrop-blur-md lg:sticky lg:top-24">
        <div>
          <p class="text-xs font-black uppercase tracking-[0.18em] text-muted-foreground">Lua plugin docs</p>
          <h1 class="mt-2 text-2xl font-black leading-tight">{plugin.title}</h1>
          <p class="mt-1 text-sm font-bold text-muted-foreground">v{plugin.version} · {creatorName}</p>
          <code class="mt-3 block rounded-md border border-white/10 bg-background/60 px-3 py-2 text-xs text-foreground">require("{moduleName}")</code>
        </div>

        <nav class="grid gap-1 border-t border-white/10 pt-4">
          {#if pages.length}
            {#each pages as docPage}
              <a
                class="truncate rounded-md px-3 py-2 text-sm font-bold hover:bg-white/10 {docPage.slug === currentPage.slug ? 'bg-primary/15 text-primary' : 'text-muted-foreground hover:text-foreground'}"
                href={`/plugins/${plugin.slug}/docs/${docPage.slug}`}
              >
                {docPage.title}
              </a>
            {/each}
          {:else}
            <span class="rounded-md px-3 py-2 text-sm font-bold text-muted-foreground">No pages yet</span>
          {/if}
        </nav>

        <Button href={`/plugins/${plugin.slug}`} variant="outline">Back to plugin</Button>
      </aside>

      <article class="min-w-0 rounded-lg border border-white/10 bg-card/86 p-6 shadow-[0_18px_50px_rgba(0,0,0,0.18)] backdrop-blur-md">
        <p class="text-xs font-black uppercase tracking-[0.22em] text-primary/90">Lua API</p>
        <h2 class="mt-2 text-4xl font-black tracking-tight">{currentPage.title}</h2>
        <div class="mt-6">
          <MarkdownContent value={currentPage.body} fallback="No details yet." />
        </div>
      </article>
    </section>
  {/if}
</main>
