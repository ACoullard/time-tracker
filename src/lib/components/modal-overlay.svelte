<script lang="ts">
	import * as Dialog from '$lib/components/ui/dialog';
	import { modal } from '$lib/modal.svelte';
	import ModalCard from './modal-card.svelte';
</script>

{#if modal() !== null}
	{@const m = modal()!}
	<Dialog.Root
		open={true}
		onOpenChange={(isOpen) => {
			if (!isOpen) m.onCancel();
		}}
	>
		<Dialog.Content showCloseButton={false}>
			<ModalCard title={m.title} message={m.message} onConfirm={m.onConfirm} onCancel={m.onCancel} />
		</Dialog.Content>
	</Dialog.Root>
{/if}
