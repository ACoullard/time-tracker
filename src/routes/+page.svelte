<script lang="ts">
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
  <h1>Welcome to time tracker</h1>
  <p>Current time: {curTime.toLocaleTimeString()}</p>
</main>

