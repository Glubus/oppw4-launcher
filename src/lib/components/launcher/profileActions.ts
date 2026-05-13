import { invoke } from "@tauri-apps/api/core";
import type { LauncherActionContext } from "./actionContext";
import type { InstalledMod, ModProfile } from "./types";

export async function createProfile(ctx: LauncherActionContext) {
  const name = ctx.getProfileName().trim();
  if (!name) return;
  const config = ctx.getConfig();
  if (config.modProfiles.some((profile) => profile.name.toLowerCase() === name.toLowerCase())) {
    ctx.setError("A profile with this name already exists.");
    return;
  }
  ctx.setConfig({ ...config, modProfiles: [...config.modProfiles, { id: `profile-${Date.now()}`, name, icon: "sparkles", color: "violet", enabledModKeys: [] }] });
  ctx.setProfileName("");
  await ctx.saveAndRefresh("Profile created.");
}

export async function saveCurrentProfile(ctx: LauncherActionContext) {
  const name = ctx.getProfileName().trim();
  if (!name) return;
  const config = ctx.getConfig();
  const existingIndex = config.modProfiles.findIndex((profile) => profile.name.toLowerCase() === name.toLowerCase());
  const nextProfile = {
    id: existingIndex >= 0 ? config.modProfiles[existingIndex].id : `profile-${Date.now()}`,
    name,
    icon: existingIndex >= 0 ? config.modProfiles[existingIndex].icon : "sparkles",
    color: existingIndex >= 0 ? config.modProfiles[existingIndex].color : "violet",
    enabledModKeys: ctx.getInstalledMods().filter((mod) => mod.enabled).map((mod) => mod.modKey)
  };
  ctx.setConfig({
    ...config,
    modProfiles: existingIndex >= 0 ? config.modProfiles.map((profile, index) => (index === existingIndex ? nextProfile : profile)) : [...config.modProfiles, nextProfile]
  });
  ctx.setProfileName("");
  await ctx.saveAndRefresh("Profile saved.");
}

export async function addModToProfile(ctx: LauncherActionContext, profile: ModProfile, mod: InstalledMod) {
  if (profile.enabledModKeys.includes(mod.modKey)) {
    ctx.setMessage(`${mod.name} is already in ${profile.name}.`);
    return;
  }
  const config = ctx.getConfig();
  ctx.setConfig({
    ...config,
    modProfiles: config.modProfiles.map((item) => item.id === profile.id ? { ...item, enabledModKeys: [...item.enabledModKeys, mod.modKey] } : item)
  });
  await ctx.saveAndRefresh(`${mod.name} added to ${profile.name}.`);
}

export async function applyProfile(ctx: LauncherActionContext, profile: ModProfile) {
  await ctx.runBusy(async () => {
    await ctx.save();
    await invoke("apply_mod_profile", { input: { profileId: profile.id } });
    await ctx.load();
    ctx.setMessage(`${profile.name} applied.`);
  }, "Could not apply profile");
}

export async function deleteProfile(ctx: LauncherActionContext, profile: ModProfile) {
  const config = ctx.getConfig();
  ctx.setConfig({ ...config, modProfiles: config.modProfiles.filter((item) => item.id !== profile.id) });
  if (ctx.getSelectedProfile()?.id === profile.id) ctx.setSelectedProfile(null);
  await ctx.saveAndRefresh("Profile deleted.");
}

export async function updateProfileStyle(ctx: LauncherActionContext, profile: ModProfile, icon: string, color: string) {
  const config = ctx.getConfig();
  ctx.setConfig({
    ...config,
    modProfiles: config.modProfiles.map((item) => item.id === profile.id ? { ...item, icon, color } : item)
  });
  await ctx.saveAndRefresh("Profile style saved.");
}
