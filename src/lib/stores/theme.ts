import { browser } from "$app/environment";
import { writable } from "svelte/store";

export type Theme = "dark" | "light";

const storageKey = "oppw4-theme";
const initialTheme: Theme = browser && localStorage.getItem(storageKey) === "light" ? "light" : "dark";

function applyTheme(theme: Theme) {
  if (!browser) return;
  document.documentElement.dataset.theme = theme;
  localStorage.setItem(storageKey, theme);
}

function createThemeStore() {
  const store = writable<Theme>(initialTheme);

  if (browser) applyTheme(initialTheme);

  return {
    subscribe: store.subscribe,
    set(theme: Theme) {
      applyTheme(theme);
      store.set(theme);
    },
    toggle() {
      store.update((theme) => {
        const next = theme === "dark" ? "light" : "dark";
        applyTheme(next);
        return next;
      });
    }
  };
}

export const themeStore = createThemeStore();
