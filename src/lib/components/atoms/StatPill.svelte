<script lang="ts">
  export let label = "";
  export let value: string | number;
  export let tone: "primary" | "accent" | "neutral" = "neutral";
</script>

<span class="badge gap-1 border-base-content/10 px-2.5 py-1.5 font-bold {tone === 'primary' ? 'badge-primary' : tone === 'accent' ? 'badge-accent' : 'badge-ghost'}">
  <span>{value}</span>
  {#if label}
    <span class="opacity-70">{label}</span>
  {/if}
</span>
