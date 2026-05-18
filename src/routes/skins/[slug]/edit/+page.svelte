<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { apiFetch, type Character, type Session, type Skin } from "$lib/api";
  import { session } from "$lib/stores/session";
  import AppHeader from "$lib/components/organisms/AppHeader.svelte";
  import SkinEditForm from "$lib/components/organisms/SkinEditForm.svelte";
  import Card from "$lib/components/ui/Card.svelte";

  let current: Session | null = null;
  let skin: Skin | null = null;
  let characters: Character[] = [];
  let error = "";

  session.subscribe((value) => (current = value));

  $: canEdit = Boolean(
    current && skin && (
      skin.submittedByUserId === current.user.id ||
      skin.creatorUserId === current.user.id ||
      skin.creditedUserId === current.user.id ||
      current.user.roles.some((role) => role === "ROLE_ADMIN" || role === "ROLE_MODERATOR")
    )
  );

  function canEditSkin(currentSession: Session, currentSkin: Skin) {
    return currentSkin.submittedByUserId === currentSession.user.id ||
      currentSkin.creatorUserId === currentSession.user.id ||
      currentSkin.creditedUserId === currentSession.user.id ||
      currentSession.user.roles.some((role) => role === "ROLE_ADMIN" || role === "ROLE_MODERATOR");
  }

  onMount(async () => {
    if (!current) {
      await goto("/login");
      return;
    }

    try {
      const me = await apiFetch<{ user: Session["user"] }>("/me", {}, current.token);
      current = { ...current, user: me.user };
      session.set(current);
      const [skinData, charactersData] = await Promise.all([
        apiFetch<{ skin: Skin }>(`/skins/${$page.params.slug}`, {}, current.token),
        apiFetch<{ characters: Character[] }>("/characters")
      ]);
      const loadedSkin = skinData.skin;
      if (!canEditSkin(current, loadedSkin)) {
        await goto(`/skins/${loadedSkin.slug}`);
        return;
      }
      skin = loadedSkin;
      characters = charactersData.characters;
    } catch (err) {
      error = err instanceof Error ? err.message : "Could not load skin";
    }
  });
</script>

<svelte:head>
  <title>{skin ? `Edit ${skin.title} | OPPW4 Skin Hub` : "Edit skin | OPPW4 Skin Hub"}</title>
</svelte:head>

<AppHeader />

<main class="mx-auto grid max-w-6xl gap-6 px-4 py-6">
  {#if error}
    <div class="rounded-xl border border-destructive/40 bg-destructive/15 px-4 py-3 text-sm text-red-100 shadow-lg">{error}</div>
  {:else if !current || !skin}
    <Card class="p-10 text-center text-sm text-muted-foreground">Loading editor...</Card>
  {:else if canEdit}
    <SkinEditForm {skin} {characters} session={current} />
  {/if}
</main>
