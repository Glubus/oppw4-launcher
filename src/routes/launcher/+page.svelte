<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import CharacterCombobox from "$lib/components/molecules/CharacterCombobox.svelte";
  import LauncherProfileCard from "$lib/components/molecules/LauncherProfileCard.svelte";
  import MarkdownContent from "$lib/components/molecules/MarkdownContent.svelte";
  import ModTypeCombobox from "$lib/components/molecules/ModTypeCombobox.svelte";
  import SortCombobox from "$lib/components/molecules/SortCombobox.svelte";
  import AppHeader from "$lib/components/organisms/AppHeader.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import Label from "$lib/components/ui/Label.svelte";
  import { apiFetch, type Character, type Skin } from "$lib/api";

  type LaunchMode = "steam" | "executable";

  type InstalledFile = {
    relativePath: string;
    backupPath?: string | null;
  };

  type ModProfile = {
    id: string;
    name: string;
    enabledModKeys: string[];
  };

  type LauncherConfig = {
    launchMode: LaunchMode;
    gameFolder?: string | null;
    gameExecutablePath?: string | null;
    modloaderRepo: string;
    modloaderRelease?: string | null;
    installedFiles: InstalledFile[];
    lastLaunchAt?: string | null;
    modProfiles: ModProfile[];
  };

  type DetectedGame = {
    gameFolder: string;
    executablePath?: string | null;
    source: string;
  };

  type LauncherState = {
    config: LauncherConfig;
    detectedGame?: DetectedGame | null;
    modloaderStatus: string;
    latestRelease?: ReleaseInfo | null;
    needsPatcherUpdate: boolean;
    installedMods: InstalledMod[];
  };

  type ReleaseInfo = {
    tagName: string;
    name?: string | null;
    body?: string | null;
    htmlUrl: string;
    prerelease: boolean;
    assetName?: string | null;
  };

  type InstalledMod = {
    name: string;
    kind: string;
    path: string;
    modKey: string;
    enabled: boolean;
    modId?: string | null;
    version?: string | null;
    sourceUrl?: string | null;
    slug?: string | null;
    characterName?: string | null;
    characterSlug?: string | null;
    modType?: string | null;
    coverDataUrl?: string | null;
  };

  const defaultConfig: LauncherConfig = {
    launchMode: "steam",
    gameFolder: null,
    gameExecutablePath: null,
    modloaderRepo: "Glubus/oppw4-patcher",
    modloaderRelease: null,
    installedFiles: [],
    lastLaunchAt: null,
    modProfiles: []
  };

  const statusOptions = [
    { value: "", label: "All status" },
    { value: "enabled", label: "Enabled" },
    { value: "disabled", label: "Disabled" },
    { value: "to_update", label: "To update" }
  ];

  let config = defaultConfig;
  let detectedGame: DetectedGame | null = null;
  let loading = true;
  let busy = false;
  let error = "";
  let message = "";
  let isDesktop = false;
  let installedMods: InstalledMod[] = [];
  let modSearch = "";
  let modCharacter = "";
  let modType = "";
  let modStatus = "";
  let modSort = "recent";
  let statusDetails: HTMLDetailsElement;
  let latestRelease: ReleaseInfo | null = null;
  let needsPatcherUpdate = false;
  let activePanel: "mods" | "profiles" | "settings" | "changelog" = "mods";
  let updateSkins: Record<string, Skin> = {};
  let checkingUpdates = false;
  let updatingAll = false;
  let profileName = "";
  let selectedProfile: ModProfile | null = null;

  $: hasGameFolder = Boolean(config.gameFolder);
  $: canLaunch = config.launchMode === "steam" || Boolean(config.gameExecutablePath);
  $: isInstalled = config.installedFiles.length > 0;
  $: currentRelease = config.modloaderRelease || "Not installed";
  $: latestReleaseLabel = latestRelease?.tagName || "Unknown";
  $: updateLabel = !isInstalled ? "Install patcher" : needsPatcherUpdate ? "Update patcher" : "";
  $: installedCharacters = localCharacters(installedMods);
  $: filteredInstalledMods = sortInstalledMods(
    installedMods.filter((mod) => matchesModFilters(mod)),
    modSort
  );
  $: updateCount = installedMods.filter((mod) => Boolean(updateSkins[mod.path])).length;
  $: selectedProfileMods = selectedProfile
    ? installedMods.filter((mod) => selectedProfile?.enabledModKeys.includes(mod.modKey))
    : [];

  onMount(async () => {
    isDesktop = "__TAURI_INTERNALS__" in window;
    if (!isDesktop) {
      loading = false;
      return;
    }
    await load();
  });

  async function load() {
    loading = true;
    error = "";
    try {
      const state = await invoke<LauncherState>("get_launcher_state");
      config = state.config;
      detectedGame = state.detectedGame ?? null;
      installedMods = state.installedMods ?? [];
      latestRelease = state.latestRelease ?? null;
      needsPatcherUpdate = state.needsPatcherUpdate;
      await checkInstalledUpdates(installedMods);
    } catch (err) {
      error = errorMessage(err, "Could not load launcher state");
    } finally {
      loading = false;
    }
  }

  async function save() {
    config = await invoke<LauncherConfig>("save_launcher_config", { config });
  }

  async function saveAndRefresh(success: string) {
    error = "";
    message = "";
    try {
      await save();
      await load();
      message = success;
    } catch (err) {
      error = errorMessage(err, "Could not save config");
    }
  }

  async function chooseGameFolder() {
    const selected = await open({ directory: true, multiple: false, title: "Select OPPW4 game folder" });
    if (typeof selected === "string") {
      config = { ...config, gameFolder: selected };
      await saveAndRefresh("Game folder saved.");
    }
  }

  async function chooseExecutable() {
    const selected = await open({
      directory: false,
      multiple: false,
      title: "Select OPPW4 executable",
      filters: [{ name: "Executable", extensions: ["exe", "AppImage", "sh", "x86_64"] }]
    });
    if (typeof selected === "string") {
      const parent = selected.replace(/[\\/][^\\/]+$/, "");
      config = { ...config, gameExecutablePath: selected, gameFolder: config.gameFolder ?? parent };
      await saveAndRefresh("Executable saved.");
    }
  }

  async function useDetectedGame() {
    if (!detectedGame) return;
    config = {
      ...config,
      gameFolder: detectedGame.gameFolder,
      gameExecutablePath: detectedGame.executablePath ?? config.gameExecutablePath
    };
    await saveAndRefresh("Detected Steam install saved.");
  }

  async function setLaunchMode(mode: LaunchMode) {
    config = { ...config, launchMode: mode };
    await saveAndRefresh(`Launch mode set to ${mode}.`);
  }

  async function launchGame() {
    busy = true;
    error = "";
    message = "";
    try {
      await save();
      await invoke("launch_game");
      await load();
      message = "Launch request sent.";
    } catch (err) {
      error = errorMessage(err, "Could not launch game");
    } finally {
      busy = false;
    }
  }

  async function installModloader() {
    busy = true;
    error = "";
    message = "";
    try {
      await save();
      config = await invoke<LauncherConfig>("install_modloader");
      await load();
      message = "Patcher installed.";
    } catch (err) {
      error = errorMessage(err, "Could not install patcher");
    } finally {
      busy = false;
    }
  }

  async function restoreModloader() {
    busy = true;
    error = "";
    message = "";
    try {
      config = await invoke<LauncherConfig>("restore_modloader");
      await load();
      message = "Patcher restored.";
    } catch (err) {
      error = errorMessage(err, "Could not restore patcher");
    } finally {
      busy = false;
    }
  }

  async function toggleInstalledMod(mod: InstalledMod) {
    busy = true;
    error = "";
    message = "";
    try {
      await invoke("set_mod_enabled", { input: { path: mod.path, enabled: !mod.enabled } });
      await load();
      message = `${mod.name} ${mod.enabled ? "disabled" : "enabled"}.`;
    } catch (err) {
      error = errorMessage(err, "Could not update mod state");
    } finally {
      busy = false;
    }
  }

  async function importExternalZip() {
    const selected = await open({
      directory: false,
      multiple: false,
      title: "Import external mod ZIP",
      filters: [{ name: "ZIP archive", extensions: ["zip"] }]
    });
    if (typeof selected !== "string") return;

    busy = true;
    error = "";
    message = "";
    try {
      await invoke("import_external_zip", { input: { path: selected } });
      await load();
      message = "External ZIP imported.";
    } catch (err) {
      error = errorMessage(err, "Could not import ZIP");
    } finally {
      busy = false;
    }
  }

  async function createProfile() {
    const name = profileName.trim();
    if (!name) return;
    if (config.modProfiles.some((profile) => profile.name.toLowerCase() === name.toLowerCase())) {
      error = "A profile with this name already exists.";
      return;
    }
    config = {
      ...config,
      modProfiles: [...config.modProfiles, { id: `profile-${Date.now()}`, name, enabledModKeys: [] }]
    };
    profileName = "";
    await saveAndRefresh("Profile created.");
  }

  async function saveCurrentProfile() {
    const name = profileName.trim();
    if (!name) return;
    const existingIndex = config.modProfiles.findIndex((profile) => profile.name.toLowerCase() === name.toLowerCase());
    const enabledModKeys = installedMods.filter((mod) => mod.enabled).map((mod) => mod.modKey);
    const nextProfile: ModProfile = {
      id: existingIndex >= 0 ? config.modProfiles[existingIndex].id : `profile-${Date.now()}`,
      name,
      enabledModKeys
    };
    const modProfiles = existingIndex >= 0
      ? config.modProfiles.map((profile, index) => (index === existingIndex ? nextProfile : profile))
      : [...config.modProfiles, nextProfile];
    config = { ...config, modProfiles };
    profileName = "";
    await saveAndRefresh("Profile saved.");
  }

  async function addModToProfile(profile: ModProfile, mod: InstalledMod) {
    if (profile.enabledModKeys.includes(mod.modKey)) {
      message = `${mod.name} is already in ${profile.name}.`;
      return;
    }
    config = {
      ...config,
      modProfiles: config.modProfiles.map((item) => item.id === profile.id
        ? { ...item, enabledModKeys: [...item.enabledModKeys, mod.modKey] }
        : item)
    };
    await saveAndRefresh(`${mod.name} added to ${profile.name}.`);
  }

  async function applyProfile(profile: ModProfile) {
    busy = true;
    error = "";
    message = "";
    try {
      await save();
      await invoke("apply_mod_profile", { input: { profileId: profile.id } });
      await load();
      message = `${profile.name} applied.`;
    } catch (err) {
      error = errorMessage(err, "Could not apply profile");
    } finally {
      busy = false;
    }
  }

  async function deleteProfile(profile: ModProfile) {
    config = { ...config, modProfiles: config.modProfiles.filter((item) => item.id !== profile.id) };
    if (selectedProfile?.id === profile.id) selectedProfile = null;
    await saveAndRefresh("Profile deleted.");
  }

  async function checkInstalledUpdates(mods = installedMods) {
    checkingUpdates = true;
    const nextUpdates: Record<string, Skin> = {};
    try {
      await Promise.all(
        mods.map(async (mod) => {
          const slug = mod.slug || slugFromSourceUrl(mod.sourceUrl);
          if (!slug || !mod.version) return;
          try {
            const data = await apiFetch<{ skin: Skin }>(`/skins/${encodeURIComponent(slug)}`);
            const latestFile = data.skin.files?.[0];
            if (latestFile && data.skin.version !== mod.version) {
              nextUpdates[mod.path] = data.skin;
            }
          } catch {
            // Local mods can point to stale or private pages; keep the launcher usable.
          }
        })
      );
      updateSkins = nextUpdates;
    } finally {
      checkingUpdates = false;
    }
  }

  async function updateAllInstalledMods() {
    const updates = installedMods
      .map((mod) => ({ mod, skin: updateSkins[mod.path] }))
      .filter((item): item is { mod: InstalledMod; skin: Skin } => Boolean(item.skin?.files?.[0]));
    if (!updates.length) return;

    updatingAll = true;
    error = "";
    message = "";
    try {
      for (const { skin } of updates) {
        const file = skin.files![0];
        await invoke("install_hosted_mod", { input: { fileId: file.id, fileName: file.fileName } });
      }
      await load();
      message = `Updated ${updates.length} mod${updates.length > 1 ? "s" : ""}.`;
    } catch (err) {
      error = errorMessage(err, "Could not update installed mods");
    } finally {
      updatingAll = false;
    }
  }

  function errorMessage(err: unknown, fallback: string) {
    return err instanceof Error ? err.message : typeof err === "string" ? err : fallback;
  }

  function modInitials(name: string) {
    return name
      .replace(/\.(zip|disabled)$/i, "")
      .split(/[\s._-]+/)
      .filter(Boolean)
      .slice(0, 2)
      .map((part) => part[0])
      .join("")
      .toUpperCase() || "MOD";
  }

  function modPageHref(mod: InstalledMod) {
    if (mod.slug) return `/skins/${encodeURIComponent(mod.slug)}`;
    if (!mod.sourceUrl) return null;
    try {
      const url = new URL(mod.sourceUrl);
      return url.pathname.startsWith("/skins/") ? url.pathname : mod.sourceUrl;
    } catch {
      return mod.sourceUrl;
    }
  }

  function slugFromSourceUrl(value?: string | null) {
    if (!value) return null;
    try {
      const parts = new URL(value).pathname.split("/").filter(Boolean);
      return parts[0] === "skins" ? parts[1] : null;
    } catch {
      return null;
    }
  }

  function matchesModFilters(mod: InstalledMod) {
    const query = modSearch.trim().toLowerCase();
    const matchesQuery = !query || [
      mod.name,
      mod.version,
      mod.characterName,
      mod.characterSlug,
      mod.modType,
      mod.slug
    ].some((part) => part?.toLowerCase().includes(query));
    const matchesCharacter = !modCharacter || mod.characterSlug === modCharacter;
    const matchesType = !modType || mod.modType === modType;
    const matchesStatus = !modStatus
      || (modStatus === "enabled" ? mod.enabled : modStatus === "disabled" ? !mod.enabled : Boolean(updateSkins[mod.path]));
    return matchesQuery && matchesCharacter && matchesType && matchesStatus;
  }

  function sortInstalledMods(mods: InstalledMod[], value: string) {
    return [...mods].sort((a, b) => {
      if (value === "popular") return a.name.localeCompare(b.name);
      if (value === "viewed") return (a.characterName || "").localeCompare(b.characterName || "") || a.name.localeCompare(b.name);
      return a.name.localeCompare(b.name);
    });
  }

  function localCharacters(mods: InstalledMod[]): Character[] {
    const bySlug = new Map<string, Character>();
    for (const mod of mods) {
      const slug = mod.characterSlug || mod.characterName?.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "");
      if (!slug || bySlug.has(slug)) continue;
      bySlug.set(slug, {
        id: slug,
        slug,
        displayName: mod.characterName || slug,
        isDlc: false,
        pack: "Installed"
      });
    }
    return [...bySlug.values()].sort((a, b) => a.displayName.localeCompare(b.displayName));
  }

  function resetInstalledFilters() {
    modSearch = "";
    modCharacter = "";
    modType = "";
    modStatus = "";
    modSort = "recent";
  }

  function selectStatus(next: string) {
    modStatus = next;
    if (statusDetails) statusDetails.open = false;
  }

  function profileModCount(profile: ModProfile) {
    return installedMods.filter((mod) => profile.enabledModKeys.includes(mod.modKey)).length;
  }

  function profileHasMod(profile: ModProfile, mod: InstalledMod) {
    return profile.enabledModKeys.includes(mod.modKey);
  }

  function profilePreviewMods(profile: ModProfile) {
    return installedMods.filter((mod) => profile.enabledModKeys.includes(mod.modKey) && mod.coverDataUrl);
  }

  function openProfile(profile: ModProfile) {
    selectedProfile = profile;
  }

  function closeProfile() {
    selectedProfile = null;
  }

  function noop() {}
</script>

<svelte:head>
  <title>Launcher | OPPW4 Skin Hub</title>
</svelte:head>

<AppHeader />

<main class="mx-auto grid max-w-7xl gap-5 px-4 py-6">
  <section class="flex flex-col gap-4 rounded-lg border border-white/10 bg-card/86 p-5 shadow-[0_18px_60px_rgba(0,0,0,0.25)] backdrop-blur-md md:flex-row md:items-end md:justify-between">
    <div>
      <p class="text-xs font-black uppercase tracking-[0.22em] text-primary/90">Desktop launcher</p>
      <h1 class="mt-1 text-4xl font-black tracking-tight">Launch and manage mods</h1>
      <p class="mt-2 max-w-2xl text-sm leading-6 text-muted-foreground">Installed patcher: <span class="font-bold text-foreground">{currentRelease}</span> · Latest: <span class="font-bold text-foreground">{latestReleaseLabel}</span></p>
    </div>
    <div class="flex flex-wrap gap-2">
      {#if updateLabel}
        <Button size="lg" disabled={!isDesktop || busy || !hasGameFolder || !latestRelease} on:click={installModloader}>{updateLabel}</Button>
      {/if}
      <Button size="lg" disabled={!isDesktop || loading || busy || !canLaunch} on:click={launchGame}>
        {busy ? "Working..." : "Launch game"}
      </Button>
    </div>
  </section>

  {#if !isDesktop}
    <Card class="p-6">
      <h2 class="text-xl font-black">Desktop-only tools</h2>
      <p class="mt-2 text-sm leading-6 text-muted-foreground">This page is part of the launcher build. Native game detection, file install, and launch actions are available inside the Tauri app.</p>
    </Card>
  {:else}
    {#if error}
      <div class="rounded-xl border border-destructive/40 bg-destructive/15 px-4 py-3 text-sm text-red-100 shadow-lg">{error}</div>
    {/if}
    {#if message}
      <div class="rounded-xl border border-emerald-400/30 bg-emerald-500/15 px-4 py-3 text-sm font-bold text-emerald-200 shadow-lg">{message}</div>
    {/if}

    <Card class="p-3">
      <div class="grid grid-cols-4 overflow-hidden rounded-lg border border-white/10 bg-background/45">
        <button class="h-11 font-black {activePanel === 'mods' ? 'bg-primary/20 text-primary' : 'text-muted-foreground hover:bg-white/8'}" type="button" on:click={() => (activePanel = "mods")}>Mods</button>
        <button class="h-11 font-black {activePanel === 'profiles' ? 'bg-primary/20 text-primary' : 'text-muted-foreground hover:bg-white/8'}" type="button" on:click={() => (activePanel = "profiles")}>Profiles</button>
        <button class="h-11 font-black {activePanel === 'settings' ? 'bg-primary/20 text-primary' : 'text-muted-foreground hover:bg-white/8'}" type="button" on:click={() => (activePanel = "settings")}>Settings</button>
        <button class="h-11 font-black {activePanel === 'changelog' ? 'bg-primary/20 text-primary' : 'text-muted-foreground hover:bg-white/8'}" type="button" on:click={() => (activePanel = "changelog")}>Changelog</button>
      </div>

      {#if activePanel === "mods"}
        <div class="p-2 pt-5">
          <div class="mb-5 flex items-center justify-between gap-3">
            <div>
              <h2 class="text-xl font-black">Installed mods</h2>
              <p class="mt-1 text-sm text-muted-foreground">{filteredInstalledMods.length}/{installedMods.length} found in your mods folder.</p>
            </div>
            <div class="flex flex-wrap gap-2">
              <Button variant="outline" size="sm" disabled={busy || !hasGameFolder} on:click={importExternalZip}>Import ZIP</Button>
              {#if updateCount}
                <Button size="sm" disabled={updatingAll} on:click={updateAllInstalledMods}>
                  {updatingAll ? "Updating..." : `Update all (${updateCount})`}
                </Button>
              {/if}
              <Button variant="outline" size="sm" disabled={checkingUpdates || updatingAll} on:click={load}>
                {checkingUpdates ? "Checking..." : "Refresh"}
              </Button>
            </div>
          </div>
          <section class="relative z-30 grid gap-3 overflow-visible rounded-lg border border-white/10 bg-card/86 p-3 shadow-[0_18px_50px_rgba(0,0,0,0.22)] backdrop-blur-md lg:grid-cols-[1fr_210px_240px_180px_160px_auto]">
            <label class="input input-bordered flex items-center gap-2 bg-background/60">
              <span class="font-black text-primary">⌕</span>
              <input bind:value={modSearch} placeholder="Search mod, character, version..." />
            </label>

            <ModTypeCombobox bind:value={modType} onChange={noop} />

            <CharacterCombobox characters={installedCharacters} bind:value={modCharacter} placeholder="All characters" valueKey="slug" includeAll={true} onChange={noop} />

            <details class="relative z-40 w-full" bind:this={statusDetails}>
              <summary class="flex h-10 w-full cursor-pointer list-none items-center justify-between rounded-md border border-white/12 bg-background/55 px-3 text-sm font-medium text-foreground shadow-sm outline-none transition-colors hover:bg-white/10 focus-visible:ring-2 focus-visible:ring-ring">
                <span class="truncate">{statusOptions.find((item) => item.value === modStatus)?.label ?? "All status"}</span>
                <span class="text-muted-foreground">⌄</span>
              </summary>
              <div class="absolute z-50 mt-2 w-full rounded-lg border border-white/12 bg-popover/95 p-2 text-popover-foreground shadow-2xl backdrop-blur-md">
                {#each statusOptions as item}
                  <button class="flex h-8 w-full items-center justify-start rounded-md px-2 text-sm hover:bg-white/10" class:bg-accent={item.value === modStatus} class:text-accent-foreground={item.value === modStatus} type="button" on:click={() => selectStatus(item.value)}>
                    {item.label}
                  </button>
                {/each}
              </div>
            </details>

            <SortCombobox bind:value={modSort} onChange={noop} />

            <Button variant="outline" type="button" on:click={resetInstalledFilters}>Reset</Button>
          </section>

          {#if !hasGameFolder}
            <p class="rounded-lg border border-white/12 bg-background/45 p-4 text-sm text-muted-foreground">Select a game folder in Settings to scan installed mods.</p>
          {:else if filteredInstalledMods.length}
            <section class="mt-5 grid gap-5 md:grid-cols-2 xl:grid-cols-3">
              {#each filteredInstalledMods as mod}
                <article class="group overflow-hidden rounded-lg border border-white/10 bg-card/92 shadow-[0_18px_55px_rgba(0,0,0,0.34)] backdrop-blur-md transition duration-200 hover:-translate-y-0.5 hover:border-white/30 {!mod.enabled ? 'grayscale opacity-60' : ''}">
                  <div class="relative aspect-[16/11] overflow-hidden bg-muted">
                    {#if mod.coverDataUrl}
                      <img class="h-full w-full object-cover transition duration-300 group-hover:scale-[1.035] {!mod.enabled ? 'brightness-75' : ''}" src={mod.coverDataUrl} alt={mod.name} />
                    {:else}
                      <div class="absolute inset-0 bg-[linear-gradient(135deg,hsl(var(--primary)/.22),hsl(var(--accent)/.18))]"></div>
                      <div class="absolute left-5 top-5 rounded-md border border-white/30 bg-white/12 px-4 py-3 text-4xl font-black text-white shadow-xl backdrop-blur">
                        {modInitials(mod.name)}
                      </div>
                    {/if}
                    <div class="absolute inset-x-0 bottom-0 h-28 bg-gradient-to-t from-background/88 to-transparent"></div>
                    <div class="absolute left-3 top-3 z-20 flex flex-wrap gap-2">
                      <span class="rounded-full border border-white/25 bg-black/45 px-2.5 py-1 text-[0.68rem] font-black uppercase tracking-wide text-white backdrop-blur">Installed</span>
                      {#if updateSkins[mod.path]}
                        <span class="rounded-full border border-amber-300/50 bg-amber-400/20 px-2.5 py-1 text-[0.68rem] font-black uppercase tracking-wide text-amber-100 backdrop-blur">To update</span>
                      {/if}
                      <span class="rounded-full border border-white/25 bg-black/45 px-2.5 py-1 text-[0.68rem] font-black uppercase tracking-wide text-white backdrop-blur">{mod.kind}</span>
                      <span class="rounded-full border border-white/25 bg-black/45 px-2.5 py-1 text-[0.68rem] font-black uppercase tracking-wide text-white backdrop-blur">{mod.enabled ? "Enabled" : "Disabled"}</span>
                    </div>
                    {#if modPageHref(mod)}
                      <a class="absolute inset-0 z-10 cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring" href={modPageHref(mod)!} aria-label={`Open ${mod.name}`}></a>
                    {/if}
                  </div>

                  <div class="grid gap-4 p-4">
                    <div class="min-w-0">
                      <p class="truncate text-xs font-black uppercase tracking-[0.18em] text-muted-foreground">{mod.characterName || "Local mod"} / {mod.modType || mod.kind}</p>
                      {#if modPageHref(mod)}
                        <a class="mt-1 block focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring" href={modPageHref(mod)!}>
                          <h2 class="line-clamp-2 text-2xl font-black leading-tight text-foreground">{mod.name}</h2>
                        </a>
                      {:else}
                        <h2 class="line-clamp-2 text-2xl font-black leading-tight text-foreground">{mod.name}</h2>
                      {/if}
                      {#if mod.version}
                        <p class="mt-1 text-xs font-bold text-primary">v{mod.version}</p>
                      {/if}
                      {#if updateSkins[mod.path]}
                        <p class="mt-1 text-xs font-bold text-amber-300">Latest v{updateSkins[mod.path].version}</p>
                      {/if}
                    </div>

                    <div class="grid grid-cols-2 gap-2">
                      {#if config.modProfiles.length}
                        <details class="relative z-30">
                          <summary class="flex h-10 w-full cursor-pointer list-none items-center justify-center rounded-md border border-white/12 bg-background/55 px-3 text-sm font-bold text-foreground shadow-sm transition-colors hover:bg-white/10">
                            Add to profile
                          </summary>
                          <div class="absolute bottom-12 left-0 z-50 grid min-w-44 gap-1 rounded-lg border border-white/12 bg-popover/95 p-2 text-popover-foreground shadow-2xl backdrop-blur-md">
                            {#each config.modProfiles as profile}
                              <button class="flex h-8 w-full items-center justify-between gap-3 rounded-md px-2 text-left text-sm hover:bg-white/10 disabled:cursor-not-allowed disabled:opacity-55" type="button" disabled={profileHasMod(profile, mod) || busy} on:click={() => addModToProfile(profile, mod)}>
                                <span class="truncate">{profile.name}</span>
                                {#if profileHasMod(profile, mod)}
                                  <span class="text-xs font-black text-primary">Added</span>
                                {/if}
                              </button>
                            {/each}
                          </div>
                        </details>
                      {:else}
                        <Button variant="outline" disabled={true}>No profile</Button>
                      {/if}
                      <Button variant={mod.enabled ? "destructive" : "default"} disabled={busy} on:click={() => toggleInstalledMod(mod)}>
                        {mod.enabled ? "Disable" : "Enable"}
                      </Button>
                    </div>
                  </div>
                </article>
              {/each}
            </section>
          {:else}
            <p class="mt-5 rounded-lg border border-white/12 bg-background/45 p-4 text-sm text-muted-foreground">{installedMods.length ? "No installed mods match this search." : "No installed mods found. Create a mods/ folder next to the game executable and add mod folders or zip files."}</p>
          {/if}
        </div>
      {:else if activePanel === "profiles"}
        <div class="grid gap-5 p-2 pt-5">
          <div class="flex flex-col gap-3 lg:flex-row lg:items-end lg:justify-between">
            <div>
              <h2 class="text-xl font-black">Profiles</h2>
              <p class="mt-2 text-sm leading-6 text-muted-foreground">Build presets for enabled mods, then switch setups before launching the game.</p>
            </div>
            <div class="flex flex-wrap gap-2">
              <Input class="w-56" bind:value={profileName} placeholder="Profile name" />
              <Button disabled={!profileName.trim() || busy} on:click={createProfile}>Create</Button>
              <Button variant="outline" disabled={!profileName.trim() || !installedMods.length || busy} on:click={saveCurrentProfile}>Save enabled mods</Button>
            </div>
          </div>

          {#if config.modProfiles.length}
            <section class="grid gap-5 md:grid-cols-2 xl:grid-cols-3">
              {#each config.modProfiles as profile}
                <LauncherProfileCard
                  {profile}
                  previewMods={profilePreviewMods(profile)}
                  availableCount={profileModCount(profile)}
                  {busy}
                  onOpen={openProfile}
                  onApply={applyProfile}
                  onDelete={deleteProfile}
                />
              {/each}
            </section>
          {:else}
            <p class="rounded-lg border border-white/12 bg-background/45 p-4 text-sm text-muted-foreground">No profiles yet. Create one here, then add mods from the Mods tab.</p>
          {/if}
        </div>
      {:else if activePanel === "settings"}
        <div class="grid gap-5 p-2 pt-5">
          <div>
            <h2 class="text-xl font-black">Settings</h2>
            <p class="mt-2 text-sm leading-6 text-muted-foreground">Only change this if the launcher picked the wrong game folder or if you are testing another patcher repo.</p>
          </div>

          <div class="grid gap-3 rounded-lg border border-white/10 bg-background/45 p-4">
            <div class="flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
              <div>
                <p class="font-black">Current game install</p>
                <p class="mt-1 text-sm text-muted-foreground">{hasGameFolder ? "Ready to scan installed mods." : "Select the game folder to enable local mod management."}</p>
              </div>
              <span class="w-fit rounded-full border border-white/15 bg-white/8 px-3 py-1 text-xs font-black uppercase tracking-wide text-muted-foreground">
                {config.launchMode === "steam" ? "Steam" : "Executable"}
              </span>
            </div>

            <div class="grid gap-3 md:grid-cols-2">
              <div class="min-w-0 rounded-md border border-white/10 bg-card/60 p-3">
                <p class="text-xs font-black uppercase tracking-[0.16em] text-muted-foreground">Game folder</p>
                <p class="mt-2 break-words text-sm font-bold text-foreground">{config.gameFolder || "Not selected"}</p>
              </div>
              <div class="min-w-0 rounded-md border border-white/10 bg-card/60 p-3">
                <p class="text-xs font-black uppercase tracking-[0.16em] text-muted-foreground">Executable</p>
                <p class="mt-2 break-words text-sm font-bold text-foreground">{config.gameExecutablePath || "Steam launch / not selected"}</p>
              </div>
            </div>
          </div>

          {#if detectedGame}
            <div class="flex flex-col gap-3 rounded-lg border border-primary/20 bg-primary/10 p-4 sm:flex-row sm:items-center sm:justify-between">
              <div>
                <p class="font-black">Steam install detected</p>
                <p class="mt-1 break-words text-sm text-muted-foreground">{detectedGame.gameFolder}</p>
              </div>
              <Button variant="outline" on:click={useDetectedGame}>Use this install</Button>
            </div>
          {/if}

          <div class="grid gap-4 rounded-lg border border-white/10 bg-background/45 p-4">
            <p class="font-black">Launch method</p>
            <div class="grid grid-cols-2 overflow-hidden rounded-lg border border-white/10 bg-background/45">
              <button class="h-11 font-black {config.launchMode === 'steam' ? 'bg-primary/20 text-primary' : 'text-muted-foreground hover:bg-white/8'}" type="button" on:click={() => setLaunchMode("steam")}>Steam</button>
              <button class="h-11 font-black {config.launchMode === 'executable' ? 'bg-primary/20 text-primary' : 'text-muted-foreground hover:bg-white/8'}" type="button" on:click={() => setLaunchMode("executable")}>Executable</button>
            </div>
            <div class="flex flex-wrap gap-2">
              <Button variant="outline" on:click={chooseGameFolder}>{config.gameFolder ? "Change game folder" : "Select game folder"}</Button>
              <Button variant="outline" on:click={chooseExecutable}>{config.gameExecutablePath ? "Change executable" : "Select executable"}</Button>
            </div>
          </div>

          <Label>
            Patcher GitHub repository
            <Input bind:value={config.modloaderRepo} on:change={() => saveAndRefresh("Repository saved.")} placeholder="owner/repository" />
          </Label>
        </div>
      {:else}
        <div class="grid gap-5 p-2 pt-5">
          <div class="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
            <div>
              <h2 class="text-xl font-black">Changelog</h2>
              <p class="mt-2 text-sm leading-6 text-muted-foreground">Latest patcher release notes from GitHub.</p>
            </div>
            {#if latestRelease?.htmlUrl}
              <Button variant="outline" href={latestRelease.htmlUrl}>Open GitHub</Button>
            {/if}
          </div>

          <div class="rounded-lg border border-white/10 bg-background/45 p-4">
            <p class="text-sm font-black text-primary">{latestRelease?.name || latestRelease?.tagName || "Latest release"}</p>
            <div class="mt-4">
              <MarkdownContent value={latestRelease?.body || ""} fallback="No changelog was published for this release." />
            </div>
          </div>
        </div>
      {/if}
    </Card>
  {/if}

  {#if selectedProfile}
    <div class="fixed inset-0 z-50 grid place-items-center p-4">
      <button class="absolute inset-0 bg-black/70 backdrop-blur-sm" type="button" aria-label="Close profile" on:click={closeProfile}></button>
      <div class="relative max-h-[86vh] w-full max-w-6xl overflow-hidden rounded-lg border border-white/12 bg-background shadow-2xl" role="dialog" aria-modal="true" aria-label={`${selectedProfile.name} profile`}>
        <div class="flex flex-col gap-3 border-b border-white/10 p-4 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <p class="text-xs font-black uppercase tracking-[0.18em] text-primary">Profile</p>
            <h2 class="text-2xl font-black">{selectedProfile.name}</h2>
            <p class="mt-1 text-sm text-muted-foreground">{selectedProfileMods.length}/{selectedProfile.enabledModKeys.length} linked mods available locally.</p>
          </div>
          <div class="flex flex-wrap gap-2">
            <Button disabled={busy} on:click={() => applyProfile(selectedProfile!)}>Apply</Button>
            <Button variant="outline" on:click={closeProfile}>Close</Button>
          </div>
        </div>

        <div class="max-h-[calc(86vh-110px)] overflow-auto p-4">
          {#if selectedProfileMods.length}
            <section class="grid gap-5 md:grid-cols-2 xl:grid-cols-3">
              {#each selectedProfileMods as mod}
                <article class="group overflow-hidden rounded-lg border border-white/10 bg-card/92 shadow-[0_18px_55px_rgba(0,0,0,0.28)] backdrop-blur-md transition duration-200 hover:-translate-y-0.5 hover:border-white/30 {!mod.enabled ? 'grayscale opacity-60' : ''}">
                  <div class="relative aspect-[16/11] overflow-hidden bg-muted">
                    {#if mod.coverDataUrl}
                      <img class="h-full w-full object-cover transition duration-300 group-hover:scale-[1.035] {!mod.enabled ? 'brightness-75' : ''}" src={mod.coverDataUrl} alt={mod.name} />
                    {:else}
                      <div class="absolute inset-0 bg-[linear-gradient(135deg,hsl(var(--primary)/.22),hsl(var(--accent)/.18))]"></div>
                      <div class="absolute left-5 top-5 rounded-md border border-white/30 bg-white/12 px-4 py-3 text-4xl font-black text-white shadow-xl backdrop-blur">
                        {modInitials(mod.name)}
                      </div>
                    {/if}
                    <div class="absolute inset-x-0 bottom-0 h-28 bg-gradient-to-t from-background/88 to-transparent"></div>
                    <div class="absolute left-3 top-3 z-20 flex flex-wrap gap-2">
                      <span class="rounded-full border border-white/25 bg-black/45 px-2.5 py-1 text-[0.68rem] font-black uppercase tracking-wide text-white backdrop-blur">Profile</span>
                      <span class="rounded-full border border-white/25 bg-black/45 px-2.5 py-1 text-[0.68rem] font-black uppercase tracking-wide text-white backdrop-blur">{mod.enabled ? "Enabled" : "Disabled"}</span>
                      {#if updateSkins[mod.path]}
                        <span class="rounded-full border border-amber-300/50 bg-amber-400/20 px-2.5 py-1 text-[0.68rem] font-black uppercase tracking-wide text-amber-100 backdrop-blur">To update</span>
                      {/if}
                    </div>
                    {#if modPageHref(mod)}
                      <a class="absolute inset-0 z-10 cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring" href={modPageHref(mod)!} aria-label={`Open ${mod.name}`}></a>
                    {/if}
                  </div>

                  <div class="grid gap-4 p-4">
                    <div class="min-w-0">
                      <p class="truncate text-xs font-black uppercase tracking-[0.18em] text-muted-foreground">{mod.characterName || "Local mod"} / {mod.modType || mod.kind}</p>
                      {#if modPageHref(mod)}
                        <a class="mt-1 block focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring" href={modPageHref(mod)!}>
                          <h3 class="line-clamp-2 text-2xl font-black leading-tight text-foreground">{mod.name}</h3>
                        </a>
                      {:else}
                        <h3 class="line-clamp-2 text-2xl font-black leading-tight text-foreground">{mod.name}</h3>
                      {/if}
                      {#if mod.version}
                        <p class="mt-1 text-xs font-bold text-primary">v{mod.version}</p>
                      {/if}
                    </div>

                    <div class="grid gap-2">
                      <Button variant={mod.enabled ? "destructive" : "default"} disabled={busy} on:click={() => toggleInstalledMod(mod)}>
                        {mod.enabled ? "Disable" : "Enable"}
                      </Button>
                    </div>
                  </div>
                </article>
              {/each}
            </section>
          {:else}
            <p class="rounded-lg border border-white/12 bg-card/70 p-4 text-sm text-muted-foreground">No linked mods from this profile are currently installed.</p>
          {/if}
        </div>
      </div>
    </div>
  {/if}
</main>
