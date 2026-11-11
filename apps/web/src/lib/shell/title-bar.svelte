<script lang="ts">
	import { onMount } from 'svelte';
	import type { Window as TauriWindow } from '@tauri-apps/api/window';
	import { Copy, Globe2, Minus, Moon, Search, Square, Sun, X } from '@lucide/svelte';
	import { Button } from '@aqardesk/ui';
	import type { AppTheme } from '#lib/core/app-context.svelte';
	import type { AppLocale } from '#lib/core/i18n';
	import type { User } from '#lib/features/auth/types';
	import * as m from '#lib/paraglide/messages';
	import TitleBreadcrumbs from './title-breadcrumbs.svelte';

	let {
		title,
		currentUser,
		connected,
		locale,
		theme,
		onSearch,
		onToggleLanguage,
		onToggleTheme
	}: {
		title: string;
		currentUser: User | null;
		connected: boolean;
		locale: AppLocale;
		theme: AppTheme;
		onSearch: () => void;
		onToggleLanguage: () => void;
		onToggleTheme: () => void;
	} = $props();

	let appWindow: TauriWindow | null = $state(null);
	let isMaximized = $state(false);

	onMount(() => {
		let disposed = false;
		let unlistenResize: (() => void) | undefined;

		void (async () => {
			try {
				const { getCurrentWindow } = await import('@tauri-apps/api/window');
				const win = getCurrentWindow();
				if (disposed) return;
				appWindow = win;
				isMaximized = await win.isMaximized();
				unlistenResize = await win.onResized(async () => {
					isMaximized = await win.isMaximized();
				});
				if (disposed) unlistenResize();
			} catch (error) {
				console.error('Unable to initialize the desktop title bar.', error);
			}
		})();

		return () => {
			disposed = true;
			unlistenResize?.();
		};
	});

	async function toggleMaximize() {
		if (!appWindow) return;
		try {
			await appWindow.toggleMaximize();
			isMaximized = await appWindow.isMaximized();
		} catch (error) {
			console.error('Unable to resize the application window.', error);
		}
	}

	async function controlWindow(action: 'minimize' | 'close') {
		if (!appWindow) return;
		try {
			await appWindow[action]();
		} catch (error) {
			console.error(`Unable to ${action} the application window.`, error);
		}
	}
</script>

<header
	class="z-50 flex h-12 shrink-0 select-none items-center justify-between gap-3 border-b border-border/70 bg-card px-3 text-xs"
	data-tauri-drag-region
>
	<div class="flex min-w-0 flex-1 items-center gap-3" data-tauri-drag-region>
		<TitleBreadcrumbs {title} />
		{#if currentUser}
			<span
				class="hidden items-center gap-1.5 rounded-full border border-border/70 bg-background px-2.5 py-1 text-[10px] text-muted-foreground sm:inline-flex"
			>
				<span class={`size-1.5 rounded-full ${connected ? 'bg-emerald-500' : 'bg-destructive'}`}></span>
				{connected ? m.status_connected() : m.status_not_connected()}
			</span>
		{/if}
	</div>

	<div class="flex shrink-0 items-center gap-1.5">
		{#if currentUser}
			<Button
				variant="outline"
				size="sm"
				class="h-8 gap-2 rounded-lg px-2.5 text-muted-foreground"
				aria-label={m.common_search()}
				title={m.titlebar_search_hint()}
				onclick={onSearch}
			>
				<Search size={14} />
				<span class="hidden sm:inline">{m.common_search()}</span>
				<kbd class="hidden rounded border border-border bg-muted px-1.5 py-0.5 font-mono text-[10px] sm:inline">Ctrl K</kbd>
			</Button>
		{/if}

		<Button
			variant="ghost"
			size="icon"
			class="size-8 rounded-lg text-muted-foreground"
			aria-label={locale === 'ar' ? m.language_english() : m.language_arabic()}
			title={m.common_language()}
			onclick={onToggleLanguage}
		>
			<Globe2 size={15} />
		</Button>

		<Button
			variant="ghost"
			size="icon"
			class="size-8 rounded-lg text-muted-foreground"
			aria-label={theme === 'dark' ? m.theme_switch_to_light() : m.theme_switch_to_dark()}
			title={theme === 'dark' ? m.theme_switch_to_light() : m.theme_switch_to_dark()}
			onclick={onToggleTheme}
		>
			{#if theme === 'dark'}<Sun size={15} />{:else}<Moon size={15} />{/if}
		</Button>

		<div class="ms-1 flex items-center border-s border-border/70 ps-1">
			<Button
				variant="ghost"
				size="icon"
				class="size-8 rounded-lg text-muted-foreground hover:bg-muted hover:text-foreground"
				aria-label={m.titlebar_minimize()}
				title={m.titlebar_minimize()}
				onclick={() => void controlWindow('minimize')}
			>
				<Minus size={13} />
			</Button>
			<Button
				variant="ghost"
				size="icon"
				class="size-8 rounded-lg text-muted-foreground hover:bg-muted hover:text-foreground"
				aria-label={isMaximized ? m.titlebar_restore() : m.titlebar_maximize()}
				title={isMaximized ? m.titlebar_restore() : m.titlebar_maximize()}
				onclick={() => void toggleMaximize()}
			>
				{#if isMaximized}<Copy size={12} />{:else}<Square size={12} />{/if}
			</Button>
			<Button
				variant="ghost"
				size="icon"
				class="size-8 rounded-lg text-muted-foreground hover:bg-destructive hover:text-destructive-foreground"
				aria-label={m.titlebar_close()}
				title={m.titlebar_close()}
				onclick={() => void controlWindow('close')}
			>
				<X size={13} />
			</Button>
		</div>
	</div>
</header>
