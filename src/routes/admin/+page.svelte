<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { apiFetch, modTypeLabel, type PublicUser, type Skin, type Session } from "$lib/api";
  import { session } from "$lib/stores/session";
  import AppHeader from "$lib/components/organisms/AppHeader.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import Input from "$lib/components/ui/Input.svelte";

  type SkinStatus = "draft" | "pending" | "published" | "rejected" | "hidden";
  type AdminTab = "skins" | "users";

  const statuses: Array<{ value: "" | SkinStatus; label: string }> = [
    { value: "", label: "All" },
    { value: "pending", label: "Pending" },
    { value: "published", label: "Published" },
    { value: "rejected", label: "Rejected" },
    { value: "hidden", label: "Hidden" },
    { value: "draft", label: "Draft" }
  ];

  let current: Session | null = null;
  let skins: Skin[] = [];
  let users: PublicUser[] = [];
  let counts: Record<string, number> = {};
  let status: "" | SkinStatus = "";
  let tab: AdminTab = "skins";
  let q = "";
  let userQ = "";
  let error = "";
  let loading = true;
  let usersLoading = false;

  const editableRoles = ["ROLE_USER", "ROLE_CREATOR", "ROLE_MODERATOR", "ROLE_ADMIN", "ROLE_BANNED"];

  session.subscribe((value) => (current = value));

  $: canAccessAdmin = Boolean(current?.user.roles.some((role) => role === "ROLE_ADMIN" || role === "ROLE_MODERATOR"));

  onMount(async () => {
    if (!current) {
      await goto("/login");
      return;
    }
    try {
      const data = await apiFetch<{ user: Session["user"] }>("/me", {}, current.token);
      current = { ...current, user: data.user };
      session.set(current);
    } catch {
      session.set(null);
      await goto("/login");
      return;
    }
    const hasAdminAccess = current.user.roles.some((role) => role === "ROLE_ADMIN" || role === "ROLE_MODERATOR");
    if (!hasAdminAccess) {
      await goto("/");
      return;
    }
    await load();
    if (current.user.roles.includes("ROLE_ADMIN")) await loadUsers();
  });

  async function load() {
    if (!current) return;
    loading = true;
    error = "";
    try {
      const params = new URLSearchParams();
      if (status) params.set("status", status);
      if (q.trim()) params.set("q", q.trim());
      const data = await apiFetch<{ skins: Skin[]; counts: Record<string, number> }>(`/admin/skins?${params}`, {}, current.token);
      skins = data.skins;
      counts = data.counts;
    } catch (err) {
      error = err instanceof Error ? err.message : "Could not load admin skins";
    } finally {
      loading = false;
    }
  }

  async function loadUsers() {
    if (!current?.user.roles.includes("ROLE_ADMIN")) return;
    usersLoading = true;
    error = "";
    try {
      const params = new URLSearchParams();
      if (userQ.trim()) params.set("q", userQ.trim());
      const data = await apiFetch<{ users: PublicUser[] }>(`/admin/users?${params}`, {}, current.token);
      users = data.users;
    } catch (err) {
      error = err instanceof Error ? err.message : "Could not load users";
    } finally {
      usersLoading = false;
    }
  }

  async function updateStatus(skin: Skin, nextStatus: SkinStatus) {
    if (!current) return;
    error = "";
    try {
      await apiFetch(`/admin/skins/${skin.id}/status`, {
        method: "PATCH",
        body: JSON.stringify({ status: nextStatus })
      }, current.token);
      skin.status = nextStatus;
      skins = skins;
      await load();
    } catch (err) {
      error = err instanceof Error ? err.message : "Could not update skin status";
    }
  }

  async function deleteSkin(skin: Skin) {
    if (!current) return;
    if (skin.status !== "rejected") {
      error = "Only rejected mods can be deleted.";
      return;
    }
    const confirmed = window.confirm(`Delete rejected mod "${skin.title}" permanently? Uploaded files will be removed from the server.`);
    if (!confirmed) return;
    error = "";
    try {
      await apiFetch(`/skins/${skin.id}`, { method: "DELETE" }, current.token);
      skins = skins.filter((item) => item.id !== skin.id);
      await load();
    } catch (err) {
      error = err instanceof Error ? err.message : "Could not delete skin";
    }
  }

  async function toggleRole(user: PublicUser, role: string) {
    if (!current || !user.roles) return;
    const nextRoles = user.roles.includes(role)
      ? user.roles.filter((item) => item !== role)
      : [...user.roles, role];
    const normalizedRoles = nextRoles.includes("ROLE_USER") ? nextRoles : ["ROLE_USER", ...nextRoles];
    error = "";
    try {
      const data = await apiFetch<{ user: PublicUser }>(`/admin/users/${user.id}/roles`, {
        method: "PATCH",
        body: JSON.stringify({ roles: normalizedRoles })
      }, current.token);
      users = users.map((item) => (item.id === user.id ? data.user : item));
    } catch (err) {
      error = err instanceof Error ? err.message : "Could not update user roles";
    }
  }
</script>

<svelte:head>
  <title>Admin | OPPW4 Skin Hub</title>
</svelte:head>

<AppHeader />

<main class="mx-auto grid max-w-7xl gap-5 px-4 py-6">
  <section class="flex flex-col gap-4 rounded-lg border border-white/10 bg-card/72 p-5 shadow-[0_18px_60px_rgba(0,0,0,0.25)] backdrop-blur-md lg:flex-row lg:items-end lg:justify-between">
    <div>
      <p class="text-xs font-black uppercase tracking-[0.22em] text-primary/90">Admin</p>
      <h1 class="mt-1 text-3xl font-black">Skin moderation</h1>
      <p class="mt-2 max-w-2xl text-sm leading-6 text-muted-foreground">Review submissions, publish clean entries, hide broken ones, and keep repost credits under control.</p>
    </div>

    <div class="grid grid-cols-3 overflow-hidden rounded-lg border border-white/10 bg-background/45 sm:grid-cols-5">
      {#each ["pending", "published", "rejected", "hidden", "draft"] as key}
        <div class="border-r border-white/10 px-4 py-3 last:border-r-0">
          <div class="text-xs capitalize text-muted-foreground">{key}</div>
          <div class="text-2xl font-bold">{counts[key] ?? 0}</div>
        </div>
      {/each}
    </div>
  </section>

  <Card class="p-4">
    <div class="mb-4 flex flex-wrap gap-2">
      <button class="rounded-md px-3 py-2 text-sm font-black {tab === 'skins' ? 'bg-primary text-primary-foreground' : 'border border-white/12 text-muted-foreground hover:bg-white/10 hover:text-foreground'}" type="button" on:click={() => (tab = "skins")}>Skins</button>
      {#if current?.user.roles.includes("ROLE_ADMIN")}
        <button class="rounded-md px-3 py-2 text-sm font-black {tab === 'users' ? 'bg-primary text-primary-foreground' : 'border border-white/12 text-muted-foreground hover:bg-white/10 hover:text-foreground'}" type="button" on:click={() => (tab = "users")}>Users</button>
      {/if}
    </div>
    {#if tab === "skins"}
      <div class="grid gap-3 md:grid-cols-[220px_1fr_auto]">
        <select class="h-10 rounded-md border border-white/12 bg-background/55 px-3 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring" bind:value={status} on:change={load}>
          {#each statuses as item}
            <option value={item.value}>{item.label}</option>
          {/each}
        </select>
        <Input bind:value={q} placeholder="Search title, creator, tag..." />
        <Button type="button" on:click={load}>Search</Button>
      </div>
    {:else}
      <div class="grid gap-3 md:grid-cols-[1fr_auto]">
        <Input bind:value={userQ} placeholder="Search username or email..." />
        <Button type="button" on:click={loadUsers}>Search users</Button>
      </div>
    {/if}
  </Card>

  {#if error}
    <div class="rounded-xl border border-destructive/40 bg-destructive/15 px-4 py-3 text-sm text-red-100 shadow-lg">
      {error}
    </div>
  {/if}

  {#if tab === "skins"}
    <Card class="overflow-hidden">
    <div class="hidden grid-cols-[1.5fr_150px_140px_120px_180px] border-b border-white/10 bg-background/45 px-4 py-3 text-xs font-bold uppercase tracking-[0.16em] text-muted-foreground md:grid">
      <span>Skin</span>
      <span>Character</span>
      <span>Type</span>
      <span>Status</span>
      <span class="text-right">Actions</span>
    </div>

    {#if loading}
      <div class="px-4 py-10 text-center text-sm text-muted-foreground">Loading skins...</div>
    {:else if skins.length === 0}
      <div class="px-4 py-10 text-center text-sm text-muted-foreground">No skins for this filter.</div>
    {:else}
      <div class="divide-y divide-white/10">
        {#each skins as skin}
          <article class="grid gap-3 px-4 py-4 md:grid-cols-[1.5fr_150px_140px_120px_180px] md:items-center">
            <div class="min-w-0">
              <a class="block truncate text-sm font-bold text-foreground hover:text-primary hover:underline" href={`/skins/${skin.slug}`}>{skin.title}</a>
              <p class="mt-1 line-clamp-1 text-xs text-muted-foreground">
                {skin.creatorName ?? (skin.creditedUsername ? `@${skin.creditedUsername}` : skin.externalCreatorName || "uncredited")}
                · {skin.tags.join(", ") || "no tags"}
              </p>
            </div>
            <span class="text-sm text-muted-foreground">{skin.character?.displayName ?? "Plugin"}</span>
            <span class="text-sm text-muted-foreground">{skin.contentKind === "plugin" ? "Plugin" : modTypeLabel(skin.modType)}</span>
            <span class="w-fit rounded-md border border-white/12 bg-background/55 px-2 py-1 text-xs font-bold capitalize">{skin.status}</span>
            <div class="flex flex-wrap justify-start gap-2 md:justify-end">
              {#if skin.status !== "published"}
                <Button size="sm" type="button" on:click={() => updateStatus(skin, "published")}>Publish</Button>
              {/if}
              {#if skin.status !== "pending"}
                <Button size="sm" variant="outline" type="button" on:click={() => updateStatus(skin, "pending")}>Pending</Button>
              {/if}
              {#if skin.status !== "rejected"}
                <Button size="sm" variant="destructive" type="button" on:click={() => updateStatus(skin, "rejected")}>Reject</Button>
              {/if}
              {#if skin.status === "rejected"}
                <Button size="sm" variant="destructive" type="button" on:click={() => deleteSkin(skin)}>Delete</Button>
              {/if}
            </div>
          </article>
        {/each}
      </div>
    {/if}
    </Card>
  {:else}
    <Card class="overflow-hidden">
      <div class="hidden grid-cols-[1fr_1.2fr_2fr] border-b border-white/10 bg-background/45 px-4 py-3 text-xs font-bold uppercase tracking-[0.16em] text-muted-foreground md:grid">
        <span>User</span>
        <span>Email</span>
        <span class="text-right">Roles</span>
      </div>

      {#if usersLoading}
        <div class="px-4 py-10 text-center text-sm text-muted-foreground">Loading users...</div>
      {:else if users.length === 0}
        <div class="px-4 py-10 text-center text-sm text-muted-foreground">No users found.</div>
      {:else}
        <div class="divide-y divide-white/10">
          {#each users as user}
            <article class="grid gap-3 px-4 py-4 md:grid-cols-[1fr_1.2fr_2fr] md:items-center">
              <div>
                <h2 class="text-sm font-black">{user.username}</h2>
                <p class="mt-1 text-xs text-muted-foreground">{user.roles?.join(", ")}</p>
              </div>
              <p class="break-all text-sm text-muted-foreground">{user.email}</p>
              <div class="flex flex-wrap justify-start gap-2 md:justify-end">
                {#each editableRoles as role}
                  <button
                    class="rounded-md border px-3 py-1.5 text-xs font-black disabled:cursor-not-allowed disabled:opacity-45 {user.roles?.includes(role) ? 'border-primary bg-primary text-primary-foreground' : 'border-white/12 bg-background/55 text-muted-foreground hover:bg-white/10 hover:text-foreground'}"
                    type="button"
                    disabled={user.id === current?.user.id}
                    on:click={() => toggleRole(user, role)}
                  >
                    {role.replace("ROLE_", "").toLowerCase()}
                  </button>
                {/each}
              </div>
            </article>
          {/each}
        </div>
      {/if}
    </Card>
  {/if}
</main>
