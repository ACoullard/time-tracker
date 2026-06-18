<script lang="ts">
  import DayAxis from '$lib/components/charts/day-axis.svelte';
  import BarChart from '$lib/components/charts/bar-chart.svelte';
  import RingChart from '$lib/components/charts/ring-chart.svelte';

  const days = Array.from({ length: 7 }, (_, i) => {
    const d = new Date();
    d.setDate(d.getDate() - 6 + i);
    d.setHours(0, 0, 0, 0);
    return d;
  });

  // DUMMY DATA — replace this block in the real-data PR.
  // rawValues mirrors the shape the backend will return: Map<"YYYY-MM-DD", value>
  const dayKey = (d: Date) => d.toISOString().slice(0, 10);
  const HOUR = 3_600_000;
  const GOAL = 8 * HOUR;
  const rawValues = new Map(days.map(d => [dayKey(d), Math.random() * 10 * HOUR]));

  // Translation layer — stays as-is when real data arrives; only rawValues changes
  const barData = days.map(d => ({ date: d, value: rawValues.get(dayKey(d)) ?? 0 }));
  const ringData = days.map(d => ({ date: d, current: rawValues.get(dayKey(d)) ?? 0, max: GOAL }));
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
