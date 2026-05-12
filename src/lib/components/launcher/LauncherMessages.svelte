<script lang="ts">
  export let error = "";
  export let message = "";
</script>

{#if error}
  <div class="rounded-xl border border-destructive/40 bg-destructive/15 px-4 py-3 text-sm text-red-100 shadow-lg">{error}</div>
{/if}
{#if message}
  <div class="rounded-xl border border-emerald-400/30 bg-emerald-500/15 px-4 py-3 text-sm font-bold text-emerald-200 shadow-lg">{message}</div>
{/if}
