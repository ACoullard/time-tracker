<script lang="ts">
  import DayAxis from '$lib/components/charts/day-axis.svelte';
  import BarChart from '$lib/components/charts/bar-chart.svelte';
  import RingChart from '$lib/components/charts/ring-chart.svelte';
  import { commands, events } from '$lib/bindings';
  import { formatIsoYMD, formatElapsed } from '$lib/utils';
  import { now } from '$lib/now.svelte';
  import { ChevronLeft, ChevronRight, Flame } from '@lucide/svelte';
  import { pageHeader } from '$lib/page-header.svelte';

  const todayKey = formatIsoYMD(new Date());
  const yesterdayKey = (() => {
    const d = new Date();
    d.setDate(d.getDate() - 1);
    d.setHours(0, 0, 0, 0);
    return formatIsoYMD(d);
  })();

  // --- Week navigation ---
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

  // --- Week chart data ---
  let fetchedTotals = $state<Map<string, number> | null>(null);
  let fetchedGoals = $state<Map<string, number> | null>(null);

  $effect(() => {
    fetchedTotals = null;
    fetchedGoals = null;
    const from = formatIsoYMD(days[0]);
    const to = formatIsoYMD(days[days.length - 1]);
    async function load() {
      const [totalsRes, goalsRes] = await Promise.all([
        commands.getDailyTotals(from, to),
        commands.getDailyGoalsForRange(from, to),
      ]);
      if (totalsRes.status === 'ok') fetchedTotals = new Map(totalsRes.data.map(t => [t.day, t.total_ms]));
      if (goalsRes.status === 'ok') fetchedGoals = new Map(goalsRes.data.map(g => [g.day, g.goal_ms]));
    }
    load();
    return events.intervalChanged.listen(load);
  });

  // --- Sidebar: streak + today (independent of week navigation) ---
  let baseStreak = $state(0);
  let fetchedTodayClosedTotal = $state(0);
  let fetchedTodayGoal = $state(0);
  let startMs = $state<number | null>(null);
  let sidebarLoaded = $state(false);

  $effect(() => {
    async function loadSidebar() {
      const [streakRes, todayTotalsRes, todayGoalsRes, timerRes] = await Promise.all([
        commands.getStreak(yesterdayKey),
        commands.getDailyTotals(todayKey, todayKey),
        commands.getDailyGoalsForRange(todayKey, todayKey),
        commands.getTimerState(),
      ]);
      if (streakRes.status === 'ok') baseStreak = streakRes.data;
      if (todayTotalsRes.status === 'ok') fetchedTodayClosedTotal = todayTotalsRes.data[0]?.total_ms ?? 0;
      if (todayGoalsRes.status === 'ok') fetchedTodayGoal = todayGoalsRes.data[0]?.goal_ms ?? 0;
      if (timerRes.status === 'ok') startMs = timerRes.data.state === 'Running' ? timerRes.data.start_ms : null;
      sidebarLoaded = true;
    }
    loadSidebar();
    return events.intervalChanged.listen(loadSidebar);
  });

  let todayTotal = $derived(fetchedTodayClosedTotal + (startMs !== null ? now() - startMs : 0));
  let todayMet = $derived(fetchedTodayGoal > 0 && todayTotal >= fetchedTodayGoal);
  let streak = $derived(todayMet ? baseStreak + 1 : baseStreak);

  let weekTotal = $derived(
    fetchedTotals === null ? 0 :
    days.reduce((sum, d) => {
      const key = formatIsoYMD(d);
      let value = fetchedTotals!.get(key) ?? 0;
      if (startMs !== null && key === todayKey) value += now() - startMs;
      return sum + value;
    }, 0)
  );

  // --- Chart data ---
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

  $effect(() => {
    pageHeader.set(weekSelector);
    return () => pageHeader.set(null);
  });
</script>

{#snippet weekSelector()}
  <div class="flex items-center gap-1 border rounded-full p-1 px-2 shadow-xs">
    <button
      onclick={() => weekOffset--}
      class="p-1 px-2 rounded-md rounded-l-3xl hover:bg-muted transition-colors"
    >
      <ChevronLeft size={24} strokeWidth={2} />
    </button>
    <span class="text-lg tabular-nums text-center w-32">{weekLabel}</span>
    <button
      onclick={() => weekOffset++}
      disabled={weekOffset >= 0}
      class="p-1 px-2 rounded-md rounded-r-3xl hover:bg-muted transition-colors disabled:opacity-30 disabled:pointer-events-none"
    >
      <ChevronRight size={24} strokeWidth={2} />
    </button>
  </div>
{/snippet}

<main class="p-8 flex gap-10 px-10 mx-auto max-w-6xl max-h-full">
  <div class="flex flex-col justify-between gap-4 mt-6 max-h-82 min-w-44">
    <!-- large total -->
    <div class="flex flex-col gap-0.5">
      <span class="text-sm text-neutral-400 font-semibold pl-0.5">This week</span>
      <div class="flex flex-col items-center px-4 pt-6 pb-10 gap-1 bg-accent rounded-xl">
        {#if fetchedTotals !== null}
          <span class="text-5xl font-bold tabular-nums">{formatElapsed(weekTotal, true)}</span>
        {/if}
      </div>
    </div>
    <!-- large streak -->
    <div class="flex flex-col">
      <span class="text-sm text-neutral-400 font-semibold pl-0.5">Streak</span>
      <div class="flex flex-col px-4 pt-6 pb-12 bg-accent rounded-xl">
      {#if sidebarLoaded}
        <div class="flex items-end gap-3">
          <Flame size={54} strokeWidth={2.5} class="text-orange-500 bg-orange-200 p-2 rounded-2xl" />
          <div class="flex items-end gap-1">
            <span class="text-5xl font-bold tabular-nums leading-none">{streak}</span>
            <span class="text-xl font-semibold text-muted-foreground">{streak === 1 ? 'day' : 'days'}</span>
          </div>
        </div>
      {/if}
    </div>
    </div>
  </div>

  <div class="flex flex-col gap-2 flex-1 min-h-0">
    <DayAxis dates={days} />
    <div class="flex flex-col gap-10 flex-1 min-h-0">
      <BarChart data={barData} count={days.length} />
      <RingChart data={ringData} count={days.length} />
    </div>
  </div>
</main>
