<script lang="ts">
  import DayAxis from '$lib/components/charts/day-axis.svelte';
  import BarChart from '$lib/components/charts/bar-chart.svelte';
  import RingChart from '$lib/components/charts/ring-chart.svelte';
  import { commands, events } from '$lib/bindings';
  import { formatIsoYMD } from '$lib/utils';

  const days = Array.from({ length: 7 }, (_, i) => {
    const d = new Date();
    d.setDate(d.getDate() - 6 + i);
    d.setHours(0, 0, 0, 0);
    return d;
  });


  let barData = $state(days.map(d => ({ date: d, value: 0 })));
  let ringData = $state(days.map(d => ({ date: d, current: 0, max: 0 })));

  $effect(() => {
    async function load() {
      const from = formatIsoYMD(days[0]);
      const to = formatIsoYMD(days[days.length - 1]);
      const [totalsRes, goalsRes] = await Promise.all([
        commands.getDailyTotals(from, to),
        commands.getDailyGoalsForRange(from, to),
      ]);
      if (totalsRes.status === 'ok') {
        const totals = new Map(totalsRes.data.map(t => [t.day, t.total_ms]));
        barData = days.map(d => ({ date: d, value: totals.get(formatIsoYMD(d)) ?? 0 }));

        if (goalsRes.status === 'ok') {
          const goals = new Map(goalsRes.data.map(g => [g.day, g.goal_ms]));
          ringData = days.map(d => ({
            date: d,
            current: totals.get(formatIsoYMD(d)) ?? 0,
            max: goals.get(formatIsoYMD(d)) ?? 0,
          }));
        }
      }
    }

    load();
    return events.intervalChanged.listen(load);
  });
</script>

<main class="p-8 flex flex-col gap-6 max-w-3xl mx-auto max-h-full">
  <div class="flex flex-col gap-2 flex-1 min-h-0">
    <DayAxis dates={days} />
    <div class="flex flex-col gap-10 flex-1 min-h-0">
      <BarChart data={barData} />
      <RingChart data={ringData} />
    </div>
  </div>
</main>
