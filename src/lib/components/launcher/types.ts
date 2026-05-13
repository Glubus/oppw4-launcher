import type { Skin } from "$lib/api";

export type LaunchMode = "steam" | "executable";
export type ActiveLauncherPanel = "mods" | "profiles" | "settings" | "changelog";

export type InstalledFile = {
  relativePath: string;
  backupPath?: string | null;
};

export type ModProfile = {
  id: string;
  name: string;
  enabledModKeys: string[];
};

export type LauncherConfig = {
  launchMode: LaunchMode;
  gameFolder?: string | null;
  gameExecutablePath?: string | null;
  modloaderRepo: string;
  modloaderRelease?: string | null;
  modloaderSha256?: string | null;
  latestModloaderSha256?: string | null;
  latestModloaderSha256CheckedAt?: string | null;
  installedFiles: InstalledFile[];
  lastLaunchAt?: string | null;
  modProfiles: ModProfile[];
};

export const defaultLauncherConfig: LauncherConfig = {
  launchMode: "steam",
  gameFolder: null,
  gameExecutablePath: null,
  modloaderRepo: "Glubus/oppw4-patcher",
  modloaderRelease: null,
  modloaderSha256: null,
  latestModloaderSha256: null,
  latestModloaderSha256CheckedAt: null,
  installedFiles: [],
  lastLaunchAt: null,
  modProfiles: []
};

export type DetectedGame = {
  gameFolder: string;
  executablePath?: string | null;
  source: string;
};

export type ReleaseInfo = {
  tagName: string;
  name?: string | null;
  body?: string | null;
  htmlUrl: string;
  prerelease: boolean;
  assetName?: string | null;
};

export type InstalledMod = {
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

export type LauncherState = {
  config: LauncherConfig;
  detectedGame?: DetectedGame | null;
  modloaderStatus: string;
  latestRelease?: ReleaseInfo | null;
  needsPatcherUpdate: boolean;
  localModloaderSha256?: string | null;
  remoteModloaderSha256?: string | null;
  installedMods: InstalledMod[];
};

export type UpdateSkinMap = Record<string, Skin>;
