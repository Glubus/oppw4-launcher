import type { Character } from "$lib/api";
import type { InstalledMod, ModProfile, PotentialOverlapGroup } from "./types";

export function errorMessage(err: unknown, fallback: string) {
  return err instanceof Error ? err.message : typeof err === "string" ? err : fallback;
}

export function modInitials(name: string) {
  return name
    .replace(/\.(zip|disabled)$/i, "")
    .split(/[\s._-]+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((part) => part[0])
    .join("")
    .toUpperCase() || "MOD";
}

export function modPageHref(mod: InstalledMod) {
  if (mod.slug) return `/skins/${encodeURIComponent(mod.slug)}`;
  if (!mod.sourceUrl) return null;
  try {
    const url = new URL(mod.sourceUrl);
    return url.pathname.startsWith("/skins/") ? url.pathname : mod.sourceUrl;
  } catch {
    return mod.sourceUrl;
  }
}

export function slugFromSourceUrl(value?: string | null) {
  if (!value) return null;
  try {
    const parts = new URL(value).pathname.split("/").filter(Boolean);
    return parts[0] === "skins" ? parts[1] : null;
  } catch {
    return null;
  }
}

export function sortInstalledMods(mods: InstalledMod[], value: string) {
  return [...mods].sort((a, b) => {
    if (value === "popular") return a.name.localeCompare(b.name);
    if (value === "viewed") return (a.characterName || "").localeCompare(b.characterName || "") || a.name.localeCompare(b.name);
    return a.name.localeCompare(b.name);
  });
}

export function localCharacters(mods: InstalledMod[]): Character[] {
  const bySlug = new Map<string, Character>();
  for (const mod of mods) {
    const slug = installedModCharacterSlug(mod);
    if (!slug || bySlug.has(slug)) continue;
    bySlug.set(slug, { id: slug, slug, displayName: mod.characterName || slug, isDlc: false, pack: "Installed" });
  }
  return [...bySlug.values()].sort((a, b) => a.displayName.localeCompare(b.displayName));
}

export function installedModCharacterSlug(mod: InstalledMod) {
  return mod.characterSlug || mod.characterName?.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "";
}

export function profileModCount(profile: ModProfile, mods: InstalledMod[]) {
  return profileMods(profile, mods).length;
}

export function profileHasMod(profile: ModProfile, mod: InstalledMod) {
  return profile.enabledModKeys.includes(mod.modKey);
}

export function profilePreviewMods(profile: ModProfile, mods: InstalledMod[]) {
  return profileMods(profile, mods).filter((mod) => mod.coverDataUrl);
}

export function profileMods(profile: ModProfile, mods: InstalledMod[]) {
  return mods.filter((mod) => profile.enabledModKeys.includes(mod.modKey));
}

export function potentialOverlaps(mods: InstalledMod[]): PotentialOverlapGroup[] {
  const groups = new Map<string, InstalledMod[]>();
  for (const mod of mods) {
    const key = overlapKey(mod);
    if (!key) continue;
    groups.set(key, [...(groups.get(key) ?? []), mod]);
  }
  return [...groups.entries()]
    .filter(([, groupMods]) => groupMods.length > 1)
    .map(([key, groupMods]) => ({
      key,
      characterLabel: groupMods[0].characterName || groupMods[0].characterSlug || "Unknown character",
      modType: groupMods[0].modType || "mod",
      mods: groupMods
    }));
}

export function enabledPotentialOverlaps(mods: InstalledMod[]) {
  return potentialOverlaps(mods.filter((mod) => mod.enabled));
}

export function overlapModPaths(groups: PotentialOverlapGroup[]) {
  return new Set(groups.flatMap((group) => group.mods.map((mod) => mod.path)));
}

export function overlapSummaryForMod(mod: InstalledMod, groups: PotentialOverlapGroup[]) {
  const group = groups.find((item) => item.mods.some((groupMod) => groupMod.path === mod.path));
  if (!group) return "";
  const others = group.mods.filter((groupMod) => groupMod.path !== mod.path).map((groupMod) => groupMod.name);
  return others.length ? `May overlap with: ${others.join(", ")}` : "Potential overlap";
}

function overlapKey(mod: InstalledMod) {
  const character = normalizeOverlapPart(mod.characterSlug || mod.characterName);
  const modType = normalizeOverlapPart(mod.modType);
  return character && modType ? `${character}:${modType}` : "";
}

function normalizeOverlapPart(value?: string | null) {
  const normalized = value?.trim().toLowerCase();
  return normalized || "";
}

export const profileIcons = [
  { value: "sparkles", label: "Sparkles" },
  { value: "bolt", label: "Bolt" },
  { value: "flame", label: "Flame" },
  { value: "test", label: "Test" },
  { value: "clean", label: "Clean" },
  { value: "gamepad", label: "Gamepad" }
];

export const profileColors = [
  { value: "violet", label: "Violet", from: "rgba(139, 92, 246, 0.36)", to: "rgba(217, 70, 239, 0.18)", border: "rgba(196, 181, 253, 0.38)", text: "rgb(245, 243, 255)" },
  { value: "cyan", label: "Cyan", from: "rgba(6, 182, 212, 0.34)", to: "rgba(14, 165, 233, 0.18)", border: "rgba(103, 232, 249, 0.38)", text: "rgb(236, 254, 255)" },
  { value: "emerald", label: "Emerald", from: "rgba(16, 185, 129, 0.34)", to: "rgba(132, 204, 22, 0.16)", border: "rgba(110, 231, 183, 0.38)", text: "rgb(236, 253, 245)" },
  { value: "amber", label: "Amber", from: "rgba(245, 158, 11, 0.34)", to: "rgba(249, 115, 22, 0.18)", border: "rgba(252, 211, 77, 0.40)", text: "rgb(255, 251, 235)" },
  { value: "rose", label: "Rose", from: "rgba(244, 63, 94, 0.34)", to: "rgba(239, 68, 68, 0.18)", border: "rgba(253, 164, 175, 0.40)", text: "rgb(255, 241, 242)" }
];

export function profileIcon(profile: ModProfile) {
  return profileIcons.find((item) => item.value === profile.icon) ?? profileIcons[0];
}

export function profileColor(profile: ModProfile) {
  return profileColors.find((item) => item.value === profile.color) ?? profileColors[0];
}
