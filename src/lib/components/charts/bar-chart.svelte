<script lang="ts">
  type BarDay = { date: Date; value: number };
  type Props = { data: BarDay[] };
  let { data }: Props = $props();

  let maxVal = $derived(Math.max(...data.map(d => d.value), 1));
</script>

<div class="grid gap-2 h-24" style="grid-template-columns: repeat({data.length}, 1fr)">
  {#each data as day}
    <div class="flex flex-col justify-end h-full">
      <div
        class="w-full rounded-t-sm bg-green-500/70 hover:bg-green-500 transition-colors"
        style="height: {(day.value / maxVal) * 100}%"
      ></div>
    </div>
  {/each}
</div>
