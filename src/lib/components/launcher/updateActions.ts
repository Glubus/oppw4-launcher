import { invoke } from "@tauri-apps/api/core";
import { apiFetch, type Skin } from "$lib/api";
import type { LauncherActionContext } from "./actionContext";
import { errorMessage, slugFromSourceUrl } from "./helpers";
import type { InstalledMod } from "./types";

export async function checkInstalledUpdates(ctx: LauncherActionContext, mods: InstalledMod[]) {
  ctx.setCheckingUpdates(true);
  const nextUpdates: Record<string, Skin> = {};
  try {
    await Promise.all(mods.map(async (mod) => {
      const slug = mod.slug || slugFromSourceUrl(mod.sourceUrl);
      if (!slug || !mod.version) return;
      try {
        const data = await apiFetch<{ skin: Skin }>(`/skins/${encodeURIComponent(slug)}`);
        if (data.skin.files?.[0] && data.skin.version !== mod.version) nextUpdates[mod.path] = data.skin;
      } catch {}
    }));
    ctx.setUpdateSkins(nextUpdates);
  } finally {
    ctx.setCheckingUpdates(false);
  }
}

export async function updateAllInstalledMods(ctx: LauncherActionContext) {
  const updates = ctx.getInstalledMods()
    .map((mod) => ctx.getUpdateSkins()[mod.path])
    .filter((skin): skin is Skin => Boolean(skin?.files?.[0]));
  if (!updates.length) return;

  ctx.setUpdatingAll(true);
  ctx.setError("");
  ctx.setMessage("");
  try {
    for (const skin of updates) {
      const file = skin.files![0];
      await invoke("install_hosted_mod", { input: { fileId: file.id, fileName: file.fileName } });
    }
    await ctx.load();
    ctx.setMessage(`Updated ${updates.length} mod${updates.length > 1 ? "s" : ""}.`);
  } catch (err) {
    ctx.setError(errorMessage(err, "Could not update installed mods"));
  } finally {
    ctx.setUpdatingAll(false);
  }
}
