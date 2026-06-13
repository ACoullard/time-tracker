<script lang="ts">
	import * as Dialog from '$lib/components/ui/dialog';
	import { activeModal, modalRegistry } from '$lib/modal.svelte';
</script>

{#if activeModal() !== null}
	{@const m = activeModal()!}
	{@const entry = modalRegistry[m.name]}
	<Dialog.Root
		open={true}
		onOpenChange={(isOpen) => {
			if (!isOpen) m.resolve(entry.defaultResult);
		}}
	>
		<Dialog.Content showCloseButton={false}>
			<svelte:component this={entry.component} {...(m.props as any)} onResolve={m.resolve} />
		</Dialog.Content>
	</Dialog.Root>
{/if}
