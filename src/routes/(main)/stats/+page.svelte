<script lang="ts">
  import DayAxis from '$lib/components/charts/day-axis.svelte';
  import BarChart from '$lib/components/charts/bar-chart.svelte';
  import RingChart from '$lib/components/charts/ring-chart.svelte';
  import { commands, events } from '$lib/bindings';
  import { formatIsoYMD } from '$lib/utils';
  import { now } from '$lib/now.svelte';

  const days = Array.from({ length: 7 }, (_, i) => {
    const d = new Date();
    d.setDate(d.getDate() - 6 + i);
    d.setHours(0, 0, 0, 0);
    return d;
  });

  const todayKey = formatIsoYMD(new Date());

  let fetchedTotals = $state<Map<string, number> | null>(null);
  let fetchedGoals = $state<Map<string, number> | null>(null);
  let startMs = $state<number | null>(null);

  $effect(() => {
    async function load() {
      const from = formatIsoYMD(days[0]);
      const to = formatIsoYMD(days[days.length - 1]);
      const [totalsRes, goalsRes, timerRes] = await Promise.all([
        commands.getDailyTotals(from, to),
        commands.getDailyGoalsForRange(from, to),
        commands.getTimerState(),
      ]);
      if (totalsRes.status === 'ok') {
        fetchedTotals = new Map(totalsRes.data.map(t => [t.day, t.total_ms]));
      }
      if (goalsRes.status === 'ok') {
        fetchedGoals = new Map(goalsRes.data.map(g => [g.day, g.goal_ms]));
      }
      if (timerRes.status === 'ok') {
        startMs = timerRes.data.state === 'Running' ? timerRes.data.start_ms : null;
      }
    }

    load();
    return events.intervalChanged.listen(load);
  });

  let barData = $derived(
    fetchedTotals === null ? null :
    days.map(d => {
      const key = formatIsoYMD(d);
      let value = fetchedTotals!.get(key) ?? 0;
      if (startMs !== null && key === todayKey) value += now() - startMs;
      return { date: d, value };
    })
  );

  let ringData = $derived(
    fetchedTotals === null || fetchedGoals === null ? null :
    days.map(d => {
      const key = formatIsoYMD(d);
      let current = fetchedTotals!.get(key) ?? 0;
      if (startMs !== null && key === todayKey) current += now() - startMs;
      return { date: d, current, max: fetchedGoals!.get(key) ?? 0 };
    })
  );
</script>

<main class="p-8 flex flex-col gap-6 max-w-3xl mx-auto max-h-full">
  <div class="flex flex-col gap-2 flex-1 min-h-0">
    <DayAxis dates={days} />
    <div class="flex flex-col gap-10 flex-1 min-h-0">
      <BarChart data={barData} count={days.length} />
      <RingChart data={ringData} count={days.length} />
    </div>
  </div>
</main>
