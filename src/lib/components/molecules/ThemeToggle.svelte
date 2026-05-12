<script lang="ts">
  import SunMoonIcon from "$lib/components/atoms/SunMoonIcon.svelte";
  import { themeStore } from "$lib/stores/theme";
</script>

<button
  class="inline-flex h-8 items-center gap-2 rounded-md border border-white/12 bg-background/55 px-3 text-sm font-bold text-foreground transition hover:bg-white/10 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
  type="button"
  aria-label="Toggle color theme"
  on:click={() => themeStore.toggle()}
>
  <SunMoonIcon mode={$themeStore === "dark" ? "moon" : "sun"} class="h-4 w-4" />
  <span class="hidden sm:inline">{$themeStore === "dark" ? "Dark" : "Light"}</span>
</button>
