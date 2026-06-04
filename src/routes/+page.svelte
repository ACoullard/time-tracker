<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import RangeTotal from "$lib/components/range-total.svelte";
  import { now, bumpNow } from "$lib/now.svelte";
  import { formatElapsed } from "$lib/utils";
  import { commands, events } from "$lib/bindings";

  let startMs = $state<number | null>(null);
  let lastDurationMs = $state<number | null>(null);
  let error = $state<string | null>(null);

  let goalMs = $state<number | null>(null);
  let editingGoal = $state(false);
  let goalInput = $state("");

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

  $effect(() => {
    (async () => {
      const result = await commands.getTimerState();
      if (result.status === "ok") {
        const s = result.data;
        if (s.state === "Running") {
          startMs = s.start_ms;
        } else if (s.state === "Paused") {
          lastDurationMs = s.last_duration_ms;
        }
      } else {
        error = result.error;
      }

      const gr = await commands.getCurrentGoal();
      if (gr.status === "ok") {
        goalMs = gr.data?.goal_ms ?? null;
      }
    })();
  });

  $effect(() => {
    const unsub = events.intervalChanged.listen((evt) => {
      const newStart = evt.payload.running_start_ms;
      if (newStart === null && startMs !== null) {
        lastDurationMs = Date.now() - startMs;
      }
      startMs = newStart;
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
    const mins = parseFloat(goalInput);
    if (isNaN(mins) || mins <= 0) return;
    const ms = Math.round(mins * 60_000);
    const result = await commands.setDailyGoal(ms);
    if (result.status === "ok") {
      goalMs = ms;
      editingGoal = false;
    } else {
      error = result.error;
    }
  }

  function startEditingGoal() {
    goalInput = goalMs !== null ? String(Math.round(goalMs / 60_000)) : "";
    editingGoal = true;
  }
</script>

<main class="p-8 max-w-md mx-auto">
  <h1 class="text-2xl font-semibold mb-6">Time Tracker</h1>

  <p class="text-5xl font-mono tabular-nums mb-6">{formatElapsed(displayMs)}</p>

  <Button onclick={toggle} class="w-full">
    {running ? "Stop" : "Start"}
  </Button>

  <div class="mt-6">
    <RangeTotal
      fromMs={todayFromMs}
      toMs={todayToMs}
      isRunning={running}
      label="Today"
    />
  </div>

  <div class="mt-4 flex items-baseline justify-between">
    <span class="text-sm text-muted-foreground">Daily goal</span>
    {#if editingGoal}
      <form
        onsubmit={(e) => { e.preventDefault(); saveGoal(); }}
        class="flex gap-2 items-center"
      >
        <input
          type="number"
          min="1"
          placeholder="minutes"
          bind:value={goalInput}
          class="w-24 text-right font-mono border rounded px-2 py-0.5 text-sm"
        />
        <Button type="submit" size="xs">Save</Button>
        <Button size="xs" variant="ghost" onclick={() => (editingGoal = false)}>Cancel</Button>
      </form>
    {:else}
      <button
        onclick={startEditingGoal}
        class="text-sm font-mono tabular-nums text-right hover:underline"
      >
        {goalMs !== null ? formatElapsed(goalMs) : "— Set goal"}
      </button>
    {/if}
  </div>

  {#if error}
    <p class="text-sm text-destructive mt-4">{error}</p>
  {/if}
</main>
