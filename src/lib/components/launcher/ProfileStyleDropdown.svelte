<script lang="ts">
  import ProfileIcon from "./ProfileIcon.svelte";
  import { profileColor, profileColors, profileIcon, profileIcons } from "./helpers";
  import type { ModProfile } from "./types";

  export let icon = "sparkles";
  export let color = "violet";
  export let disabled = false;
  export let onIcon: (value: string) => void = () => {};
  export let onColor: (value: string) => void = () => {};

  $: previewProfile = { id: "preview", name: "Preview", icon, color, enabledModKeys: [] } satisfies ModProfile;
  $: selectedIcon = profileIcon(previewProfile);
  $: selectedColor = profileColor(previewProfile);
</script>

<div class="flex gap-2">
  <div class="group relative">
    <button
      class="flex h-10 items-center gap-2 rounded-md border border-white/12 bg-background/70 px-3 text-sm font-black text-foreground transition hover:border-white/25 hover:bg-white/8 disabled:opacity-50"
      type="button"
      title="Profile icon"
      {disabled}
    >
      <span class="grid h-6 w-6 place-items-center rounded-md border" style={`border-color: ${selectedColor.border}; color: ${selectedColor.text};`}>
        <ProfileIcon name={selectedIcon.value} className="h-4 w-4" />
      </span>
      Icon
    </button>
    <div class="invisible absolute left-0 top-[calc(100%-1px)] z-40 grid gap-1 rounded-lg border border-white/12 bg-popover p-1.5 opacity-0 shadow-2xl transition group-hover:visible group-hover:opacity-100 group-focus-within:visible group-focus-within:opacity-100">
      {#each profileIcons as item}
        <button
          class="grid h-12 w-12 place-items-center rounded-md border border-transparent text-muted-foreground transition hover:bg-white/10 hover:text-foreground {icon === item.value ? 'bg-white/10 text-foreground' : ''}"
          type="button"
          title={item.label}
          on:click={() => onIcon(item.value)}
        >
          <ProfileIcon name={item.value} className="h-7 w-7" />
        </button>
      {/each}
    </div>
  </div>

  <div class="group relative">
    <button
      class="flex h-10 items-center gap-2 rounded-md border border-white/12 bg-background/70 px-3 text-sm font-black text-foreground transition hover:border-white/25 hover:bg-white/8 disabled:opacity-50"
      type="button"
      title="Profile color"
      {disabled}
    >
      <span class="h-6 w-6 rounded-md border" style={`background: linear-gradient(135deg, ${selectedColor.from}, ${selectedColor.to}); border-color: ${selectedColor.border};`}></span>
      Color
    </button>
    <div class="invisible absolute left-0 top-[calc(100%-1px)] z-40 grid gap-1 rounded-lg border border-white/12 bg-popover p-1.5 opacity-0 shadow-2xl transition group-hover:visible group-hover:opacity-100 group-focus-within:visible group-focus-within:opacity-100">
      {#each profileColors as item}
        <button
          class="grid h-12 w-12 place-items-center rounded-md border border-transparent transition hover:bg-white/10 {color === item.value ? 'bg-white/10' : ''}"
          type="button"
          title={item.label}
          on:click={() => onColor(item.value)}
        >
          <span class="h-8 w-8 rounded-md border-2" style={`background: linear-gradient(135deg, ${item.from}, ${item.to}); border-color: ${color === item.value ? item.text : item.border};`}></span>
        </button>
      {/each}
    </div>
  </div>
</div>
