import type { Character } from "$lib/api";
import type { InstalledMod, ModProfile } from "./types";

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
  return mods.filter((mod) => profile.enabledModKeys.includes(mod.modKey)).length;
}

export function profileHasMod(profile: ModProfile, mod: InstalledMod) {
  return profile.enabledModKeys.includes(mod.modKey);
}

export function profilePreviewMods(profile: ModProfile, mods: InstalledMod[]) {
  return mods.filter((mod) => profile.enabledModKeys.includes(mod.modKey) && mod.coverDataUrl);
}

export const profileIcons = [
  { value: "sparkles", label: "Sparkles", glyph: "✦" },
  { value: "bolt", label: "Bolt", glyph: "↯" },
  { value: "flame", label: "Flame", glyph: "◆" },
  { value: "test", label: "Test", glyph: "T" },
  { value: "clean", label: "Clean", glyph: "✓" }
];

export const profileColors = [
  { value: "violet", label: "Violet", className: "from-violet-500/35 to-fuchsia-500/20 border-violet-300/35 text-violet-50" },
  { value: "cyan", label: "Cyan", className: "from-cyan-500/35 to-sky-500/20 border-cyan-300/35 text-cyan-50" },
  { value: "emerald", label: "Emerald", className: "from-emerald-500/35 to-lime-500/20 border-emerald-300/35 text-emerald-50" },
  { value: "amber", label: "Amber", className: "from-amber-500/35 to-orange-500/20 border-amber-300/35 text-amber-50" },
  { value: "rose", label: "Rose", className: "from-rose-500/35 to-red-500/20 border-rose-300/35 text-rose-50" }
];

export function profileIcon(profile: ModProfile) {
  return profileIcons.find((item) => item.value === profile.icon) ?? profileIcons[0];
}

export function profileColor(profile: ModProfile) {
  return profileColors.find((item) => item.value === profile.color) ?? profileColors[0];
}
