<script lang="ts">
  import { onMount } from "svelte";
  import AppHeader from "$lib/components/organisms/AppHeader.svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import type { Skin } from "$lib/api";
  import DesktopOnlyCard from "$lib/components/launcher/DesktopOnlyCard.svelte";
  import LauncherChangelogPanel from "$lib/components/launcher/LauncherChangelogPanel.svelte";
  import LauncherHero from "$lib/components/launcher/LauncherHero.svelte";
  import LauncherModsPanel from "$lib/components/launcher/LauncherModsPanel.svelte";
  import LauncherProfilesPanel from "$lib/components/launcher/LauncherProfilesPanel.svelte";
  import LauncherSettingsPanel from "$lib/components/launcher/LauncherSettingsPanel.svelte";
  import LauncherTabs from "$lib/components/launcher/LauncherTabs.svelte";
  import LauncherUpdateModal from "$lib/components/launcher/LauncherUpdateModal.svelte";
  import ProfileModal from "$lib/components/launcher/ProfileModal.svelte";
  import { createLauncherLogger } from "$lib/components/launcher/launcherLogger";
  import { createLauncherRuntime, type LauncherRuntimeState } from "$lib/components/launcher/launcherRuntime";
  import type { LauncherActionContext } from "$lib/components/launcher/actionContext";
  import * as nativeActions from "$lib/components/launcher/nativeActions";
  import * as profileActions from "$lib/components/launcher/profileActions";
  import * as updateActions from "$lib/components/launcher/updateActions";
  import { defaultLauncherConfig, type ActiveLauncherPanel, type DetectedGame, type HealthCheckItem, type InstalledMod, type LauncherUpdateInfo, type LauncherUpdateInstallResult, type ModProfile, type ReleaseInfo } from "$lib/components/launcher/types";
  import { invoke } from "@tauri-apps/api/core";

  let config = defaultLauncherConfig;
  let detectedGame: DetectedGame | null = null;
  let loading = true;
  let busy = false;
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
  let launcherUpdate: LauncherUpdateInfo | null = null;
  let checkingLauncherUpdate = false;
  let installingLauncherUpdate = false;
  let launcherUpdatePromptDismissed = false;
  let showLauncherUpdatePrompt = false;

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

  const logger = createLauncherLogger(() => config, () => isDesktop);
  const runtime = createLauncherRuntime({
    getConfig: () => config,
    setConfig: (value) => (config = value),
    setState: applyRuntimeState,
    setLoading: (value) => (loading = value),
    setBusy: (value) => (busy = value),
    setHealthItems: (items) => (healthItems = items),
    logger
  });
  const ctx: LauncherActionContext = {
    getConfig: () => config, setConfig: (value) => (config = value), getDetectedGame: () => detectedGame, getInstalledMods: () => installedMods,
    getProfileName: () => profileName, setProfileName: (value) => (profileName = value), getSelectedProfile: () => selectedProfile, setSelectedProfile: (value) => (selectedProfile = value),
    getUpdateSkins: () => updateSkins, setUpdateSkins: (value) => (updateSkins = value), setCheckingUpdates: (value) => (checkingUpdates = value), setUpdatingAll: (value) => (updatingAll = value),
    setError: logger.error, setMessage: logger.success, logDebug: logger.debug, load: () => runtime.load(), save: () => runtime.save(), saveAndRefresh: (success) => runtime.saveAndRefresh(success), runBusy: (action, fallback) => runtime.runBusy(action, fallback)
  };

  onMount(() => {
    isDesktop = "__TAURI_INTERNALS__" in window;
    if (!isDesktop) {
      loading = false;
      return;
    }
    void startup();
    const refreshOnFocus = () => {
      if (!loading && !busy) void runtime.load();
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

  function applyRuntimeState(state: LauncherRuntimeState) {
    config = state.config;
    detectedGame = state.detectedGame;
    installedMods = state.installedMods;
    modloaderStatus = state.modloaderStatus;
    latestRelease = state.latestRelease;
    needsPatcherUpdate = state.needsPatcherUpdate;
    localModloaderSha256 = state.localModloaderSha256;
    remoteModloaderSha256 = state.remoteModloaderSha256;
  }

  async function startup() {
    await runtime.load();
    await checkLauncherUpdate(true);
  }

  async function checkLauncherUpdate(prompt = false) {
    checkingLauncherUpdate = true;
    try {
      launcherUpdate = await invoke<LauncherUpdateInfo>("check_launcher_update");
      if (launcherUpdate.available) {
        logger.debug(`launcher update available: current=${launcherUpdate.currentVersion}; latest=${launcherUpdate.latestVersion}; asset=${launcherUpdate.assetName ?? "none"}`);
        if (prompt && !launcherUpdatePromptDismissed) showLauncherUpdatePrompt = true;
        if (!prompt) logger.success(`Launcher ${launcherUpdate.latestVersion} is available.`);
      } else if (!prompt) {
        const current = launcherUpdate.currentVersion.replace(/^v/i, "");
        const latest = launcherUpdate.latestVersion.replace(/^v/i, "");
        if (current !== latest) {
          logger.success(`Launcher build ${launcherUpdate.currentVersion} differs from GitHub release ${launcherUpdate.latestVersion}; no installable asset is available.`);
        } else {
          logger.success(`Launcher build matches GitHub release ${launcherUpdate.latestVersion}.`);
        }
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : typeof err === "string" ? err : "Could not check launcher update";
      if (!prompt) logger.error(message);
      else logger.debug(`launcher update check failed: ${message}`);
    } finally {
      checkingLauncherUpdate = false;
    }
  }

  async function installLauncherUpdate() {
    installingLauncherUpdate = true;
    try {
      const result = await invoke<LauncherUpdateInstallResult>("install_launcher_update");
      showLauncherUpdatePrompt = false;
      logger.success(`Launcher update downloaded and opened: ${result.path}`);
    } catch (err) {
      logger.error(err instanceof Error ? err.message : typeof err === "string" ? err : "Could not install launcher update");
    } finally {
      installingLauncherUpdate = false;
    }
  }

  function dismissLauncherUpdatePrompt() {
    launcherUpdatePromptDismissed = true;
    showLauncherUpdatePrompt = false;
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
    <Card class="w-full p-3">
      <LauncherTabs {activePanel} onSelect={(panel) => (activePanel = panel)} />

      {#if activePanel === "mods"}
        <LauncherModsPanel installedMods={installedMods} profiles={config.modProfiles} {updateSkins} {hasGameFolder} {busy} {checkingUpdates} {updatingAll} {updateCount} onImportZip={() => nativeActions.importExternalZip(ctx)} onUpdateAll={() => updateActions.updateAllInstalledMods(ctx)} onToggleMod={(mod) => nativeActions.toggleInstalledMod(ctx, mod)} onRemoveMod={(mod) => nativeActions.removeInstalledMod(ctx, mod)} onAddToProfile={(profile, mod) => profileActions.addModToProfile(ctx, profile, mod)} />
      {:else if activePanel === "profiles"}
        <LauncherProfilesPanel profiles={config.modProfiles} {installedMods} bind:profileName {busy} onCreateWithStyle={createProfile} onSaveEnabledWithStyle={saveEnabledProfile} onOpen={openProfile} onApply={applyProfile} onDelete={deleteProfile} />
      {:else if activePanel === "settings"}
        <LauncherSettingsPanel bind:config {detectedGame} {hasGameFolder} {healthItems} {modloaderStatus} {latestRelease} {needsPatcherUpdate} {launcherUpdate} {checkingLauncherUpdate} {installingLauncherUpdate} {busy} onUseDetected={() => nativeActions.useDetectedGame(ctx)} onSetLaunchMode={(mode) => nativeActions.setLaunchMode(ctx, mode)} onChooseGameFolder={() => nativeActions.chooseGameFolder(ctx)} onChooseExecutable={() => nativeActions.chooseExecutable(ctx)} onRepositoryChange={() => runtime.saveAndRefresh("Repository saved.")} onRunHealth={() => runtime.runHealthCheck()} onExportDiagnostics={() => nativeActions.exportDiagnostics(ctx)} onDebugLogsChange={() => runtime.saveAndRefresh(config.debugLogs ? "Debug logs enabled." : "Debug logs disabled.")} onCheckLauncherUpdate={() => checkLauncherUpdate(false)} onInstallLauncherUpdate={installLauncherUpdate} />
      {:else}
        <LauncherChangelogPanel {latestRelease} />
      {/if}
    </Card>
  {/if}

  {#if selectedProfile}
    <ProfileModal profile={selectedProfile} mods={selectedProfileMods} {updateSkins} {busy} onApply={applyProfile} onClose={closeProfile} onStyle={updateProfileStyle} onToggleMod={toggleProfileMod} />
  {/if}

  {#if showLauncherUpdatePrompt && launcherUpdate?.available}
    <LauncherUpdateModal update={launcherUpdate} installing={installingLauncherUpdate} onInstall={installLauncherUpdate} onDismiss={dismissLauncherUpdatePrompt} />
  {/if}
</main>
