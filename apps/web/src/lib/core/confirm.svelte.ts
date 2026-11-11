export type ConfirmOptions = {
	title: string;
	description: string;
	confirmText: string;
	cancelText: string;
	variant?: 'default' | 'destructive';
};

class ConfirmStore {
	isOpen = $state(false);
	options = $state<ConfirmOptions | null>(null);
	private resolvePromise: ((confirmed: boolean) => void) | null = null;

	prompt(options: ConfirmOptions): Promise<boolean> {
		if (this.resolvePromise) {
			throw new Error('A confirmation prompt is already open.');
		}

		this.options = options;
		this.isOpen = true;
		return new Promise((resolve) => {
			this.resolvePromise = resolve;
		});
	}

	respond(confirmed: boolean): void {
		const resolve = this.resolvePromise;
		this.resolvePromise = null;
		this.options = null;
		this.isOpen = false;
		resolve?.(confirmed);
	}

	cancelIfClosed(): void {
		if (!this.isOpen && this.resolvePromise) {
			this.respond(false);
		}
	}
}

export const confirmStore = new ConfirmStore();
