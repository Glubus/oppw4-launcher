<script lang="ts">
  import { onMount } from "svelte";
  import BrandMark from "$lib/components/atoms/BrandMark.svelte";
  import { session } from "$lib/stores/session";
  import { apiFetch, type Session } from "$lib/api";
  import Button from "$lib/components/ui/Button.svelte";
  import ThemeToggle from "$lib/components/molecules/ThemeToggle.svelte";

  let current: Session | null = null;
  session.subscribe((value) => (current = value));

  $: canAccessAdmin = Boolean(current?.user.roles.some((role) => role === "ROLE_ADMIN" || role === "ROLE_MODERATOR"));

  onMount(async () => {
    if (!current) return;
    try {
      const data = await apiFetch<{ user: Session["user"] }>("/me", {}, current.token);
      session.set({ ...current, user: data.user });
    } catch {
      session.set(null);
    }
  });
</script>

<header class="sticky top-0 z-20 border-b border-border/70 bg-background/72 backdrop-blur-xl">
  <div class="mx-auto grid max-w-7xl gap-3 px-4 py-3 md:flex md:items-center md:gap-4">
    <div class="min-w-0 md:flex-1">
      <BrandMark />
    </div>

    <nav class="hidden items-center gap-1 md:flex">
      <Button href="/" variant="ghost" size="sm">Skins</Button>
      <Button href="/launcher" variant="ghost" size="sm">Launcher</Button>
      <Button href="/modloader" variant="ghost" size="sm">Modloader</Button>
      <Button href="/upload" variant="ghost" size="sm">Upload</Button>
      {#if canAccessAdmin}
        <Button href="/admin" variant="ghost" size="sm">Admin</Button>
      {/if}
      {#if current}
        <Button href="/settings" variant="ghost" size="sm">Settings</Button>
      {/if}
    </nav>

    <div class="flex min-w-0 items-center justify-between gap-2 md:flex-1 md:justify-end">
      <nav class="flex min-w-0 items-center gap-1 overflow-x-auto md:hidden">
        <Button href="/" variant="ghost" size="sm">Skins</Button>
        <Button href="/launcher" variant="ghost" size="sm">Launcher</Button>
        <Button href="/modloader" variant="ghost" size="sm">Modloader</Button>
        <Button href="/upload" variant="ghost" size="sm">Upload</Button>
        {#if current}
          <Button href="/settings" variant="ghost" size="sm">Settings</Button>
        {/if}
      </nav>
      {#if current}
        <div class="hidden text-right sm:block">
          <a class="text-sm font-black leading-none text-foreground hover:underline" href={`/users/${encodeURIComponent(current.user.username)}`}>
            {current.user.username}
          </a>
        </div>
        {#if canAccessAdmin}
          <Button href="/admin" size="sm">Admin</Button>
        {/if}
        <Button variant="ghost" size="sm" on:click={() => session.set(null)}>Logout</Button>
      {:else}
        <Button href="/login" variant="ghost" size="sm">Login</Button>
        <Button href="/register" size="sm">Register</Button>
      {/if}
      <ThemeToggle />
    </div>
  </div>
</header>
