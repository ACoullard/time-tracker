<script lang="ts">
  import { Check } from '@lucide/svelte';

  type RingDay = { date: Date; current: number; max: number };
  type Props = { data: RingDay[] };
  let { data }: Props = $props();

  const r = 40;
  const circumference = 2 * Math.PI * r;
</script>

<div class="grid gap-2 h-24" style="grid-template-columns: repeat({data.length}, 1fr)">
  {#each data as day}
    {@const progress = day.max > 0 ? Math.min(1, day.current / day.max) : 0}
    {@const offset = circumference * (1 - progress)}
    <div class="relative">
      <svg viewBox="0 0 100 100" class="-rotate-90 w-full h-full">
        <circle
          cx="50"
          cy="50"
          r={r}
          stroke-width="10"
          class="fill-none stroke-muted-foreground/15"
        />
        <circle
          cx="50"
          cy="50"
          r={r}
          stroke-width="10"
          class="fill-none stroke-green-500"
          stroke-linecap="round"
          stroke-dasharray={circumference}
          stroke-dashoffset={offset}
          style="transition: stroke-dashoffset 0.4s ease"
        />
      </svg>
      {#if day.current >= day.max}
        <div class="absolute inset-0 flex items-center justify-center pointer-events-none">
          <Check size={36} strokeWidth={5.5} stroke-linecap="square" class="text-green-500" />
        </div>
      {/if}
    </div>
  {/each}
</div>
