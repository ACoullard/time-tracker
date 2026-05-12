<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { invoke } from "@tauri-apps/api/core";
  import { SvelteDate } from "svelte/reactivity";

  let curTime = new SvelteDate();
  let stopWatchRunning = $state(false);

  $effect(() => {
    const interval = setInterval(() => {
      curTime.setTime(Date.now())
    }, 100);

    return () => {
			clearInterval(interval);
		};
  })

</script>

<main>
  <div class="p-4">
    <h1>Welcome to time tracker</h1>
    <p>Current time: {curTime.toLocaleTimeString()}</p>
    <div class="flex gap-4 p-4">
      <Button class="p-4">Start</Button>
      <Button class="p-4">Stop</Button>
    </div>
  </div>
</main>

