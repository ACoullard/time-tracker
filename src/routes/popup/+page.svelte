<script lang="ts">
	import { onMount } from 'svelte';
	import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
	import { events } from '$lib/bindings';
	import ModalCard from '$lib/components/modal-card.svelte';

	let title = $state('');
	let message = $state('');

	async function dismiss() {
		await getCurrentWebviewWindow().hide();
	}

	onMount(() => {
		const unlisten = events.popupShow.listen((e) => {
			title = e.payload.title;
			message = e.payload.message;
		});
		return () => {
			unlisten.then((fn) => fn());
		};
	});
</script>

<div class="flex h-screen items-center justify-center bg-transparent p-4">
	{#if title}
		<ModalCard {title} {message} onConfirm={dismiss} onCancel={dismiss} />
	{/if}
</div>
