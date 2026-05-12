<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { invoke } from "@tauri-apps/api/core";

  let startMs = $state<number | null>(null);
  let lastDurationMs = $state<number | null>(null);
  let now = $state(Date.now());
  let error = $state<string | null>(null);

  let running = $derived(startMs !== null);
  let displayMs = $derived(
    startMs !== null ? now - startMs : (lastDurationMs ?? 0),
  );

  $effect(() => {
    (async () => {
      try {
        startMs = await invoke<number | null>("get_current_interval");
      } catch (e) {
        error = String(e);
      }
    })();
  });

  $effect(() => {
    if (!running) return;
    const id = setInterval(() => {
      now = Date.now();
    }, 1000);
    return () => clearInterval(id);
  });

  async function toggle() {
    error = null;
    try {
      if (running) {
        const duration = now - startMs!;
        await invoke("end_interval");
        startMs = null;
        lastDurationMs = duration;
      } else {
        startMs = await invoke<number>("begin_interval");
        now = Date.now();
      }
    } catch (e) {
      error = String(e);
    }
  }

  function formatElapsed(ms: number): string {
    const total = Math.max(0, Math.floor(ms / 1000));
    const h = Math.floor(total / 3600);
    const m = Math.floor((total % 3600) / 60);
    const s = total % 60;
    return `${h}:${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
  }
</script>

<main class="p-8 max-w-md mx-auto">
  <h1 class="text-2xl font-semibold mb-6">Time Tracker</h1>

  <p class="text-5xl font-mono tabular-nums mb-6">{formatElapsed(displayMs)}</p>

  <Button onclick={toggle} class="w-full">
    {running ? "Stop" : "Start"}
  </Button>

  {#if error}
    <p class="text-sm text-destructive mt-4">{error}</p>
  {/if}
</main>
