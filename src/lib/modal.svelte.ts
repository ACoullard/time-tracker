import type { Component } from 'svelte';
import ConfirmModal from '$lib/components/modals/confirm.svelte';

export type ModalMap = {
	confirm: { props: { title: string; message: string }; result: boolean };
};

type RegistryEntry = {
	component: Component<any>;
	defaultResult: unknown;
};

export const modalRegistry: Record<keyof ModalMap, RegistryEntry> = {
	confirm: { component: ConfirmModal, defaultResult: false },
};

type ModalState = {
	name: keyof ModalMap;
	props: unknown;
	resolve: (result: unknown) => void;
};

let _modal = $state<ModalState | null>(null);

export function activeModal(): ModalState | null {
	return _modal;
}

export function openModal<K extends keyof ModalMap>(
	name: K,
	props: ModalMap[K]['props']
): Promise<ModalMap[K]['result']> {
	return new Promise((resolve) => {
		_modal = {
			name,
			props,
			resolve: (result) => {
				_modal = null;
				(resolve as (v: unknown) => void)(result);
			}
		};
	});
}
