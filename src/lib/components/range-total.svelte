<script lang="ts">
  import { now } from "$lib/now.svelte";
  import { formatElapsed } from "$lib/utils";
  import { invoke } from "@tauri-apps/api/core";

  type Interval = { id: number; start_ms: number; end_ms: number | null };
  type RangeTotalData = { total_ms: number; most_recent: Interval | null };

  type Props = {
    fromMs: number;
    toMs: number;
    isRunning: boolean;
    label?: string;
  };

  let { fromMs, toMs, isRunning, label = "Today" }: Props = $props();

  let range_total = $state<RangeTotalData>({ total_ms: 0, most_recent: null });
  let error = $state<string | null>(null);

  $effect(() => {
    fromMs;
    toMs;
    isRunning;
    (async () => {
      try {
        range_total = await invoke<RangeTotalData>("get_range_total", { fromMs, toMs });
      } catch (e) {
        error = String(e);
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
