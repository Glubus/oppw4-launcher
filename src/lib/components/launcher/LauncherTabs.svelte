<script lang="ts">
  import type { ActiveLauncherPanel } from "./types";

  export let activePanel: ActiveLauncherPanel = "mods";
  export let onSelect: (panel: ActiveLauncherPanel) => void = () => {};

  const panels: ActiveLauncherPanel[] = ["mods", "profiles", "settings", "changelog"];
</script>

<div class="grid grid-cols-4 overflow-hidden rounded-lg border border-white/10 bg-background/45">
  {#each panels as panel}
    <button class="h-11 font-black {activePanel === panel ? 'bg-primary/20 text-primary' : 'text-muted-foreground hover:bg-white/8'}" type="button" on:click={() => onSelect(panel)}>
      {panel === "mods" ? "Mods" : panel === "profiles" ? "Profiles" : panel === "settings" ? "Settings" : "Changelog"}
    </button>
  {/each}
</div>
