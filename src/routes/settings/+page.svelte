<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { apiFetch, type Session, type SocialLink } from "$lib/api";
  import { session } from "$lib/stores/session";
  import AppHeader from "$lib/components/organisms/AppHeader.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import Label from "$lib/components/ui/Label.svelte";

  let current: Session | null = null;
  let socialLinks: SocialLink[] = [{ label: "", url: "", kind: "patreon" }];
  let message = "";
  let error = "";

  session.subscribe((value) => {
    current = value;
    socialLinks = value?.user.socialLinks?.length ? value.user.socialLinks : [{ label: "", url: "", kind: "patreon" }];
  });

  onMount(async () => {
    if (!current) await goto("/login");
  });

  function addSocialLink() {
    if (socialLinks.length >= 12) return;
    socialLinks = [...socialLinks, { label: "", url: "", kind: "external" }];
  }

  function removeSocialLink(index: number) {
    socialLinks = socialLinks.filter((_, itemIndex) => itemIndex !== index);
    if (socialLinks.length === 0) socialLinks = [{ label: "", url: "", kind: "patreon" }];
  }

  async function save() {
    if (!current) return;
    error = "";
    message = "";

    const cleanLinks = socialLinks
      .map((link) => ({ label: link.label.trim() || link.kind, url: link.url.trim(), kind: link.kind }))
      .filter((link) => link.url);

    try {
      const result = await apiFetch<{ user: Session["user"] }>("/me/social-links", {
        method: "PATCH",
        body: JSON.stringify({ socialLinks: cleanLinks })
      }, current.token);
      session.set({ ...current, user: result.user });
      message = "Social links saved.";
    } catch (err) {
      error = err instanceof Error ? err.message : "Could not save social links";
    }
  }
</script>

<svelte:head>
  <title>Settings | OPPW4 Skin Hub</title>
</svelte:head>

<AppHeader />

<main class="mx-auto grid max-w-3xl gap-5 px-4 py-6">
  <Card class="p-5">
    <p class="text-xs font-bold uppercase tracking-[0.22em] text-primary">Account</p>
    <h1 class="mt-1 text-3xl font-bold tracking-tight">Creator profile</h1>
    <p class="mt-2 text-sm leading-6 text-muted-foreground">
      These links are attached to your creator credit on every skin you publish as your own work.
    </p>
  </Card>

  {#if current}
    <form class="grid gap-5" on:submit|preventDefault={save}>
      <Card class="p-5">
        <div class="mb-5 flex items-end justify-between gap-4">
          <div>
            <h2 class="text-lg font-bold">Social networks</h2>
            <p class="mt-1 text-sm text-muted-foreground">Patreon, Ko-fi, mirrors, portfolio, whatever matters for attribution.</p>
          </div>
          <Button variant="outline" type="button" on:click={addSocialLink}>Add link</Button>
        </div>

        <div class="grid gap-4">
          {#each socialLinks as link, index}
            <div class="grid gap-3 rounded-lg border border-border bg-background/45 p-3 sm:grid-cols-[1fr_1fr_140px_auto] sm:items-end">
              <Label>
                Label
                <Input bind:value={link.label} placeholder="Patreon" />
              </Label>
              <Label>
                URL
                <Input bind:value={link.url} placeholder="https://..." />
              </Label>
              <Label>
                Type
                <select class="h-10 rounded-md border border-input bg-background px-3 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring" bind:value={link.kind}>
                  <option value="patreon">Patreon</option>
                  <option value="kofi">Ko-fi</option>
                  <option value="portfolio">Portfolio</option>
                  <option value="discord">Discord</option>
                  <option value="mirror">Mirror</option>
                  <option value="external">External</option>
                </select>
              </Label>
              <Button variant="ghost" type="button" on:click={() => removeSocialLink(index)}>Remove</Button>
            </div>
          {/each}
        </div>
      </Card>

      <Card class="p-5">
        <Button type="submit" class="w-full">Save profile</Button>
      </Card>
    </form>
  {/if}

  {#if message}
    <div class="rounded-xl border border-emerald-400/30 bg-emerald-500/15 px-4 py-3 text-sm text-emerald-100">{message}</div>
  {/if}
  {#if error}
    <div class="rounded-xl border border-destructive/40 bg-destructive/15 px-4 py-3 text-sm text-red-100">{error}</div>
  {/if}
</main>
