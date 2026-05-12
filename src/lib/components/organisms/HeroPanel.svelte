<script lang="ts">
  export let totalSkins = 0;
  export let totalCharacters = 0;
</script>

<section class="relative overflow-hidden rounded-box border border-base-content/10 bg-neutral px-6 py-10 text-neutral-content shadow-2xl md:px-10">
  <div class="absolute inset-0 bg-[radial-gradient(circle_at_10%_10%,rgba(245,188,66,.35),transparent_28%),radial-gradient(circle_at_90%_0%,rgba(60,181,190,.28),transparent_34%)]"></div>
  <div class="absolute inset-x-0 bottom-0 h-24 bg-gradient-to-t from-black/35 to-transparent"></div>
  <div class="relative grid gap-8 lg:grid-cols-[1fr_360px] lg:items-end">
    <div>
      <p class="mb-3 text-xs font-black uppercase tracking-[0.3em] text-primary">Community mod discovery</p>
      <h1 class="max-w-4xl text-5xl font-black leading-[0.92] md:text-7xl">Browse skins by character, source, stats, and creator credit.</h1>
      <p class="mt-5 max-w-2xl text-base leading-7 text-neutral-content/75">A fast archive for OPPW4 skins: reposts stay credited, creator uploads publish cleanly, and every external obtain link is tracked without pretending the file belongs to us.</p>
    </div>
    <div class="grid gap-3 rounded-box border border-white/10 bg-white/10 p-4 backdrop-blur">
      <div class="stat rounded-box bg-black/20">
        <div class="stat-title text-neutral-content/60">Skins indexed</div>
        <div class="stat-value text-primary">{totalSkins}</div>
      </div>
      <div class="stat rounded-box bg-black/20">
        <div class="stat-title text-neutral-content/60">Playable roster</div>
        <div class="stat-value text-accent">{totalCharacters}</div>
      </div>
    </div>
  </div>
</section>
