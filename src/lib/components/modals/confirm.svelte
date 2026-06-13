<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import XIcon from '@lucide/svelte/icons/x';

	let { title, message, onResolve }: {
		title: string;
		message: string;
		onResolve: (result: boolean) => void;
	} = $props();
</script>

<div class="flex items-start gap-2">
	<Button variant="ghost" size="icon-sm" class="-ml-3 -mt-3 shrink-0" onclick={() => onResolve(false)}>
		<XIcon size={16} />
	</Button>
	<div class="flex flex-col gap-1.5">
		<h2 class="text-base leading-none font-medium">{title}</h2>
		<p class="text-muted-foreground text-sm">{message}</p>
	</div>
</div>
<div class="flex flex-col-reverse gap-2 sm:flex-row sm:justify-end">
	<Button variant="outline" onclick={() => onResolve(false)}>Cancel</Button>
	<Button onclick={() => onResolve(true)}>Confirm</Button>
</div>
