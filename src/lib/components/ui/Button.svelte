<script lang="ts">
  import { cn } from "$lib/utils/cn";

  export let href: string | undefined = undefined;
  export let type: "button" | "submit" | "reset" = "button";
  export let variant: "default" | "secondary" | "ghost" | "outline" | "destructive" = "default";
  export let size: "sm" | "md" | "lg" | "icon" = "md";
  export let disabled = false;
  let className = "";
  export { className as class };

  $: classes = cn(
    "inline-flex items-center justify-center gap-2 rounded-md text-sm font-bold transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50",
    variant === "default" && "bg-primary text-primary-foreground shadow-[0_10px_28px_rgba(255,255,255,0.12)] hover:bg-primary/90",
    variant === "secondary" && "bg-secondary text-secondary-foreground hover:bg-secondary/80",
    variant === "ghost" && "text-foreground/78 hover:bg-white/8 hover:text-foreground",
    variant === "outline" && "border border-white/14 bg-background/55 text-foreground hover:bg-white/10 hover:text-foreground",
    variant === "destructive" && "bg-destructive text-destructive-foreground hover:bg-destructive/90",
    size === "sm" && "h-8 px-3",
    size === "md" && "h-10 px-4 py-2",
    size === "lg" && "h-11 px-6",
    size === "icon" && "h-10 w-10",
    className
  );
</script>

{#if href}
  <a {href} class={classes}><slot /></a>
{:else}
  <button {type} {disabled} class={classes} on:click><slot /></button>
{/if}
