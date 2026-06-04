<script lang="ts">
  import { now } from "$lib/now.svelte";
  import { formatElapsed } from "$lib/utils";
  import { commands, type RangeTotal } from "$lib/bindings";

  type Props = {
    fromMs: number;
    toMs: number;
    isRunning: boolean;
    goalMs: number | null;
  };

  let { fromMs, toMs, isRunning, goalMs }: Props = $props();

  const r = 40;
  const circumference = 2 * Math.PI * r;

  let rangeTotal = $state<RangeTotal>({ total_ms: 0, most_recent: null });

  $effect(() => {
    fromMs; toMs; isRunning;
    (async () => {
      const result = await commands.getRangeTotal(fromMs, toMs);
      if (result.status === "ok") rangeTotal = result.data;
    })();
  });

  let liveMs = $derived.by(() => {
    const m = rangeTotal.most_recent;
    if (!m || m.end_ms !== null) return 0;
    const start = Math.max(m.start_ms, fromMs);
    const end = Math.min(now(), toMs);
    return Math.max(0, end - start);
  });

  let totalMs = $derived(rangeTotal.total_ms + liveMs);
  let progress = $derived(goalMs !== null && goalMs > 0 ? Math.min(1, totalMs / goalMs) : 0);
  let offset = $derived(circumference * (1 - progress));
</script>

<div class="relative flex items-center justify-center">
  <svg viewBox="0 0 100 100" class="-rotate-90 w-full h-full">
    <circle
      cx="50"
      cy="50"
      r={r}
      stroke-width="8"
      class="fill-none stroke-muted-foreground/15"
    />
    {#if goalMs !== null}
      <circle
        cx="50"
        cy="50"
        r={r}
        stroke-width="8"
        class="fill-none stroke-green-500"
        stroke-linecap="round"
        stroke-dasharray={circumference}
        stroke-dashoffset={offset}
        style="transition: stroke-dashoffset 0.4s ease"
      />
    {/if}
  </svg>

  <!-- centered label — not rotated, layered over the SVG -->
  <div class="absolute flex flex-col items-center pointer-events-none select-none">
    <span class="text-3xl font-mono tabular-nums leading-tight">{formatElapsed(totalMs)}</span>
    <span class="text-muted-foreground mt-0.5">Today</span>
    {#if goalMs !== null}
      <span class="text-sm text-muted-foreground">{Math.round(progress * 100)}%</span>
    {/if}
  </div>
</div>
