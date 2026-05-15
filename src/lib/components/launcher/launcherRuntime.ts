import { invoke } from "@tauri-apps/api/core";
import { errorMessage } from "./helpers";
import * as nativeActions from "./nativeActions";
import type { DetectedGame, HealthCheckItem, InstalledMod, LauncherConfig, LauncherState, ReleaseInfo } from "./types";

type LauncherLogger = {
  success: (message: string) => void;
  error: (message: string) => void;
  debug: (message: string) => void;
};

export type LauncherRuntimeState = {
  config: LauncherConfig;
  detectedGame: DetectedGame | null;
  installedMods: InstalledMod[];
  modloaderStatus: string;
  latestRelease: ReleaseInfo | null;
  needsPatcherUpdate: boolean;
  localModloaderSha256: string | null;
  remoteModloaderSha256: string | null;
};

export type LauncherRuntimeConfig = {
  getConfig: () => LauncherConfig;
  setConfig: (config: LauncherConfig) => void;
  setState: (state: LauncherRuntimeState) => void;
  setLoading: (value: boolean) => void;
  setBusy: (value: boolean) => void;
  setHealthItems: (items: HealthCheckItem[]) => void;
  logger: LauncherLogger;
};

export function createLauncherRuntime(options: LauncherRuntimeConfig) {
  async function load() {
    options.setLoading(true);
    try {
      const state = await invoke<LauncherState>("get_launcher_state");
      options.setState({
        config: state.config,
        detectedGame: state.detectedGame ?? null,
        installedMods: state.installedMods ?? [],
        modloaderStatus: state.modloaderStatus,
        latestRelease: state.latestRelease ?? null,
        needsPatcherUpdate: state.needsPatcherUpdate,
        localModloaderSha256: state.localModloaderSha256 ?? null,
        remoteModloaderSha256: state.remoteModloaderSha256 ?? null
      });
      options.logger.debug(`state loaded; mods=${state.installedMods?.length ?? 0}; profiles=${state.config.modProfiles.length}; modloader=${state.modloaderStatus}; release=${state.config.modloaderRelease ?? "none"}`);
    } catch (err) {
      options.logger.error(errorMessage(err, "Could not load launcher state"));
    } finally {
      options.setLoading(false);
    }
  }

  async function save() {
    const config = await invoke<LauncherConfig>("save_launcher_config", { config: options.getConfig() });
    options.setConfig(config);
    options.logger.debug(`config saved; launchMode=${config.launchMode}; gameFolder=${config.gameFolder ?? "none"}; debugLogs=${config.debugLogs}`);
  }

  async function saveAndRefresh(success: string) {
    try {
      await save();
      await load();
      options.logger.success(success);
    } catch (err) {
      options.logger.error(errorMessage(err, "Could not save config"));
    }
  }

  async function runBusy(action: () => Promise<void>, fallback: string) {
    options.setBusy(true);
    try {
      await action();
    } catch (err) {
      options.logger.error(errorMessage(err, fallback));
    } finally {
      options.setBusy(false);
    }
  }

  async function runHealthCheck() {
    try {
      await nativeActions.runHealthCheck(options.setHealthItems);
      options.logger.success("Health check complete.");
    } catch (err) {
      options.logger.error(errorMessage(err, "Could not run health check"));
    }
  }

  return { load, save, saveAndRefresh, runBusy, runHealthCheck };
}
