<script lang="ts">
  import Button from "$lib/components/ui/Button.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import ProfileStyleDropdown from "./ProfileStyleDropdown.svelte";

  export let profileName = "";
  export let installedCount = 0;
  export let busy = false;
  export let onClose: () => void = () => {};
  export let onCreate: (icon: string, color: string) => void | Promise<void> = () => {};
  export let onSaveEnabled: (icon: string, color: string) => void | Promise<void> = () => {};

  let icon = "sparkles";
  let color = "violet";

  function create() {
    void onCreate(icon, color);
    onClose();
  }

  function saveEnabled() {
    void onSaveEnabled(icon, color);
    onClose();
  }
</script>

<div class="fixed inset-0 z-50 grid place-items-center p-4">
  <button class="absolute inset-0 bg-black/70 backdrop-blur-sm" type="button" aria-label="Close profile creator" on:click={onClose}></button>
  <div class="relative w-full max-w-md rounded-lg border border-white/12 bg-background p-5 shadow-2xl" role="dialog" aria-modal="true" aria-label="Add new profile">
    <div class="mb-5">
      <p class="text-xs font-black uppercase tracking-[0.18em] text-primary">Profile</p>
      <h2 class="mt-1 text-2xl font-black">Add new profile</h2>
    </div>

    <div class="grid gap-4">
      <Input bind:value={profileName} placeholder="Profile name" />
      <ProfileStyleDropdown {icon} {color} disabled={busy} onIcon={(value) => (icon = value)} onColor={(value) => (color = value)} />
    </div>

    <div class="mt-6 grid gap-2 sm:grid-cols-2">
      <Button disabled={!profileName.trim() || busy} on:click={create}>Create</Button>
      <Button variant="outline" disabled={!profileName.trim() || !installedCount || busy} on:click={saveEnabled}>Save enabled mods</Button>
    </div>
    <Button class="mt-2 w-full" variant="ghost" on:click={onClose}>Cancel</Button>
  </div>
</div>
