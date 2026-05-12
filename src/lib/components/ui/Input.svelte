<script lang="ts">
  import { cn } from "$lib/utils/cn";

  export let value = "";
  export let type = "text";
  export let placeholder = "";
  export let autocomplete: HTMLInputElement["autocomplete"] | undefined = undefined;
  let className = "";
  export { className as class };
</script>

<input
  bind:value
  {type}
  {placeholder}
  {autocomplete}
  class={cn("flex h-10 w-full rounded-md border border-white/12 bg-background/55 px-3 py-2 text-sm text-foreground shadow-sm outline-none transition-colors placeholder:text-muted-foreground focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50", className)}
  on:input
/>
