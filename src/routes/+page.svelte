<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import GoalRing from "$lib/components/goal-ring.svelte";
  import IntervalsPanel from "$lib/components/intervals-panel.svelte";
  import TimeInput from "$lib/components/time-input.svelte";
  import { Time } from "@internationalized/date";
  import { now, bumpNow } from "$lib/now.svelte";
  import { formatElapsed } from "$lib/utils";
  import { commands, events } from "$lib/bindings";

  let startMs = $state<number | null>(null);
  let lastDurationMs = $state<number | null>(null);
  let error = $state<string | null>(null);

  let goalMs = $state<number | null>(null);
  let goalTime = $state<Time | undefined>(undefined);

  let running = $derived(startMs !== null);
  let displayMs = $derived(
    startMs !== null ? now() - startMs : (lastDurationMs ?? 0),
  );

  const todayStart = new Date();
  todayStart.setHours(0, 0, 0, 0);
  const todayEnd = new Date(todayStart);
  todayEnd.setDate(todayEnd.getDate() + 1);
  const todayFromMs = todayStart.getTime();
  const todayToMs = todayEnd.getTime();

  async function syncTimerState() {
    const result = await commands.getTimerState();
    if (result.status === "ok") {
      const s = result.data;
      startMs = s.state === "Running" ? s.start_ms : null;
      lastDurationMs = s.state === "Paused" ? s.last_duration_ms : null;
    } else {
      error = result.error;
    }
  }

  $effect(() => {
    (async () => {
      await syncTimerState();

      const gr = await commands.getCurrentGoal();
      if (gr.status === "ok") {
        goalMs = gr.data?.goal_ms ?? null;
        if (goalMs !== null) {
          const totalMins = Math.round(goalMs / 60_000);
          goalTime = new Time(Math.floor(totalMins / 60), totalMins % 60);
        }
      }
    })();
  });

  $effect(() => {
    const unsub = events.intervalChanged.listen(async () => {
      await syncTimerState();
      bumpNow();
    });
    return () => {
      unsub.then((fn) => fn());
    };
  });

  async function toggle() {
    error = null;
    if (running) {
      const duration = now() - startMs!;
      const result = await commands.endInterval();
      if (result.status === "ok") {
        startMs = null;
        lastDurationMs = duration;
      } else {
        error = result.error;
      }
    } else {
      const result = await commands.beginInterval();
      if (result.status === "ok") {
        startMs = result.data;
      } else {
        error = result.error;
      }
    }
  }

  async function saveGoal() {
    if (!goalTime) return;
    const ms = (goalTime.hour * 60 + goalTime.minute) * 60_000;
    if (ms <= 0) return;
    const result = await commands.setDailyGoal(ms);
    if (result.status === "ok") {
      goalMs = ms;
    } else {
      error = result.error;
    }
  }
</script>

<main class="p-8 flex gap-20 max-w-3xl mx-auto">
  <div class="flex-none w-64">
    <h1 class="text-2xl font-semibold mb-6">Time Tracker</h1>

    <div class="flex justify-center mb-6">
      <GoalRing
        fromMs={todayFromMs}
        toMs={todayToMs}
        isRunning={running}
        goalMs={goalMs}
      />
    </div>

    <p class="text-5xl font-mono tabular-nums mb-6">{formatElapsed(displayMs)}</p>

    <Button onclick={toggle} class="w-full">
      {running ? "Stop" : "Start"}
    </Button>

    <div class="mt-4 flex items-baseline justify-between">
      <span class="text-sm text-muted-foreground">Daily goal</span>
      <div onfocusout={(e) => {
        if (!e.currentTarget.contains(e.relatedTarget as Node)) saveGoal();
      }}>
        <TimeInput bind:value={goalTime} showPeriod={false} />
      </div>
    </div>

    {#if error}
      <p class="text-sm text-destructive mt-4">{error}</p>
    {/if}
  </div>

  <div class="flex-1 min-w-0">
    <IntervalsPanel fromMs={todayFromMs} toMs={todayToMs} label="Today's intervals" />
  </div>
</main>
