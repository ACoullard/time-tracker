<script lang="ts">
  type Props = { dates: Date[] };
  let { dates }: Props = $props();

  function fmtDate(d: Date) {
    return d.toLocaleDateString('en-GB', { day: 'numeric', month: 'short' });
  }

  function fmtDay(d: Date) {
    return d.toLocaleDateString('en-GB', { weekday: 'short' });
  }
</script>

<div class="grid gap-2" style="grid-template-columns: repeat({dates.length}, 1fr)">
  {#each dates as date}
    <div class="flex flex-col items-center text-sm text-muted-foreground">
      <span>{fmtDate(date)}</span>
      <span>{fmtDay(date)}</span>
    </div>
  {/each}
</div>
