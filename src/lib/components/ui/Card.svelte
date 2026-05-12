<script lang="ts">
  import { cn } from "$lib/utils/cn";
  let className = "";
  export { className as class };
</script>

<section class={cn("rounded-lg border border-white/10 bg-card/92 text-card-foreground shadow-[0_18px_60px_rgba(0,0,0,0.28)] backdrop-blur-md", className)}>
  <slot />
</section>
