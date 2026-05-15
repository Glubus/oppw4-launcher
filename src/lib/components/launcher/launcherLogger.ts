import { invoke } from "@tauri-apps/api/core";
import { toastStore } from "$lib/stores/toasts";
import type { LauncherConfig } from "./types";

export type LauncherLogLevel = "success" | "error" | "debug";

export function createLauncherLogger(getConfig: () => LauncherConfig, isDesktop: () => boolean) {
  const fileStamp = new Date().toISOString().replace(/[-:]/g, "").replace(/\..+$/, "").replace("T", "-");

  function write(level: LauncherLogLevel, message: string, debug = false) {
    if (!isDesktop()) return;
    void invoke("write_launcher_log", {
      input: {
        level,
        message,
        fileStamp,
        debug
      }
    });
  }

  return {
    success(message: string) {
      if (!message) return;
      toastStore.push(message, "success");
      write("success", message);
    },
    error(message: string) {
      if (!message) return;
      toastStore.push(message, "error");
      write("error", message);
    },
    debug(message: string) {
      if (getConfig().debugLogs) write("debug", message, true);
    }
  };
}
