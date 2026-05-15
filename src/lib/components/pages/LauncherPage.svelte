<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import AppHeader from "$lib/components/organisms/AppHeader.svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import type { Skin } from "$lib/api";
  import DesktopOnlyCard from "$lib/components/launcher/DesktopOnlyCard.svelte";
  import LauncherChangelogPanel from "$lib/components/launcher/LauncherChangelogPanel.svelte";
  import LauncherHero from "$lib/components/launcher/LauncherHero.svelte";
  import LauncherMessages from "$lib/components/launcher/LauncherMessages.svelte";
  import LauncherModsPanel from "$lib/components/launcher/LauncherModsPanel.svelte";
  import LauncherProfilesPanel from "$lib/components/launcher/LauncherProfilesPanel.svelte";
  import LauncherSettingsPanel from "$lib/components/launcher/LauncherSettingsPanel.svelte";
  import LauncherTabs from "$lib/components/launcher/LauncherTabs.svelte";
  import ProfileModal from "$lib/components/launcher/ProfileModal.svelte";
  import { errorMessage } from "$lib/components/launcher/helpers";
  import type { LauncherActionContext } from "$lib/components/launcher/actionContext";
  import * as nativeActions from "$lib/components/launcher/nativeActions";
  import * as profileActions from "$lib/components/launcher/profileActions";
  import * as updateActions from "$lib/components/launcher/updateActions";
  import { defaultLauncherConfig, type ActiveLauncherPanel, type DetectedGame, type HealthCheckItem, type InstalledMod, type LauncherConfig, type LauncherState, type ModProfile, type ReleaseInfo } from "$lib/components/launcher/types";

  let config = defaultLauncherConfig;
  let detectedGame: DetectedGame | null = null;
  let loading = true;
  let busy = false;
  let error = "";
  let message = "";
  let isDesktop = false;
  let installedMods: InstalledMod[] = [];
  let modloaderStatus = "Missing";
  let latestRelease: ReleaseInfo | null = null;
  let needsPatcherUpdate = false;
  let localModloaderSha256: string | null = null;
  let remoteModloaderSha256: string | null = null;
  let activePanel: ActiveLauncherPanel = "mods";
  let updateSkins: Record<string, Skin> = {};
  let checkingUpdates = false;
  let updatingAll = false;
  let profileName = "";
  let selectedProfile: ModProfile | null = null;
  let healthItems: HealthCheckItem[] = [];
  let lastUpdateFingerprint = "";

  $: hasGameFolder = Boolean(config.gameFolder);
  $: canLaunch = config.launchMode === "steam" || Boolean(config.gameExecutablePath);
  $: isInstalled = config.installedFiles.length > 0;
  $: currentRelease = config.modloaderRelease || "Not installed";
  $: latestReleaseLabel = latestRelease?.tagName || "";
  $: latestReleaseDate = latestRelease?.publishedAt || "";
  $: updateLabel = !isInstalled ? "Install patcher" : needsPatcherUpdate ? "Update patcher" : "";
  $: updateCount = installedMods.filter((mod) => Boolean(updateSkins[mod.path])).length;
  $: selectedProfileMods = selectedProfile ? installedMods.filter((mod) => selectedProfile?.enabledModKeys.includes(mod.modKey)) : [];
  $: installedModsFingerprint = installedMods.map((mod) => [mod.path, mod.version ?? "", mod.slug ?? "", mod.sourceUrl ?? ""].join("|")).join("::");
  $: if (isDesktop && installedModsFingerprint !== lastUpdateFingerprint) {
    lastUpdateFingerprint = installedModsFingerprint;
    void updateActions.checkInstalledUpdates(ctx, installedMods);
  }

  const ctx: LauncherActionContext = {
    getConfig: () => config, setConfig: (value) => (config = value), getDetectedGame: () => detectedGame, getInstalledMods: () => installedMods,
    getProfileName: () => profileName, setProfileName: (value) => (profileName = value), getSelectedProfile: () => selectedProfile, setSelectedProfile: (value) => (selectedProfile = value),
    getUpdateSkins: () => updateSkins, setUpdateSkins: (value) => (updateSkins = value), setCheckingUpdates: (value) => (checkingUpdates = value), setUpdatingAll: (value) => (updatingAll = value),
    setError: (value) => (error = value), setMessage: (value) => (message = value), load: () => load(), save: () => save(), saveAndRefresh: (success) => saveAndRefresh(success), runBusy: (action, fallback) => runBusy(action, fallback)
  };

  onMount(() => {
    isDesktop = "__TAURI_INTERNALS__" in window;
    if (!isDesktop) {
      loading = false;
      return;
    }
    void load();
    const refreshOnFocus = () => {
      if (!loading && !busy) void load();
    };
    const refreshOnVisible = () => {
      if (document.visibilityState === "visible") refreshOnFocus();
    };
    window.addEventListener("focus", refreshOnFocus);
    document.addEventListener("visibilitychange", refreshOnVisible);
    return () => {
      window.removeEventListener("focus", refreshOnFocus);
      document.removeEventListener("visibilitychange", refreshOnVisible);
    };
  });

  async function load() {
    loading = true;
    error = "";
    try {
      const state = await invoke<LauncherState>("get_launcher_state");
      config = state.config;
      detectedGame = state.detectedGame ?? null;
      installedMods = state.installedMods ?? [];
      modloaderStatus = state.modloaderStatus;
      latestRelease = state.latestRelease ?? null;
      needsPatcherUpdate = state.needsPatcherUpdate;
      localModloaderSha256 = state.localModloaderSha256 ?? null;
      remoteModloaderSha256 = state.remoteModloaderSha256 ?? null;
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

  async function runBusy(action: () => Promise<void>, fallback: string) {
    busy = true;
    error = "";
    message = "";
    try {
      await action();
    } catch (err) {
      error = errorMessage(err, fallback);
    } finally {
      busy = false;
    }
  }

  async function runHealthCheck() {
    error = "";
    message = "";
    try {
      await nativeActions.runHealthCheck((items) => (healthItems = items));
      message = "Health check complete.";
    } catch (err) {
      error = errorMessage(err, "Could not run health check");
    }
  }

  function createProfile(icon: string, color: string) {
    return profileActions.createProfile(ctx, icon, color);
  }

  function saveEnabledProfile(icon: string, color: string) {
    return profileActions.saveCurrentProfile(ctx, icon, color);
  }

  function openProfile(profile: ModProfile) {
    selectedProfile = profile;
  }

  function applyProfile(profile: ModProfile) {
    return profileActions.applyProfile(ctx, profile);
  }

  function deleteProfile(profile: ModProfile) {
    return profileActions.deleteProfile(ctx, profile);
  }

  function updateProfileStyle(profile: ModProfile, icon: string, color: string) {
    return profileActions.updateProfileStyle(ctx, profile, icon, color);
  }

  function closeProfile() {
    selectedProfile = null;
  }

  function toggleProfileMod(mod: InstalledMod) {
    return nativeActions.toggleInstalledMod(ctx, mod);
  }
</script>

<svelte:head>
  <title>Launcher | OPPW4 Skin Hub</title>
</svelte:head>

<AppHeader />

<main class="mx-auto grid w-full max-w-7xl gap-5 px-4 py-6">
  <LauncherHero {currentRelease} {latestReleaseLabel} {latestReleaseDate} {modloaderStatus} {updateLabel} {isDesktop} {busy} {loading} {hasGameFolder} {canLaunch} hasLatestRelease={Boolean(latestRelease)} onInstall={() => nativeActions.installModloader(ctx)} onLaunch={() => nativeActions.launchGame(ctx)} onCheck={() => nativeActions.checkModloaderIntegrity(ctx)} />

  {#if !isDesktop}
    <DesktopOnlyCard />
  {:else}
    <LauncherMessages {error} {message} />
    <Card class="w-full p-3">
      <LauncherTabs {activePanel} onSelect={(panel) => (activePanel = panel)} />

      {#if activePanel === "mods"}
        <LauncherModsPanel installedMods={installedMods} profiles={config.modProfiles} {updateSkins} {hasGameFolder} {busy} {checkingUpdates} {updatingAll} {updateCount} onImportZip={() => nativeActions.importExternalZip(ctx)} onUpdateAll={() => updateActions.updateAllInstalledMods(ctx)} onToggleMod={(mod) => nativeActions.toggleInstalledMod(ctx, mod)} onRemoveMod={(mod) => nativeActions.removeInstalledMod(ctx, mod)} onAddToProfile={(profile, mod) => profileActions.addModToProfile(ctx, profile, mod)} />
      {:else if activePanel === "profiles"}
        <LauncherProfilesPanel profiles={config.modProfiles} {installedMods} bind:profileName {busy} onCreateWithStyle={createProfile} onSaveEnabledWithStyle={saveEnabledProfile} onOpen={openProfile} onApply={applyProfile} onDelete={deleteProfile} />
      {:else if activePanel === "settings"}
        <LauncherSettingsPanel bind:config {detectedGame} {hasGameFolder} {healthItems} {busy} onUseDetected={() => nativeActions.useDetectedGame(ctx)} onSetLaunchMode={(mode) => nativeActions.setLaunchMode(ctx, mode)} onChooseGameFolder={() => nativeActions.chooseGameFolder(ctx)} onChooseExecutable={() => nativeActions.chooseExecutable(ctx)} onRepositoryChange={() => saveAndRefresh("Repository saved.")} onRunHealth={runHealthCheck} onExportDiagnostics={() => nativeActions.exportDiagnostics(ctx)} />
      {:else}
        <LauncherChangelogPanel {latestRelease} />
      {/if}
    </Card>
  {/if}

  {#if selectedProfile}
    <ProfileModal profile={selectedProfile} mods={selectedProfileMods} {updateSkins} {busy} onApply={applyProfile} onClose={closeProfile} onStyle={updateProfileStyle} onToggleMod={toggleProfileMod} />
  {/if}
</main>
