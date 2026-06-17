<script lang="ts">
  import { page } from "$app/stores";
  import { Timer, ChartBar, Menu } from "@lucide/svelte";
  import { cn } from "$lib/utils";

  const navItems = [
    { href: "/", icon: Timer, label: "Timer" },
    { href: "/stats", icon: ChartBar, label: "Stats" },
  ] as const;
</script>

<nav
  aria-label="Main navigation"
  class="group fixed top-3 left-10 z-50 overflow-hidden rounded-full border bg-background shadow-sm
         h-11 w-11 hover:w-32 transition-[width] duration-200 ease-out"
>
  <!-- Hamburger: fades out immediately on hover, reappears after collapse -->
  <div
    class="absolute inset-0 flex items-center justify-center text-muted-foreground pointer-events-none
           transition-opacity duration-80 delay-120
           group-hover:opacity-0 group-hover:delay-0"
  >
    <Menu size={22} />
  </div>

  <!-- Nav items: fade in after hamburger is gone, fade out quickly on collapse -->
  <div
    class="flex h-full items-center gap-5 px-4
           opacity-0 transition-opacity duration-60
           group-hover:opacity-100 group-hover:duration-120 group-hover:delay-60"
  >
    {#each navItems as { href, icon: Icon, label }}
      <a
        {href}
        aria-label={label}
        class={cn(
          "flex items-center rounded-full p-2 transition-colors shrink-0",
          $page.url.pathname === href
            ? "bg-muted text-foreground"
            : "text-muted-foreground hover:bg-muted hover:text-foreground",
        )}
      >
        <Icon size={20} />
      </a>
    {/each}
  </div>
</nav>
