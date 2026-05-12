<script lang="ts">
  import { cn } from "$lib/utils/cn";
  let className = "";
  export { className as class };
</script>

<label class={cn("grid gap-2 text-sm font-medium leading-none text-foreground", className)}>
  <slot />
</label>
