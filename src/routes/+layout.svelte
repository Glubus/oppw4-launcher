<script lang="ts">
  import { onMount } from "svelte";
  import "../styles.css";
  import AmethystBackground from "$lib/components/layout/AmethystBackground.svelte";
  import ToastHost from "$lib/components/organisms/ToastHost.svelte";
  import { installExternalLinkHandler } from "$lib/utils/externalLinks";

  onMount(installExternalLinkHandler);
</script>

<AmethystBackground />
<div class="relative z-10 min-h-screen">
  <slot />
</div>
<ToastHost />
