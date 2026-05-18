<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { API_BASE, type Plugin } from "$lib/api";
  import LinkKindIcon from "$lib/components/atoms/LinkKindIcon.svelte";
  import StatPill from "$lib/components/atoms/StatPill.svelte";
  import TagChip from "$lib/components/atoms/TagChip.svelte";
  import { toastStore } from "$lib/stores/toasts";
  import { markdownToPlainText } from "$lib/utils/markdown";

  export let plugin: Plugin;

  let installing = false;

  $: isDesktop = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
  $: creatorName = plugin.creatorName ?? plugin.creditedUsername ?? plugin.externalCreatorName ?? "uncredited";
  $: creatorHref = plugin.creatorSlug ? `/creators/${encodeURIComponent(plugin.creatorSlug)}` : null;
  $: docsPreview = markdownToPlainText(plugin.docs);
  $: file = plugin.files?.[0];
  $: initials = plugin.title
    .split(/\s+/)
    .slice(0, 2)
    .map((part) => part[0])
    .join("")
    .toUpperCase();

  async function installPlugin() {
    if (!file || installing) return;
    installing = true;
    try {
      await invoke("install_hosted_mod", {
        input: {
          fileId: file.id,
          fileName: file.fileName,
          contentKind: "plugin",
          slug: plugin.slug,
          title: plugin.title,
          version: plugin.version,
          sourceCodeUrl: plugin.sourceCodeUrl
        }
      });
      toastStore.push(`${plugin.title} installed.`, "success");
    } catch (err) {
      toastStore.push(err instanceof Error ? err.message : typeof err === "string" ? err : "Could not install plugin.", "error");
    } finally {
      installing = false;
    }
  }
</script>

<article
  class="group overflow-hidden rounded-lg border border-white/10 bg-card/92 shadow-[0_18px_55px_rgba(0,0,0,0.34)] backdrop-blur-md transition duration-200 hover:-translate-y-0.5 hover:border-white/30"
>
  <div class="relative aspect-[16/11] overflow-hidden bg-muted">
    <div class="absolute inset-0 bg-[linear-gradient(135deg,hsl(var(--primary)/.22),hsl(var(--accent)/.18))]"></div>
    <div class="absolute left-5 top-5 rounded-md border border-white/30 bg-white/12 px-4 py-3 text-4xl font-black text-white shadow-xl backdrop-blur">
      {initials || "PL"}
    </div>
    <div class="absolute inset-x-0 bottom-0 h-28 bg-gradient-to-t from-background/88 to-transparent"></div>
    <a
      class="absolute inset-0 z-10 cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring"
      href={`/plugins/${plugin.slug}`}
      aria-label={`Open ${plugin.title}`}
    ></a>

    <div class="absolute left-3 top-3 z-20 flex flex-wrap gap-2">
      <span class="rounded-full border border-white/25 bg-black/45 px-2.5 py-1 text-[0.68rem] font-black uppercase tracking-wide text-white backdrop-blur">Plugin</span>
      {#if file}
        <span class="rounded-full border border-white/25 bg-black/45 px-2.5 py-1 text-[0.68rem] font-black uppercase tracking-wide text-white backdrop-blur">DLL</span>
      {/if}
    </div>
  </div>

  <div class="grid gap-4 p-4">
    <div class="min-w-0">
      <p class="truncate text-xs font-black uppercase tracking-[0.18em] text-muted-foreground">Plugin / Lua library</p>
      <a class="mt-1 block focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring" href={`/plugins/${plugin.slug}`}>
        <h2 class="line-clamp-2 text-2xl font-black leading-tight text-foreground">{plugin.title}</h2>
      </a>
      <p class="mt-1 text-xs font-bold text-muted-foreground">v{plugin.version}</p>
    </div>

    <p class="line-clamp-2 min-h-11 text-sm leading-5 text-muted-foreground">{docsPreview || "No docs yet."}</p>

    <div class="flex flex-wrap gap-2 text-xs">
      <StatPill label="views" value={plugin.viewedCount} tone="neutral" />
      <StatPill label="downloads" value={plugin.downloadCount} tone="neutral" />
    </div>

    {#if plugin.dependencies.length}
      <p class="rounded-md border border-white/10 bg-background/45 p-2 text-xs font-bold text-muted-foreground">Needs {plugin.dependencies.join(", ")}</p>
    {/if}

    <div class="flex items-center justify-between gap-3 border-t border-white/10 pt-4 text-sm">
      {#if creatorHref}
        <a class="min-w-0 truncate font-bold text-primary hover:underline" href={creatorHref}>
          {creatorName}
        </a>
      {:else}
        <span class="min-w-0 truncate font-bold text-muted-foreground">{creatorName}</span>
      {/if}
      <span class="inline-flex shrink-0 items-center overflow-hidden rounded-md border border-white/14 bg-background/60 text-sm font-black">
        <span class="border-r border-white/10 px-2.5 py-1.5 text-primary">↓</span>
        <span class="min-w-9 px-2.5 py-1.5 text-center">{plugin.downloadCount}</span>
      </span>
    </div>

    {#if plugin.tags.length}
      <div class="flex flex-wrap gap-2 overflow-hidden">
        {#each plugin.tags.slice(0, 3) as tag}
          <TagChip label={tag} />
        {/each}
      </div>
    {/if}

    <div class="grid grid-cols-2 gap-2">
      <a class="pointer-events-auto inline-flex h-10 items-center justify-center rounded-md border border-input bg-background/70 px-4 py-2 text-sm font-bold text-foreground backdrop-blur hover:bg-accent" href={`/plugins/${plugin.slug}`}>View</a>
      {#if file && isDesktop}
        <button class="pointer-events-auto inline-flex h-10 items-center justify-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-bold text-primary-foreground hover:bg-primary/90 disabled:cursor-not-allowed disabled:opacity-60" type="button" disabled={installing} on:click={installPlugin}>
          <LinkKindIcon kind="file" />
          {installing ? "Installing..." : "Install"}
        </button>
      {:else if file}
        <a class="pointer-events-auto inline-flex h-10 items-center justify-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-bold text-primary-foreground hover:bg-primary/90" href={`${API_BASE}/files/${file.id}/download`}>
          <LinkKindIcon kind="file" />
          Download
        </a>
      {:else}
        <span class="inline-flex h-10 items-center justify-center rounded-md border border-border bg-background/50 px-4 py-2 text-sm font-bold text-muted-foreground">No DLL</span>
      {/if}
    </div>
  </div>
</article>
