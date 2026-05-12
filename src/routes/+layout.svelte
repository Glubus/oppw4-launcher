<script lang="ts">
  import "../styles.css";
  import AmethystBackground from "$lib/components/layout/AmethystBackground.svelte";
  import ToastHost from "$lib/components/organisms/ToastHost.svelte";
</script>

<AmethystBackground />
<div class="relative z-10 min-h-screen">
  <slot />
</div>
<ToastHost />
