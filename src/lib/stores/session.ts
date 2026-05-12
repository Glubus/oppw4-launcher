import { browser } from "$app/environment";
import { writable } from "svelte/store";
import type { Session } from "$lib/api";

const storageKey = "oppw4-session";

function createSessionStore() {
  const initial = browser ? localStorage.getItem(storageKey) : null;
  let parsedInitial: Session | null = null;

  if (initial) {
    try {
      parsedInitial = JSON.parse(initial) as Session;
    } catch {
      localStorage.removeItem(storageKey);
    }
  }

  const store = writable<Session | null>(parsedInitial);

  return {
    subscribe: store.subscribe,
    set(session: Session | null) {
      if (browser) {
        if (session) localStorage.setItem(storageKey, JSON.stringify(session));
        else localStorage.removeItem(storageKey);
      }
      store.set(session);
    }
  };
}

export const session = createSessionStore();
