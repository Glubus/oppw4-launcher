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
  import { defaultLauncherConfig, type ActiveLauncherPanel, type DetectedGame, type InstalledMod, type LauncherConfig, type LauncherState, type ModProfile, type ReleaseInfo } from "$lib/components/launcher/types";

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

  $: hasGameFolder = Boolean(config.gameFolder);
  $: canLaunch = config.launchMode === "steam" || Boolean(config.gameExecutablePath);
  $: isInstalled = config.installedFiles.length > 0;
  $: currentRelease = config.modloaderRelease || "Not installed";
  $: latestReleaseLabel = latestRelease?.tagName || "Unknown";
  $: updateLabel = !isInstalled ? "Install patcher" : needsPatcherUpdate ? "Update patcher" : "";
  $: updateCount = installedMods.filter((mod) => Boolean(updateSkins[mod.path])).length;
  $: selectedProfileMods = selectedProfile ? installedMods.filter((mod) => selectedProfile?.enabledModKeys.includes(mod.modKey)) : [];

  const ctx: LauncherActionContext = {
    getConfig: () => config, setConfig: (value) => (config = value), getDetectedGame: () => detectedGame, getInstalledMods: () => installedMods,
    getProfileName: () => profileName, setProfileName: (value) => (profileName = value), getSelectedProfile: () => selectedProfile, setSelectedProfile: (value) => (selectedProfile = value),
    getUpdateSkins: () => updateSkins, setUpdateSkins: (value) => (updateSkins = value), setCheckingUpdates: (value) => (checkingUpdates = value), setUpdatingAll: (value) => (updatingAll = value),
    setError: (value) => (error = value), setMessage: (value) => (message = value), load: () => load(), save: () => save(), saveAndRefresh: (success) => saveAndRefresh(success), runBusy: (action, fallback) => runBusy(action, fallback)
  };

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
      modloaderStatus = state.modloaderStatus;
      latestRelease = state.latestRelease ?? null;
      needsPatcherUpdate = state.needsPatcherUpdate;
      localModloaderSha256 = state.localModloaderSha256 ?? null;
      remoteModloaderSha256 = state.remoteModloaderSha256 ?? null;
      await updateActions.checkInstalledUpdates(ctx, installedMods);
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

</script>

<svelte:head>
  <title>Launcher | OPPW4 Skin Hub</title>
</svelte:head>

<AppHeader />

<main class="mx-auto grid max-w-7xl gap-5 px-4 py-6">
  <LauncherHero {currentRelease} {latestReleaseLabel} {modloaderStatus} localHash={localModloaderSha256} remoteHash={remoteModloaderSha256} {updateLabel} {isDesktop} {busy} {loading} {hasGameFolder} {canLaunch} hasLatestRelease={Boolean(latestRelease)} onInstall={() => nativeActions.installModloader(ctx)} onLaunch={() => nativeActions.launchGame(ctx)} onCheck={() => nativeActions.checkModloaderIntegrity(ctx)} />

  {#if !isDesktop}
    <DesktopOnlyCard />
  {:else}
    <LauncherMessages {error} {message} />
    <Card class="p-3">
      <LauncherTabs {activePanel} onSelect={(panel) => (activePanel = panel)} />

      {#if activePanel === "mods"}
        <LauncherModsPanel installedMods={installedMods} profiles={config.modProfiles} {updateSkins} {hasGameFolder} {busy} {checkingUpdates} {updatingAll} {updateCount} onImportZip={() => nativeActions.importExternalZip(ctx)} onUpdateAll={() => updateActions.updateAllInstalledMods(ctx)} onRefresh={load} onToggleMod={(mod) => nativeActions.toggleInstalledMod(ctx, mod)} onAddToProfile={(profile, mod) => profileActions.addModToProfile(ctx, profile, mod)} />
      {:else if activePanel === "profiles"}
        <LauncherProfilesPanel profiles={config.modProfiles} {installedMods} bind:profileName {busy} onCreate={() => profileActions.createProfile(ctx)} onSaveEnabled={() => profileActions.saveCurrentProfile(ctx)} onOpen={(profile) => (selectedProfile = profile)} onApply={(profile) => profileActions.applyProfile(ctx, profile)} onDelete={(profile) => profileActions.deleteProfile(ctx, profile)} />
      {:else if activePanel === "settings"}
        <LauncherSettingsPanel bind:config {detectedGame} {hasGameFolder} onUseDetected={() => nativeActions.useDetectedGame(ctx)} onSetLaunchMode={(mode) => nativeActions.setLaunchMode(ctx, mode)} onChooseGameFolder={() => nativeActions.chooseGameFolder(ctx)} onChooseExecutable={() => nativeActions.chooseExecutable(ctx)} onRepositoryChange={() => saveAndRefresh("Repository saved.")} />
      {:else}
        <LauncherChangelogPanel {latestRelease} />
      {/if}
    </Card>
  {/if}

  {#if selectedProfile}
    <ProfileModal profile={selectedProfile} mods={selectedProfileMods} {updateSkins} {busy} onApply={(profile) => profileActions.applyProfile(ctx, profile)} onClose={() => (selectedProfile = null)} onToggleMod={(mod) => nativeActions.toggleInstalledMod(ctx, mod)} />
  {/if}
</main>
