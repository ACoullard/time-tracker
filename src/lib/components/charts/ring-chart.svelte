<!-- @component
  Per-day progress rings. `data: null` renders a skeleton (empty cells, no rings).
  Days with `max === 0` also render nothing. Shows a checkmark when `current >= max`.
-->
<script lang="ts">
  import { Check } from '@lucide/svelte';

  type RingDay = { date: Date; current: number; max: number };
  type Props = { data: RingDay[] | null; count: number };
  let { data, count }: Props = $props();

  const r = 40;
  const circumference = 2 * Math.PI * r;
</script>

<div class="grid gap-2 h-24 min-h-24" style="grid-template-columns: repeat({count}, 1fr)">
  {#each data ?? [] as day}
    <div class="relative">
      {#if day.max > 0}
        {@const progress = Math.min(1, day.current / day.max)}
        {@const offset = circumference * (1 - progress)}
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
      {/if}
    </div>
  {/each}
</div>
