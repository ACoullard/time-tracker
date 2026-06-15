<script lang="ts">
	import { onMount } from 'svelte';
	import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
	import { events, commands } from '$lib/bindings';
	import ModalCard from '$lib/components/modal-card.svelte';
	import IdlePrompt from '$lib/components/modals/idle-prompt.svelte';

	let title = $state('');
	let message = $state('');
	let idleSinceMs = $state<number | null>(null);

	async function dismiss() {
		await getCurrentWebviewWindow().hide();
		title = '';
	}

	onMount(() => {
		const unlistenPopup = events.popupShow.listen((e) => {
			idleSinceMs = null;
			title = e.payload.title;
			message = e.payload.message;
		});
		const unlistenIdle = events.idleDetected.listen((e) => {
			title = '';
			idleSinceMs = e.payload.idleSinceMs;
		});
		return () => {
			unlistenPopup.then((fn) => fn());
			unlistenIdle.then((fn) => fn());
		};
	});
</script>

<div class="flex h-screen items-center justify-center bg-transparent p-4">
	{#if idleSinceMs !== null}
		<IdlePrompt {idleSinceMs} onClose={() => { idleSinceMs = null; }} />
	{:else if title}
		<ModalCard {title} {message} onConfirm={dismiss} onCancel={dismiss} />
	{/if}
</div>
