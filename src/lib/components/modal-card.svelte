<script lang="ts">
	import { Button } from '$lib/components/ui/button';

	type Props = {
		title: string;
		message: string;
		onConfirm?: () => void;
		onCancel?: () => void;
	};

	let { title, message, onConfirm, onCancel }: Props = $props();
</script>

<div class="flex flex-col gap-1.5">
	<h2 class="text-base leading-none font-medium">{title}</h2>
	<p class="text-muted-foreground text-sm">{message}</p>
</div>
<div class="flex flex-col-reverse gap-2 sm:flex-row sm:justify-end">
	<Button variant="outline" onclick={onCancel}>Cancel</Button>
	<Button onclick={onConfirm}>Confirm</Button>
</div>
