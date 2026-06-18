<script lang="ts">
  import DayAxis from '$lib/components/charts/day-axis.svelte';
  import BarChart from '$lib/components/charts/bar-chart.svelte';
  import RingChart from '$lib/components/charts/ring-chart.svelte';
  import { commands, events } from '$lib/bindings';
  import { formatIsoYMD } from '$lib/utils';
  import { now } from '$lib/now.svelte';
  import { ChevronLeft, ChevronRight } from '@lucide/svelte';

  const todayKey = formatIsoYMD(new Date());

  let weekOffset = $state(0);

  let days = $derived.by(() => {
    const today = new Date();
    today.setHours(0, 0, 0, 0);
    const sunday = new Date(today);
    sunday.setDate(today.getDate() - today.getDay() + weekOffset * 7);
    return Array.from({ length: 7 }, (_, i) => {
      const d = new Date(sunday);
      d.setDate(sunday.getDate() + i);
      d.setHours(0, 0, 0, 0);
      return d;
    });
  });

  let weekLabel = $derived(
    `${days[0].toLocaleDateString('en-US', { month: 'numeric', day: 'numeric' })} - ` +
    `${days[6].toLocaleDateString('en-US', { month: 'numeric', day: 'numeric' })}`
  );

  let fetchedTotals = $state<Map<string, number> | null>(null);
  let fetchedGoals = $state<Map<string, number> | null>(null);
  let startMs = $state<number | null>(null);

  $effect(() => {
    fetchedTotals = null;
    fetchedGoals = null;

    const from = formatIsoYMD(days[0]);
    const to = formatIsoYMD(days[days.length - 1]);

    async function load() {
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
    <div class="flex items-center justify-between mb-1">
      <button
        onclick={() => weekOffset--}
        class="p-1 rounded text-muted-foreground hover:text-foreground transition-colors"
      >
        <ChevronLeft size={16} />
      </button>
      <span class="text-sm text-muted-foreground">{weekLabel}</span>
      <button
        onclick={() => weekOffset++}
        disabled={weekOffset >= 0}
        class="p-1 rounded text-muted-foreground hover:text-foreground transition-colors disabled:opacity-30 disabled:pointer-events-none"
      >
        <ChevronRight size={16} />
      </button>
    </div>
    <DayAxis dates={days} />
    <div class="flex flex-col gap-10 flex-1 min-h-0">
      <BarChart data={barData} count={days.length} />
      <RingChart data={ringData} count={days.length} />
    </div>
  </div>
</main>
