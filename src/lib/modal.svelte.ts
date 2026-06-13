export type ModalVariant = {
	title: string;
	message: string;
	onConfirm: () => void;
	onCancel: () => void;
};

let _modal = $state<ModalVariant | null>(null);

export function modal(): ModalVariant | null {
	return _modal;
}

export function openConfirm(title: string, message: string): Promise<boolean> {
	return new Promise((resolve) => {
		_modal = {
			title,
			message,
			onConfirm: () => {
				_modal = null;
				resolve(true);
			},
			onCancel: () => {
				_modal = null;
				resolve(false);
			}
		};
	});
}
