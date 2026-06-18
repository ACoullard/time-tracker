<script lang="ts">
  type Props = { dates: Date[] };
  let { dates }: Props = $props();

  function fmt(d: Date) {
    return d.toLocaleDateString('en-GB', { day: 'numeric', month: 'short' });
  }
</script>

<div class="grid gap-2" style="grid-template-columns: repeat({dates.length}, 1fr)">
  {#each dates as date}
    <div class="text-center text-xs text-muted-foreground">{fmt(date)}</div>
  {/each}
</div>
