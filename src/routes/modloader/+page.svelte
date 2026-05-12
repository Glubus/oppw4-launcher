<script lang="ts">
  import AppHeader from "$lib/components/organisms/AppHeader.svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import Button from "$lib/components/ui/Button.svelte";
</script>

<svelte:head>
  <title>Modloader | OPPW4 Skin Hub</title>
</svelte:head>

<AppHeader />

<main class="mx-auto grid max-w-5xl gap-5 px-4 py-6">
  <section class="rounded-lg border border-white/10 bg-card/86 p-5 shadow-[0_18px_60px_rgba(0,0,0,0.25)] backdrop-blur-md">
    <p class="text-xs font-black uppercase tracking-[0.22em] text-primary/90">Modloader</p>
    <h1 class="mt-2 text-4xl font-black tracking-tight">Install an OPPW4 skin</h1>
    <p class="mt-3 max-w-3xl text-sm leading-6 text-muted-foreground">
      This page will become the clean install guide for skins and the future modloader workflow.
      The full guide will be published when the tool is ready to share.
    </p>
  </section>

  <section class="grid gap-5 md:grid-cols-[1fr_320px]">
    <Card class="p-5">
      <div class="border-b border-white/10 pb-4">
        <h2 class="text-xl font-black">Guide in progress</h2>
        <p class="mt-2 text-sm leading-6 text-muted-foreground">
          The detailed content is not published yet. For now, this page reserves the tutorial space
          without documenting a process that is still changing.
        </p>
      </div>

      <div class="mt-5 grid gap-3">
        <div class="rounded-lg border border-dashed border-white/16 bg-background/45 p-4">
          <p class="text-sm font-black">1. Download the modloader</p>
          <p class="mt-1 text-sm text-muted-foreground">Instructions coming soon.</p>
        </div>
        <div class="rounded-lg border border-dashed border-white/16 bg-background/45 p-4">
          <p class="text-sm font-black">2. Add a skin</p>
          <p class="mt-1 text-sm text-muted-foreground">Instructions coming soon.</p>
        </div>
        <div class="rounded-lg border border-dashed border-white/16 bg-background/45 p-4">
          <p class="text-sm font-black">3. Check the in-game slot</p>
          <p class="mt-1 text-sm text-muted-foreground">Instructions coming soon.</p>
        </div>
      </div>
    </Card>

    <aside class="grid content-start gap-5">
      <Card class="p-5">
      <h2 class="text-sm font-black uppercase tracking-[0.18em] text-muted-foreground">Status</h2>
      <div class="mt-4 rounded-lg border border-white/12 bg-background/55 p-4">
        <span class="rounded-full border border-primary/30 bg-primary/12 px-2.5 py-1 text-xs font-black text-primary">Coming soon</span>
        <p class="mt-3 text-sm leading-6 text-muted-foreground">
          The install workflow will be documented here once the new modloader is ready.
        </p>
      </div>
      </Card>

      <Card class="p-5">
        <h2 class="text-sm font-black uppercase tracking-[0.18em] text-muted-foreground">Navigation</h2>
        <div class="mt-4 grid gap-2">
          <Button href="/" variant="outline" class="w-full">Browse skins</Button>
          <Button href="/upload" class="w-full">Upload a skin</Button>
        </div>
      </Card>
    </aside>
  </section>
</main>
