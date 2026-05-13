import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import type { LauncherActionContext } from "./actionContext";
import type { HealthCheckItem, InstalledMod, LauncherConfig, LaunchMode } from "./types";

export async function chooseGameFolder(ctx: LauncherActionContext) {
  const selected = await open({ directory: true, multiple: false, title: "Select OPPW4 game folder" });
  if (typeof selected !== "string") return;
  ctx.setConfig({ ...ctx.getConfig(), gameFolder: selected });
  await ctx.saveAndRefresh("Game folder saved.");
}

export async function chooseExecutable(ctx: LauncherActionContext) {
  const selected = await open({
    directory: false,
    multiple: false,
    title: "Select OPPW4 executable",
    filters: [{ name: "Executable", extensions: ["exe", "AppImage", "sh", "x86_64"] }]
  });
  if (typeof selected !== "string") return;
  const parent = selected.replace(/[\\/][^\\/]+$/, "");
  const config = ctx.getConfig();
  ctx.setConfig({ ...config, gameExecutablePath: selected, gameFolder: config.gameFolder ?? parent });
  await ctx.saveAndRefresh("Executable saved.");
}

export async function useDetectedGame(ctx: LauncherActionContext) {
  const detectedGame = ctx.getDetectedGame();
  if (!detectedGame) return;
  const config = ctx.getConfig();
  ctx.setConfig({ ...config, gameFolder: detectedGame.gameFolder, gameExecutablePath: detectedGame.executablePath ?? config.gameExecutablePath });
  await ctx.saveAndRefresh("Detected Steam install saved.");
}

export async function setLaunchMode(ctx: LauncherActionContext, mode: LaunchMode) {
  ctx.setConfig({ ...ctx.getConfig(), launchMode: mode });
  await ctx.saveAndRefresh(`Launch mode set to ${mode}.`);
}

export async function launchGame(ctx: LauncherActionContext) {
  await ctx.runBusy(async () => {
    await ctx.save();
    await invoke("launch_game");
    await ctx.load();
    ctx.setMessage("Launch request sent.");
  }, "Could not launch game");
}

export async function installModloader(ctx: LauncherActionContext) {
  await ctx.runBusy(async () => {
    await ctx.save();
    ctx.setConfig(await invoke<LauncherConfig>("install_modloader"));
    await ctx.load();
    ctx.setMessage("Patcher installed.");
  }, "Could not install patcher");
}

export async function checkModloaderIntegrity(ctx: LauncherActionContext) {
  await ctx.runBusy(async () => {
    ctx.setConfig(await invoke<LauncherConfig>("check_modloader_integrity"));
    await ctx.load();
    ctx.setMessage("Patcher checked.");
  }, "Could not check patcher");
}

export async function toggleInstalledMod(ctx: LauncherActionContext, mod: InstalledMod) {
  await ctx.runBusy(async () => {
    await invoke("set_mod_enabled", { input: { path: mod.path, enabled: !mod.enabled } });
    await ctx.load();
    ctx.setMessage(`${mod.name} ${mod.enabled ? "disabled" : "enabled"}.`);
  }, "Could not update mod state");
}

export async function removeInstalledMod(ctx: LauncherActionContext, mod: InstalledMod) {
  const confirmed = window.confirm(`Remove "${mod.name}" from your mods folder?`);
  if (!confirmed) return;
  await ctx.runBusy(async () => {
    await invoke("remove_installed_mod", { input: { path: mod.path } });
    await ctx.load();
    ctx.setMessage(`${mod.name} removed.`);
  }, "Could not remove mod");
}

export async function importExternalZip(ctx: LauncherActionContext) {
  const selected = await open({ directory: false, multiple: false, title: "Import external mod ZIP", filters: [{ name: "ZIP archive", extensions: ["zip"] }] });
  if (typeof selected !== "string") return;
  await ctx.runBusy(async () => {
    await invoke("import_external_zip", { input: { path: selected } });
    await ctx.load();
    ctx.setMessage("External ZIP imported.");
  }, "Could not import ZIP");
}

export async function runHealthCheck(setHealthItems: (items: HealthCheckItem[]) => void) {
  setHealthItems(await invoke<HealthCheckItem[]>("run_health_check"));
}

export async function exportDiagnostics(ctx: LauncherActionContext) {
  const selected = await save({
    title: "Export diagnostics",
    defaultPath: "oppw4-launcher-diagnostics.zip",
    filters: [{ name: "ZIP archive", extensions: ["zip"] }]
  });
  if (typeof selected !== "string") return;
  await ctx.runBusy(async () => {
    await invoke("export_diagnostics", { input: { path: selected } });
    ctx.setMessage("Diagnostics exported.");
  }, "Could not export diagnostics");
}
