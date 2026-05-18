<script lang="ts">
  import { page } from "$app/stores";
  import PluginDocsWiki from "$lib/components/organisms/PluginDocsWiki.svelte";
</script>

<PluginDocsWiki pluginSlug={$page.params.slug} />
