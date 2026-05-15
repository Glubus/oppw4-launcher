import { invoke } from "@tauri-apps/api/core";

function isTauriRuntime() {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

function shouldIgnoreClick(event: MouseEvent) {
  return event.defaultPrevented || event.button !== 0;
}

function isExternalHttpUrl(url: URL) {
  return (url.protocol === "http:" || url.protocol === "https:") && url.origin !== window.location.origin;
}

export function installExternalLinkHandler() {
  if (!isTauriRuntime()) return () => {};

  const handleClick = (event: MouseEvent) => {
    if (shouldIgnoreClick(event)) return;
    const target = event.target;
    if (!(target instanceof Element)) return;

    const anchor = target.closest("a[href]");
    if (!(anchor instanceof HTMLAnchorElement)) return;

    const url = new URL(anchor.href);
    if (!isExternalHttpUrl(url)) return;

    event.preventDefault();
    void invoke("open_external_url", { url: url.toString() });
  };

  document.addEventListener("click", handleClick, true);
  return () => document.removeEventListener("click", handleClick, true);
}
