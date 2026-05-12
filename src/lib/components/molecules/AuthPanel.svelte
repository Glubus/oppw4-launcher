<script lang="ts">
  import { apiFetch, type Session } from "$lib/api";
  import { session } from "$lib/stores/session";

  let current: Session | null = null;
  session.subscribe((value) => (current = value));

  let email = "";
  let password = "";
  let username = "";
  let authMode: "login" | "register" = "login";
  let error = "";

  async function submitAuth() {
    error = "";
    try {
      const body = authMode === "login" ? { email, password } : { username, email, password };
      const next = await apiFetch<Session>(authMode === "login" ? "/auth/login" : "/auth/register", {
        method: "POST",
        body: JSON.stringify(body)
      });
      session.set(next);
    } catch (err) {
      error = err instanceof Error ? err.message : "Connexion impossible";
    }
  }
</script>

<div class="rounded-box border border-base-content/10 bg-base-100/85 p-3 shadow-xl backdrop-blur">
  {#if current}
    <div class="flex flex-wrap items-center gap-2">
      <div class="avatar placeholder">
        <div class="w-9 rounded-box bg-neutral text-neutral-content">
          <span class="text-xs">{current.user.username.slice(0, 2).toUpperCase()}</span>
        </div>
      </div>
      <div class="mr-auto">
        <p class="text-sm font-black leading-none">{current.user.username}</p>
        <p class="text-xs text-base-content/60">{current.user.roles.join(", ")}</p>
      </div>
      <a class="btn btn-primary btn-sm" href="/upload">Upload</a>
      <button class="btn btn-ghost btn-sm" on:click={() => session.set(null)}>Logout</button>
    </div>
  {:else}
    <div class="join mb-3 w-full">
      <button class="btn join-item btn-sm flex-1 {authMode === 'login' ? 'btn-neutral' : 'btn-ghost'}" on:click={() => (authMode = "login")}>Login</button>
      <button class="btn join-item btn-sm flex-1 {authMode === 'register' ? 'btn-neutral' : 'btn-ghost'}" on:click={() => (authMode = "register")}>Register</button>
    </div>
    <div class="grid gap-2">
      {#if authMode === "register"}
        <input class="input input-bordered input-sm" bind:value={username} placeholder="username" />
      {/if}
      <input class="input input-bordered input-sm" bind:value={email} placeholder="email" />
      <input class="input input-bordered input-sm" bind:value={password} placeholder="password" type="password" />
      <button class="btn btn-primary btn-sm" on:click={submitAuth}>Enter</button>
      {#if error}<p class="text-xs font-bold text-error">{error}</p>{/if}
    </div>
  {/if}
</div>
