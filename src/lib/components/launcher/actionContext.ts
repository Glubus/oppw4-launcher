import type { Skin } from "$lib/api";
import type { DetectedGame, InstalledMod, LauncherConfig, ModProfile } from "./types";

export type LauncherActionContext = {
  getConfig: () => LauncherConfig;
  setConfig: (config: LauncherConfig) => void;
  getDetectedGame: () => DetectedGame | null;
  getInstalledMods: () => InstalledMod[];
  getProfileName: () => string;
  setProfileName: (value: string) => void;
  getSelectedProfile: () => ModProfile | null;
  setSelectedProfile: (value: ModProfile | null) => void;
  getUpdateSkins: () => Record<string, Skin>;
  setUpdateSkins: (value: Record<string, Skin>) => void;
  setCheckingUpdates: (value: boolean) => void;
  setUpdatingAll: (value: boolean) => void;
  setError: (value: string) => void;
  setMessage: (value: string) => void;
  load: () => Promise<void>;
  save: () => Promise<void>;
  saveAndRefresh: (success: string) => Promise<void>;
  runBusy: (action: () => Promise<void>, fallback: string) => Promise<void>;
};
