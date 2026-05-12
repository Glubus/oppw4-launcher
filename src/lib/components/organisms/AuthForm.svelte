<script lang="ts">
  import { goto } from "$app/navigation";
  import { apiFetch, type Session } from "$lib/api";
  import { session } from "$lib/stores/session";
  import BrandMark from "$lib/components/atoms/BrandMark.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import Label from "$lib/components/ui/Label.svelte";

  export let mode: "login" | "register" = "login";

  let email = "";
  let password = "";
  let username = "";
  let error = "";
  let loading = false;

  $: isRegister = mode === "register";

  async function submit() {
    error = "";
    loading = true;
    try {
      const body = isRegister ? { username, email, password } : { email, password };
      const result = await apiFetch<Session>(isRegister ? "/auth/register" : "/auth/login", {
        method: "POST",
        body: JSON.stringify(body)
      });
      session.set(result);
      await goto("/");
    } catch (err) {
      error = err instanceof Error ? err.message : "Authentication failed";
    } finally {
      loading = false;
    }
  }
</script>

<main class="min-h-screen">
  <header class="border-b border-border/70 bg-background/72 backdrop-blur-xl">
    <div class="mx-auto flex max-w-6xl items-center justify-between px-4 py-4">
      <BrandMark />
      <Button href="/" variant="ghost" size="sm">
        <span aria-hidden="true">←</span>
        Back
      </Button>
    </div>
  </header>

  <section class="mx-auto grid min-h-[calc(100vh-73px)] max-w-6xl items-center gap-10 px-4 py-10 lg:grid-cols-[420px_1fr]">
    <Card class="relative overflow-hidden p-6">
      <div class="pointer-events-none absolute inset-x-0 top-0 h-1 bg-primary"></div>

      <div class="mb-8 flex items-start justify-between gap-4">
        <div>
          <p class="text-xs font-bold uppercase tracking-[0.22em] text-primary">
            {isRegister ? "Create access" : "Account access"}
          </p>
          <h1 class="mt-3 text-3xl font-bold tracking-tight text-foreground">
            {isRegister ? "Register" : "Login"}
          </h1>
          <p class="mt-2 text-sm leading-6 text-muted-foreground">
            {isRegister ? "Create an account to submit skins and vote." : "Sign in to manage uploads and votes."}
          </p>
        </div>
        <div class="rounded-xl border border-border bg-muted p-3 text-primary">
          <span aria-hidden="true">{isRegister ? "+" : "*"}</span>
        </div>
      </div>

      <form class="grid gap-4" on:submit|preventDefault={submit}>
        {#if isRegister}
          <Label>
            Username
            <Input bind:value={username} autocomplete="username" placeholder="modder_name" />
          </Label>
        {/if}

        <Label>
          {isRegister ? "Email" : "Email or username"}
          <Input bind:value={email} autocomplete={isRegister ? "email" : "username"} placeholder={isRegister ? "you@example.com" : "Osef or you@example.com"} />
        </Label>

        <Label>
          Password
          <Input bind:value={password} type="password" autocomplete={isRegister ? "new-password" : "current-password"} placeholder="Minimum 8 characters" />
        </Label>

        {#if error}
          <div class="rounded-lg border border-destructive/40 bg-destructive/15 px-3 py-2 text-sm text-red-100">
            {error}
          </div>
        {/if}

        <Button type="submit" class="mt-2 w-full" disabled={loading}>
          {loading ? "Please wait..." : isRegister ? "Create account" : "Login"}
        </Button>
      </form>

      <div class="mt-5 rounded-lg border border-border bg-muted/55 p-3 text-center text-sm text-muted-foreground">
        {#if isRegister}
          Already registered? <a class="font-bold text-primary hover:underline" href="/login">Login</a>
        {:else}
          Need an account? <a class="font-bold text-primary hover:underline" href="/register">Register</a>
        {/if}
      </div>
    </Card>

    <aside class="hidden lg:block">
      <div class="max-w-xl">
        <p class="text-xs font-bold uppercase tracking-[0.28em] text-primary">OPPW4 Skin Hub</p>
        <h2 class="mt-4 text-5xl font-bold leading-[0.95] tracking-tight text-foreground">Browse, credit, and preserve community skins.</h2>
        <p class="mt-5 max-w-lg text-sm leading-7 text-muted-foreground">
          A focused account layer for uploads, votes, creator attribution, and external obtain links.
        </p>
        <div class="mt-8 grid gap-3">
          {#each ["Submit skins with source links", "Vote without forcing direct downloads", "Keep creator credits visible"] as item}
            <div class="flex items-center gap-3 rounded-xl border border-border bg-card/72 p-4 backdrop-blur">
              <div class="rounded-lg bg-primary/15 px-2 py-1 text-primary" aria-hidden="true">✦</div>
              <span class="text-sm font-medium">{item}</span>
            </div>
          {/each}
        </div>
      </div>
    </aside>
  </section>
</main>
