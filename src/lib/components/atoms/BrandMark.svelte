<script lang="ts">
  export let compact = false;
</script>

<a class="group flex items-center gap-3 no-underline" href="/">
  <div class="grid h-11 w-11 place-items-center rounded-xl border border-border bg-primary text-lg font-black text-primary-foreground shadow-[0_0_28px_hsl(var(--primary)/.28)] transition-transform group-hover:-translate-y-0.5">
    OP
  </div>
  {#if !compact}
    <div class="leading-none">
      <p class="text-[0.68rem] font-black uppercase tracking-[0.22em] text-primary">Pirate Warriors 4</p>
      <p class="font-black text-foreground">Skin Hub</p>
    </div>
  {/if}
</a>
