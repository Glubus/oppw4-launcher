<script lang="ts">
  import { markdownToHtml } from "$lib/utils/markdown";

  export let value = "";
  export let fallback = "No description yet.";

  $: html = markdownToHtml(value);
</script>

{#if html}
  <div class="markdown-content text-sm leading-7 text-muted-foreground">
    {@html html}
  </div>
{:else}
  <p class="text-sm leading-7 text-muted-foreground">{fallback}</p>
{/if}

<style>
  :global(.markdown-content) {
    display: grid;
    gap: 0.85rem;
  }

  :global(.markdown-content h2),
  :global(.markdown-content h3),
  :global(.markdown-content h4) {
    color: hsl(var(--foreground));
    font-weight: 900;
    line-height: 1.15;
  }

  :global(.markdown-content h2) {
    font-size: 1.15rem;
  }

  :global(.markdown-content h3),
  :global(.markdown-content h4) {
    font-size: 1rem;
  }

  :global(.markdown-content ul),
  :global(.markdown-content ol) {
    display: grid;
    gap: 0.35rem;
    padding-left: 1.25rem;
  }

  :global(.markdown-content ul) {
    list-style: disc;
  }

  :global(.markdown-content ol) {
    list-style: decimal;
  }

  :global(.markdown-content a) {
    color: hsl(var(--primary));
    font-weight: 800;
    text-decoration: underline;
    text-underline-offset: 0.2em;
  }

  :global(.markdown-content blockquote) {
    border-left: 3px solid hsl(var(--primary) / 0.7);
    color: hsl(var(--foreground));
    padding-left: 0.9rem;
  }

  :global(.markdown-content code) {
    border-radius: 0.35rem;
    background: hsl(var(--muted) / 0.7);
    color: hsl(var(--foreground));
    font-size: 0.85em;
    padding: 0.15rem 0.35rem;
  }

  :global(.markdown-content pre) {
    overflow-x: auto;
    border-radius: 0.5rem;
    border: 1px solid hsl(var(--border));
    background: hsl(var(--background) / 0.7);
    padding: 0.85rem;
  }

  :global(.markdown-content pre code) {
    background: transparent;
    padding: 0;
  }
</style>
