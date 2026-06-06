<script lang="ts">
  import { TimeField } from "bits-ui";
  import type { Time } from "@internationalized/date";

  let {
    value = $bindable(undefined),
    onchange,
    label,
    showPeriod = true,
    disabled = false,
    variant = "default",
  }: {
    value?: Time | undefined;
    onchange?: (v: Time | undefined) => void;
    label?: string;
    showPeriod?: boolean;
    disabled?: boolean;
    variant?: "default" | "ghost";
  } = $props();
</script>

<TimeField.Root
  bind:value
  hourCycle={showPeriod ? 12 : 24}
  onValueChange={onchange}
  {disabled}
>
  {#if label}
    <TimeField.Label class="block text-sm text-muted-foreground mb-1">
      {label}
    </TimeField.Label>
  {/if}

  <TimeField.Input
    onkeydown={(e) => { if (e.key === "Enter") (e.target as HTMLElement).blur(); }}
    class="inline-flex items-center rounded-md px-2 py-1 text-sm font-mono
           focus-within:ring-3 focus-within:ring-ring/30 focus-within:border-ring
           transition-all disabled:opacity-50 disabled:pointer-events-none
           {variant === 'ghost'
             ? 'border border-transparent bg-transparent hover:border-input/50 cursor-pointer'
             : 'border border-input bg-background'}"
  >
    {#snippet children({ segments })}
      {#each segments as { part, value: segVal }}
        {#if part === "literal"}
          <span class="text-muted-foreground select-none">{segVal}</span>
        {:else}
          <TimeField.Segment
            {part}
            class="rounded px-0.5 tabular-nums outline-none text-center select-none
                   text-foreground
                   data-[type=hour]:w-7
                   data-[type=minute]:w-7
                   data-[type=dayPeriod]:w-8
                   data-[placeholder]:text-muted-foreground
                   focus:bg-primary focus:text-primary-foreground"
          >{segVal}</TimeField.Segment>
        {/if}
      {/each}
    {/snippet}
  </TimeField.Input>
</TimeField.Root>
