export type ToastKind = 'success' | 'error' | 'info';

export interface Toast {
	id: number;
	kind: ToastKind;
	message: string;
}

let nextId = 1;

class ToastStore {
	items = $state<Toast[]>([]);

	push(message: string, kind: ToastKind = 'info', ttl = 4000) {
		const id = nextId++;
		this.items = [...this.items, { id, kind, message }];
		setTimeout(() => this.dismiss(id), ttl);
		return id;
	}

	success(message: string) {
		return this.push(message, 'success');
	}

	error(message: string) {
		return this.push(message, 'error');
	}

	info(message: string) {
		return this.push(message, 'info');
	}

	dismiss(id: number) {
		this.items = this.items.filter((t) => t.id !== id);
	}
}

export const toasts = new ToastStore();
