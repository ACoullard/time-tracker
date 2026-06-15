<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { formatTime } from '$lib/utils';
	import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
	import { commands } from '$lib/bindings';

	let { idleSinceMs, onClose }: {
		idleSinceMs: number;
		onClose: () => void;
	} = $props();

	async function yes() {
		await commands.endIntervalAt(idleSinceMs);
		onClose();
		await getCurrentWebviewWindow().hide();
	}

	async function no() {
		onClose();
		await getCurrentWebviewWindow().hide();
	}
</script>

<div class="flex flex-col gap-4 w-full px-2">
	<h1 class="text-lg leading-none font-medium">Still There?</h1>
	<div class="flex flex-col gap-3 px-10 py-4">
		<Button onclick={yes}>Stop timer at {formatTime(idleSinceMs)}</Button>
		<Button variant="outline" onclick={no}>Continue</Button>
	</div>
</div>
