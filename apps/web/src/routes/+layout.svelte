<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import './layout.css';
	import favicon from '#lib/assets/aqardesk-logo.svg';
	import { QueryClientProvider } from '@tanstack/svelte-query';
	import { Toaster } from '@aqardesk/ui';
	import { queryClient } from '#lib/core/query-client';
	import ConfirmDialog from '#lib/shared/components/confirm-dialog.svelte';
	import { createAppContext, setAppContext } from '#lib/core/app-context.svelte';
	import { deLocalizeUrl } from '#lib/paraglide/runtime';
	import * as m from '#lib/paraglide/messages';
	import { getViewPath, type ViewKey } from '#lib/shell/navigation';
	import TitleBar from '#lib/shell/title-bar.svelte';
	import SearchModal from '#lib/shell/search-modal.svelte';
	import type { UpdateProgress } from '#lib/api/desktop';
	import type { LayoutProps } from './$types';

	let { children }: LayoutProps = $props();
	const app = createAppContext();
	setAppContext(app);
	let desktopShell = $state(false);
	let searchOpen = $state(false);

	const pageTitle = $derived.by(() => {
		if (!app.currentUser) return m.app_name();
		const parts = deLocalizeUrl(page.url).pathname.split('/').filter(Boolean);
		const routes = parts.slice(1);
		const known = ['properties', 'units', 'tenants', 'contracts', 'payments', 'users', 'settings', 'account', 'organization', 'profile', 'audit', 'currencies', 'contract-templates'];
		const segment = [...routes].reverse().find((part) => known.includes(part)) ?? routes[0] ?? 'overview';
		switch (segment) {
			case 'properties': return m.nav_properties();
			case 'units': return m.nav_units();
			case 'tenants': return m.nav_tenants();
			case 'contracts': return m.nav_contracts();
			case 'payments': return m.nav_payments();
			case 'users': return m.nav_users();
			case 'settings': return m.nav_settings();
			case 'account': return m.nav_account();
			case 'organization': return m.settings_organization();
			case 'profile': return m.settings_my_profile();
			case 'audit': return m.settings_audit();
			case 'currencies': return m.settings_currencies();
			case 'contract-templates': return m.settings_contract_templates();
			default: return m.nav_overview();
		}
	});

	function selectSearchResult(view: ViewKey) {
		if (!app.currentUser) return;
		void goto(getViewPath(app.currentUser.organization_id, view));
	}

	onMount(() => {
		desktopShell = '__TAURI_INTERNALS__' in window;
		app.initializeTheme();
		void app.initialize();
		let disposed = false;
		let unlisten: UnlistenFn | undefined;
		void listen<UpdateProgress>('desktop-update-progress', (event) => {
			app.updateProgress = event.payload;
		})
			.then((stopListening) => {
				if (disposed) stopListening();
				else unlisten = stopListening;
			})
			.catch(app.setError);

		return () => {
			disposed = true;
			unlisten?.();
		};
	});
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<QueryClientProvider client={queryClient}>
	<div class="flex h-dvh flex-col overflow-hidden bg-background">
		{#if desktopShell}
			<TitleBar
				title={pageTitle}
				organizationName={app.organizationName}
				currentUser={app.currentUser}
				connected={app.connected}
				locale={app.locale}
				theme={app.theme}
				onSearch={() => (searchOpen = true)}
				onToggleLanguage={() => void app.changeLanguage(app.locale === 'ar' ? 'en' : 'ar')}
				onToggleTheme={() => app.changeTheme(app.theme === 'dark' ? 'light' : 'dark')}
			/>
		{/if}
		<div class="min-h-0 flex-1 overflow-hidden">
			{@render children()}
		</div>
	</div>
	{#if desktopShell && app.currentUser}
		<SearchModal bind:open={searchOpen} currentUser={app.currentUser} onSelect={selectSearchResult} />
	{/if}
</QueryClientProvider>
<Toaster position={app.toastPosition} />
<ConfirmDialog />
