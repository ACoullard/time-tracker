<script lang="ts">
  import { now } from "$lib/now.svelte";
  import { formatElapsed } from "$lib/utils";
  import { commands, type RangeTotal } from "$lib/bindings";

  type Props = {
    fromMs: number;
    toMs: number;
    isRunning: boolean;
    label?: string;
  };

  let { fromMs, toMs, isRunning, label = "Today" }: Props = $props();

  let range_total = $state<RangeTotal>({ total_ms: 0, most_recent: null });
  let error = $state<string | null>(null);

  $effect(() => {
    fromMs;
    toMs;
    isRunning;
    (async () => {
      const result = await commands.getRangeTotal(fromMs, toMs);
      if (result.status === "ok") {
        range_total = result.data;
      } else {
        error = result.error;
      }
    })();
  });

  let liveMs = $derived.by(() => {
    const m = range_total.most_recent;
    if (!m || m.end_ms !== null) return 0;
    const start = Math.max(m.start_ms, fromMs);
    const end = Math.min(now(), toMs);
    return Math.max(0, end - start);
  });

  let displayMs = $derived(range_total.total_ms + liveMs);
</script>

<div class="flex items-baseline justify-between">
  <span class="text-sm text-muted-foreground">{label}</span>
  <span class="text-xl font-mono tabular-nums">
    {error ?? formatElapsed(displayMs)}
  </span>
</div>
