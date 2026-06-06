<script lang="ts">
  import { now } from "$lib/now.svelte";
  import { formatElapsed, formatTime } from "$lib/utils";
  import { commands, events, type Interval } from "$lib/bindings";

  type Props = {
    fromMs: number;
    toMs: number;
    label?: string;
  };

  let { fromMs, toMs, label = "Intervals" }: Props = $props();

  let intervals = $state<Interval[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  async function fetchIntervals() {
    loading = true;
    error = null;
    const r = await commands.getIntervals(fromMs, toMs);
    if (r.status === "ok") intervals = r.data;
    else error = r.error;
    loading = false;
  }

  $effect(() => {
    fromMs; toMs;
    fetchIntervals();
  });

  $effect(() => {
    const unsub = events.intervalChanged.listen(() => fetchIntervals());
    return () => { unsub.then((fn) => fn()); };
  });

  let runningInterval = $derived(
    intervals.length > 0 && intervals.at(-1)!.end_ms === null
      ? intervals.at(-1)!
      : null
  );

  let displayIntervals = $derived([...intervals].reverse());

  let committedMs = $derived(
    intervals.reduce(
      (acc, iv) => (iv.end_ms !== null ? acc + (iv.end_ms - iv.start_ms) : acc),
      0
    )
  );

  let liveMs = $derived.by(() => {
    const r = runningInterval;
    if (!r) return 0;
    return Math.max(0, Math.min(now(), toMs) - Math.max(r.start_ms, fromMs));
  });

  let totalMs = $derived(committedMs + liveMs);
</script>

<div class="mt-6">
  <div class="flex items-baseline justify-between mb-2">
    <span class="text-sm font-medium">{label}</span>
    {#if !loading && !error}
      <span class="text-xs text-muted-foreground">
        {intervals.length}
        {intervals.length === 1 ? "interval" : "intervals"}
      </span>
    {/if}
  </div>

  {#if loading}
    <div class="space-y-1.5">
      {#each [0, 1, 2] as _}
        <div class="h-8 rounded-md bg-muted animate-pulse"></div>
      {/each}
    </div>
  {:else if error}
    <p class="text-sm text-destructive">{error}</p>
  {:else if intervals.length === 0}
    <p class="text-sm text-muted-foreground text-center py-4">No intervals recorded.</p>
  {:else}
    <div class="divide-y divide-border rounded-md border border-border overflow-hidden">
      {#each displayIntervals as interval (interval.id)}
        {@const isRunning = interval.end_ms === null}
        {@const durationMs = isRunning
          ? Math.max(0, Math.min(now(), toMs) - Math.max(interval.start_ms, fromMs))
          : interval.end_ms! - interval.start_ms}

        <div
          class="flex items-center gap-3 px-3 py-2 text-sm bg-card {isRunning
            ? 'bg-green-500/5'
            : ''}"
        >
          <span class="font-mono tabular-nums text-foreground w-20 shrink-0">
            {formatTime(interval.start_ms)}
          </span>

          <span class="text-muted-foreground select-none">→</span>

          <span
            class="font-mono tabular-nums w-20 shrink-0 {isRunning
              ? 'text-green-500'
              : 'text-foreground'}"
          >
            {#if isRunning}
              <span
                class="inline-block w-1.5 h-1.5 rounded-full bg-green-500 mr-1 mb-0.5 animate-pulse"
              ></span>live
            {:else}
              {formatTime(interval.end_ms!)}
            {/if}
          </span>

          <span class="font-mono tabular-nums text-muted-foreground ml-auto">
            {formatElapsed(durationMs)}
          </span>
        </div>
      {/each}
    </div>

    <div class="flex items-baseline justify-between mt-2 px-1">
      <span class="text-xs text-muted-foreground">Total</span>
      <span class="text-sm font-mono tabular-nums">{formatElapsed(totalMs)}</span>
    </div>
  {/if}
</div>
