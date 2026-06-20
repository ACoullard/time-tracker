<!-- @component
  Per-day bar chart. `data: null` renders a skeleton (column backgrounds only).
  Y-scale is computed from the max value in the current dataset, with 15% top padding.
-->
<script lang="ts">
  import { scaleLinear } from 'd3-scale';
  import { max } from 'd3-array';

  type BarDay = { date: Date; value: number };
  type Props = { data: BarDay[] | null; count: number };
  let { data, count }: Props = $props();

  import { formatElapsed, formatElapsedSeconds } from '$lib/utils';

  let yScale = $derived(
    scaleLinear()
      .domain([0, max(data ?? [], d => d.value) || 1])
      .range([0, 85])
  );
</script>

<div class="grid gap-2 min-h-20 min-w-xl" style="grid-template-columns: repeat({count}, 1fr)">
  {#each (data ?? Array.from({ length: count }, () => null)) as day}
    {@const barHeight = day ? yScale(day.value) : 0}
    <div class="relative min-h-0">
      <svg viewBox="0 0 10 100" class="w-full h-full" preserveAspectRatio="none">
        <rect x="0" y="0" width="10" height="100" class="fill-muted-foreground/5" />
        {#if day}
          <rect
            x="1"
            y={100 - barHeight}
            width="8"
            height={barHeight}
            class="fill-green-500/70 hover:fill-green-500 transition-colors"
          />
        {/if}
      </svg>
      {#if day && day.value > 0}
        <div
          class="absolute left-0 right-0 flex justify-center pointer-events-none"
          style="bottom: {barHeight + 2}%"
        >
          <span class=" leading-none">{day.value > 60 * 1000 ?formatElapsed(day.value, true) : formatElapsedSeconds(day.value)}</span>
        </div>
      {/if}
    </div>
  {/each}
</div>
