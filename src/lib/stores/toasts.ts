import { writable } from "svelte/store";

export type ToastTone = "default" | "error" | "success";

export type Toast = {
  id: string;
  message: string;
  tone: ToastTone;
};

const toasts = writable<Toast[]>([]);

export const toastStore = {
  subscribe: toasts.subscribe,
  push(message: string, tone: ToastTone = "default") {
    const id = crypto.randomUUID();
    toasts.update((items) => [...items, { id, message, tone }]);
    setTimeout(() => {
      toasts.update((items) => items.filter((item) => item.id !== id));
    }, 3200);
  },
  dismiss(id: string) {
    toasts.update((items) => items.filter((item) => item.id !== id));
  }
};
