<script lang="ts">
  import Bolt from "lucide-svelte/icons/bolt";
  import Check from "lucide-svelte/icons/check";
  import Flame from "lucide-svelte/icons/flame";
  import FlaskConical from "lucide-svelte/icons/flask-conical";
  import Gamepad2 from "lucide-svelte/icons/gamepad-2";
  import Sparkles from "lucide-svelte/icons/sparkles";

  export let name = "sparkles";
  export let className = "h-6 w-6";
</script>

{#if name === "bolt"}
  <Bolt class={className} />
{:else if name === "flame"}
  <Flame class={className} />
{:else if name === "test"}
  <FlaskConical class={className} />
{:else if name === "clean"}
  <Check class={className} />
{:else if name === "gamepad"}
  <Gamepad2 class={className} />
{:else}
  <Sparkles class={className} />
{/if}
