<script lang="ts">
  import { fly } from "svelte/transition";
  import XIcon from "$lib/components/atoms/XIcon.svelte";
  import { toastStore } from "$lib/stores/toasts";
</script>

<div class="pointer-events-none fixed bottom-4 right-4 z-50 grid w-[calc(100vw-2rem)] max-w-sm gap-2 sm:bottom-6 sm:right-6">
  {#each $toastStore as toast (toast.id)}
    <div
      class="pointer-events-auto flex items-start gap-3 rounded-lg border px-4 py-3 text-sm font-bold shadow-[0_18px_55px_rgba(0,0,0,0.38)] backdrop-blur-md
        {toast.tone === 'error'
          ? 'border-destructive/45 bg-destructive/20 text-red-100'
          : toast.tone === 'success'
            ? 'border-emerald-400/35 bg-emerald-500/18 text-emerald-100'
            : 'border-white/14 bg-card/95 text-foreground'}"
      in:fly={{ x: 28, duration: 180 }}
      out:fly={{ x: 28, duration: 170 }}
    >
      <span class="min-w-0 flex-1 leading-5">{toast.message}</span>
      <span class="relative mt-0.5 grid h-6 w-6 shrink-0 place-items-center text-current/72" aria-hidden="true">
        <svg class="h-5 w-5 -rotate-90" viewBox="0 0 20 20" fill="none">
          <circle class="text-current/18" cx="10" cy="10" r="7.5" stroke="currentColor" stroke-width="2" />
          <circle class="toast-timer-ring" cx="10" cy="10" r="7.5" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
        </svg>
      </span>
      <button
        class="grid h-6 w-6 shrink-0 place-items-center rounded-md text-current/72 transition hover:bg-white/10 hover:text-current focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        type="button"
        aria-label="Dismiss notification"
        on:click={() => toastStore.dismiss(toast.id)}
      >
        <XIcon class="h-4 w-4" />
      </button>
    </div>
  {/each}
</div>

<style>
  .toast-timer-ring {
    stroke-dasharray: 47.12;
    stroke-dashoffset: 0;
    animation: toast-timer 3.2s linear forwards;
  }

  @keyframes toast-timer {
    to {
      stroke-dashoffset: 47.12;
    }
  }
</style>
