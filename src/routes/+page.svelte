<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import RangeTotal from "$lib/components/range-total.svelte";
  import { now } from "$lib/now.svelte";
  import { formatElapsed } from "$lib/utils";
  import { invoke } from "@tauri-apps/api/core";

  let startMs = $state<number | null>(null);
  let lastDurationMs = $state<number | null>(null);
  let error = $state<string | null>(null);

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
      try {
        startMs = await invoke<number | null>("get_current_interval");
      } catch (e) {
        error = String(e);
      }
    })();
  });

  async function toggle() {
    error = null;
    try {
      if (running) {
        const duration = now() - startMs!;
        await invoke("end_interval");
        startMs = null;
        lastDurationMs = duration;
      } else {
        startMs = await invoke<number>("begin_interval");
      }
    } catch (e) {
      error = String(e);
    }
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

  {#if error}
    <p class="text-sm text-destructive mt-4">{error}</p>
  {/if}
</main>
