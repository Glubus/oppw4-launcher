<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { invoke } from "@tauri-apps/api/core";
  import { API_BASE, apiFetch, type Plugin } from "$lib/api";
  import AppHeader from "$lib/components/organisms/AppHeader.svelte";
  import MarkdownContent from "$lib/components/molecules/MarkdownContent.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import { toastStore } from "$lib/stores/toasts";

  let plugin: Plugin | null = null;
  let error = "";
  let installing = false;

  $: creatorName = plugin?.creatorName ?? plugin?.creditedUsername ?? plugin?.externalCreatorName ?? "uncredited";
  $: creatorHref = plugin?.creatorSlug ? `/creators/${encodeURIComponent(plugin.creatorSlug)}` : null;
  $: isDesktop = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
  $: luaModuleName = plugin?.luaModuleName ?? plugin?.slug.replaceAll("-", "_") ?? "";

  onMount(async () => {
    try {
      const data = await apiFetch<{ plugin: Plugin }>(`/plugins/${$page.params.slug}`);
      plugin = data.plugin;
    } catch (err) {
      error = err instanceof Error ? err.message : "Plugin not found";
    }
  });

  async function install(file: { id: string; fileName: string }) {
    if (installing) return;
    installing = true;
    try {
      await invoke("install_hosted_mod", {
        input: {
          fileId: file.id,
          fileName: file.fileName,
          contentKind: "plugin",
          slug: plugin?.slug,
          title: plugin?.title,
          version: plugin?.version,
          sourceCodeUrl: plugin?.sourceCodeUrl
        }
      });
      toastStore.push("Plugin installed.", "success");
    } catch (err) {
      toastStore.push(err instanceof Error ? err.message : typeof err === "string" ? err : "Could not install plugin.", "error");
    } finally {
      installing = false;
    }
  }
</script>

<svelte:head>
  <title>{plugin ? `${plugin.title} | OPPW4 Plugin` : "Plugin | OPPW4 Skin Hub"}</title>
</svelte:head>

<AppHeader />

<main class="mx-auto grid max-w-5xl gap-5 px-4 py-6">
  {#if error}
    <div class="rounded-xl border border-destructive/40 bg-destructive/15 px-4 py-3 text-sm text-red-100 shadow-lg">{error}</div>
  {:else if !plugin}
    <div class="skeleton h-72 rounded-lg"></div>
  {:else}
    <Card class="p-5">
      <p class="text-xs font-black uppercase tracking-[0.22em] text-primary/90">Plugin / Lua library</p>
      <div class="mt-2 flex flex-wrap items-end gap-3">
        <h1 class="text-4xl font-black tracking-tight">{plugin.title}</h1>
        <span class="rounded-md border border-white/12 bg-background/55 px-2 py-1 text-xs font-black text-muted-foreground">v{plugin.version}</span>
      </div>
      <p class="mt-2 text-sm font-bold text-muted-foreground">
        by
        {#if creatorHref}
          <a class="text-primary hover:underline" href={creatorHref}>{creatorName}</a>
        {:else}
          <span>{creatorName}</span>
        {/if}
      </p>
    </Card>

    <Card class="p-5">
      <MarkdownContent value={plugin.docs} />
      {#if plugin.dependencies.length}
        <p class="mt-4 rounded-md border border-white/10 bg-background/45 p-3 text-sm font-bold text-muted-foreground">Needs {plugin.dependencies.join(", ")}</p>
      {/if}
      {#if luaModuleName}
        <p class="mt-4 rounded-md border border-white/10 bg-background/45 p-3 text-sm font-bold text-muted-foreground">Lua usage: <code class="text-foreground">require("{luaModuleName}")</code></p>
      {/if}
      <div class="mt-4 flex flex-wrap gap-2">
        <Button href={`/plugins/${plugin.slug}/docs`}>Documentation</Button>
        <Button href={plugin.sourceCodeUrl} variant="outline">Source code</Button>
      </div>
    </Card>

    {#if plugin.files?.length}
      <Card class="grid gap-3 p-5">
        <h2 class="text-lg font-black">Files</h2>
        {#each plugin.files as file}
          <div class="flex flex-col gap-3 rounded-md border border-white/10 bg-background/45 p-3 sm:flex-row sm:items-center sm:justify-between">
            <span class="font-bold">{file.fileName}</span>
            {#if isDesktop}
              <Button disabled={installing} on:click={() => install(file)}>{installing ? "Installing..." : "Install plugin"}</Button>
            {:else}
              <Button href={`${API_BASE}/files/${file.id}/download`}>Download</Button>
            {/if}
          </div>
        {/each}
      </Card>
    {/if}
  {/if}
</main>
