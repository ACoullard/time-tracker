<script lang="ts">
  import { scaleLinear } from 'd3-scale';
  import { max } from 'd3-array';

  type BarDay = { date: Date; value: number };
  type Props = { data: BarDay[] };
  let { data }: Props = $props();

  import { formatElapsed } from '$lib/utils';

  let yScale = $derived(
    scaleLinear()
      .domain([0, max(data, d => d.value) ?? 1])
      .range([0, 85])
  );
</script>

<div class="grid gap-2 min-h-20" style="grid-template-columns: repeat({data.length}, 1fr)">
  {#each data as day}
    {@const barHeight = yScale(day.value)}
    <div class="relative min-h-0">
      <svg viewBox="0 0 10 100" class="w-full h-full" preserveAspectRatio="none">
        <rect
          x="1"
          y={100 - barHeight}
          width="8"
          height={barHeight}
          class="fill-green-500/70 hover:fill-green-500 transition-colors"
        />
      </svg>
      <div
        class="absolute left-0 right-0 flex justify-center pointer-events-none"
        style="bottom: {barHeight + 2}%"
      >
        <span class="text-sm text-muted-foreground leading-none">{formatElapsed(day.value, true)}</span>
      </div>
    </div>
  {/each}
</div>
