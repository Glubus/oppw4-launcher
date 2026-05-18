<script lang="ts">
  import MarkdownContent from "$lib/components/molecules/MarkdownContent.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import type { ReleaseInfo } from "./types";

  export let latestRelease: ReleaseInfo | null = null;
</script>

<div class="grid gap-5 p-2 pt-5">
  <div class="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
    <div>
      <h2 class="text-xl font-black">Changelog</h2>
      <p class="mt-2 text-sm leading-6 text-muted-foreground">Latest modloader release notes from GitHub.</p>
    </div>
    {#if latestRelease?.htmlUrl}
      <Button variant="outline" href={latestRelease.htmlUrl}>Open GitHub</Button>
    {/if}
  </div>

  <div class="rounded-lg border border-white/10 bg-background/45 p-4">
    <p class="text-sm font-black text-primary">{latestRelease?.name || latestRelease?.tagName || "Latest release"}</p>
    <div class="mt-4">
      <MarkdownContent value={latestRelease?.body || ""} fallback="No changelog was published for this release." />
    </div>
  </div>
</div>
