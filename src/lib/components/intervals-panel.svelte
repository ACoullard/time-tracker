<script lang="ts">
  import { Time } from "@internationalized/date";
  import { Trash2 } from "@lucide/svelte";
  import { now } from "$lib/now.svelte";
  import { formatElapsed, msToTime, applyTimeToMs } from "$lib/utils";
  import { commands, events, type Interval } from "$lib/bindings";
  import TimeInput from "$lib/components/time-input.svelte";

  type Props = {
    fromMs: number;
    toMs: number;
    label?: string;
  };

  let { fromMs, toMs, label = "Intervals" }: Props = $props();

  let intervals = $state<Interval[]>([]);
  let initialized = $state(false);
  let error = $state<string | null>(null);

  // Per-row editable copies, keyed by interval id
  let editValues = $state(new Map<number, { startTime: Time; endTime: Time | undefined }>());

  function initEditValues(ivs: Interval[]) {
    const m = new Map<number, { startTime: Time; endTime: Time | undefined }>();
    for (const iv of ivs) {
      m.set(iv.id, {
        startTime: msToTime(iv.start_ms),
        endTime: iv.end_ms !== null ? msToTime(iv.end_ms) : undefined,
      });
    }
    editValues = m;
  }

  async function fetchIntervals() {
    error = null;
    const r = await commands.getIntervals(fromMs, toMs);
    if (r.status === "ok") {
      intervals = r.data;
      initEditValues(r.data);
    } else {
      error = r.error;
    }
    initialized = true;
  }

  $effect(() => {
    fromMs; toMs;
    fetchIntervals();
  });

  $effect(() => {
    const unsub = events.intervalChanged.listen(() => fetchIntervals());
    return () => { unsub.then((fn) => fn()); };
  });

  async function deleteInterval(id: number) {
    const r = await commands.deleteInterval(id);
    if (r.status === "error") error = r.error;
  }

  async function saveInterval(id: number) {
    const interval = intervals.find((iv) => iv.id === id);
    const ev = editValues.get(id);
    if (!interval || !ev) return;

    const origStart = msToTime(interval.start_ms);
    const startChanged = ev.startTime.hour !== origStart.hour || ev.startTime.minute !== origStart.minute;
    const newStartMs = startChanged
      ? applyTimeToMs(interval.start_ms, ev.startTime)
      : interval.start_ms;

    const origEnd = interval.end_ms !== null ? msToTime(interval.end_ms) : undefined;
    const endChanged = origEnd !== undefined && ev.endTime !== undefined &&
      (ev.endTime.hour !== origEnd.hour || ev.endTime.minute !== origEnd.minute);
    const newEndMs = endChanged
      ? applyTimeToMs(interval.end_ms!, ev.endTime!)
      : interval.end_ms;

    if (newEndMs !== null && newStartMs >= newEndMs) return;
    if (newStartMs === interval.start_ms && newEndMs === interval.end_ms) return;

    const r = await commands.updateInterval(id, newStartMs, newEndMs);
    if (r.status === "error") error = r.error;
    // intervalChanged event fired by backend triggers fetchIntervals automatically
  }

  let nowTime = $derived.by(() => {
    const d = new Date(now());
    return new Time(d.getHours(), d.getMinutes());
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
    {#if initialized && !error}
      <span class="text-xs text-muted-foreground">
        {intervals.length}
        {intervals.length === 1 ? "interval" : "intervals"}
      </span>
    {/if}
  </div>

  {#if !initialized}
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
        {@const ev = editValues.get(interval.id)}
        {@const durationMs = isRunning
          ? Math.max(0, Math.min(now(), toMs) - Math.max(interval.start_ms, fromMs))
          : interval.end_ms! - interval.start_ms}

        <div
          class="flex items-center gap-2 px-3 py-1.5 text-sm bg-card {isRunning
            ? 'bg-green-500/5'
            : ''}"
          onfocusout={(e) => {
            if (!e.currentTarget.contains(e.relatedTarget as Node)) saveInterval(interval.id);
          }}
        >
          <TimeInput
            value={ev?.startTime}
            variant="ghost"
            showPeriod={true}
            maxValue={ev?.endTime ?? nowTime}
            onchange={(t) => {
              if (t && ev) editValues.set(interval.id, { ...ev, startTime: t });
            }}
          />

          <span class="text-muted-foreground select-none shrink-0">→</span>

          {#if isRunning}
            <span class="font-mono tabular-nums w-20 shrink-0 text-green-500">
              <span
                class="inline-block w-1.5 h-1.5 rounded-full bg-green-500 mr-1 mb-0.5 animate-pulse"
              ></span>live
            </span>
          {:else}
            <TimeInput
              value={ev?.endTime}
              variant="ghost"
              showPeriod={true}
              minValue={ev?.startTime}
              maxValue={nowTime}
              onchange={(t) => {
                if (t && ev) editValues.set(interval.id, { ...ev, endTime: t });
              }}
            />
          {/if}

          <span class="font-mono tabular-nums text-muted-foreground ml-auto shrink-0">
            {formatElapsed(durationMs)}
          </span>

          <button
            type="button"
            class="shrink-0 p-0.5 rounded text-muted-foreground/40 hover:text-destructive transition-colors"
            onclick={() => deleteInterval(interval.id)}
          >
            <Trash2 size={13} />
          </button>
        </div>
      {/each}
    </div>

    <div class="flex items-baseline justify-between mt-2 px-1">
      <span class="text-xs text-muted-foreground">Total</span>
      <span class="text-sm font-mono tabular-nums">{formatElapsed(totalMs)}</span>
    </div>
  {/if}
</div>
