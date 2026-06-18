<script lang="ts">
  import { scaleLinear } from 'd3-scale';
  import { max } from 'd3-array';

  type BarDay = { date: Date; value: number };
  type Props = { data: BarDay[] };
  let { data }: Props = $props();

  let yScale = $derived(
    scaleLinear()
      .domain([0, max(data, d => d.value) ?? 1])
      .range([0, 85])
  );
</script>

<div class="grid gap-2 min-h-20" style="grid-template-columns: repeat({data.length}, 1fr)">
  {#each data as day}
    {@const barHeight = yScale(day.value)}
    <svg viewBox="0 0 10 100" class="w-full h-full" preserveAspectRatio="none">
      <rect
        x="1"
        y={100 - barHeight}
        width="8"
        height={barHeight}
        class="fill-green-500/70 hover:fill-green-500 transition-colors"
      />
    </svg>
  {/each}
</div>
