<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { apiFetch, type Character, type Session } from "$lib/api";
  import { session } from "$lib/stores/session";
  import AppHeader from "$lib/components/organisms/AppHeader.svelte";
  import UploadForm from "$lib/components/organisms/UploadForm.svelte";

  let current: Session | null = null;
  let characters: Character[] = [];
  let error = "";

  session.subscribe((value) => (current = value));

  onMount(async () => {
    if (!current) {
      await goto("/login");
      return;
    }

    try {
      const data = await apiFetch<{ characters: Character[] }>("/characters");
      characters = data.characters;
    } catch (err) {
      error = err instanceof Error ? err.message : "Could not load characters";
    }
  });
</script>

<svelte:head>
  <title>Upload skin | OPPW4 Skin Hub</title>
</svelte:head>

<AppHeader />

<main class="mx-auto grid max-w-6xl gap-6 px-4 py-6">
  {#if error}
    <div class="rounded-xl border border-destructive/40 bg-destructive/15 px-4 py-3 text-sm text-red-100 shadow-lg">
      <span>{error}</span>
    </div>
  {/if}
  {#if current}
    <UploadForm {characters} session={current} />
  {/if}
</main>
