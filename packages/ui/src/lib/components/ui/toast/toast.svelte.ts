export type ToastVariant = 'default' | 'success' | 'error' | 'info' | 'warning';

export type ToastOptions = {
	description?: string;
	duration?: number;
	variant?: ToastVariant;
};

type ToastItem = {
	id: number;
	title: string;
	description?: string;
	variant: ToastVariant;
	open: boolean;
};

class ToastManager {
	items = $state<ToastItem[]>([]);
	private nextId = 0;
	private timers = new Map<number, ReturnType<typeof setTimeout>>();

	show(title: string, options: ToastOptions = {}) {
		const item: ToastItem = {
			id: ++this.nextId,
			title,
			description: options.description,
			variant: options.variant ?? 'default',
			open: true
		};

		this.items = [item, ...this.items].slice(0, 5);
		const duration = options.duration ?? 4000;
		if (Number.isFinite(duration) && duration > 0) {
			this.scheduleDismiss(item.id, duration);
		}
		return item.id;
	}

	dismiss(id: number) {
		const item = this.items.find((entry) => entry.id === id);
		if (!item || !item.open) return;
		item.open = false;
		this.scheduleRemoval(id, 180);
	}

	private scheduleDismiss(id: number, duration: number) {
		this.clearTimer(id);
		this.timers.set(
			id,
			setTimeout(() => {
				this.timers.delete(id);
				const item = this.items.find((entry) => entry.id === id);
				if (!item) return;
				item.open = false;
				this.scheduleRemoval(id, 180);
			}, duration)
		);
	}

	private scheduleRemoval(id: number, duration: number) {
		this.clearTimer(id);
		this.timers.set(
			id,
			setTimeout(() => {
				this.timers.delete(id);
				this.items = this.items.filter((item) => item.id !== id);
			}, duration)
		);
	}

	private clearTimer(id: number) {
		const existingTimer = this.timers.get(id);
		if (!existingTimer) return;
		clearTimeout(existingTimer);
		this.timers.delete(id);
	}
}

export const toastManager = new ToastManager();

function showWithVariant(variant: ToastVariant) {
	return (title: string, options: Omit<ToastOptions, 'variant'> = {}) =>
		toastManager.show(title, { ...options, variant });
}

export const toast = Object.assign(
	(title: string, options?: ToastOptions) => toastManager.show(title, options),
	{
		success: showWithVariant('success'),
		error: showWithVariant('error'),
		info: showWithVariant('info'),
		warning: showWithVariant('warning'),
		dismiss: (id: number) => toastManager.dismiss(id)
	}
);
