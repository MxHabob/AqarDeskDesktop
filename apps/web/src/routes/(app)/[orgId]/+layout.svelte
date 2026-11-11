<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { onMount } from 'svelte';
	import type { LayoutProps } from './$types';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import type { ViewKey } from '#lib/shell/navigation';
	import { getViewPath } from '#lib/shell/navigation';
	import AppShell from '#lib/shell/AppShell.svelte';
	import MessageBanner from '#lib/shared/components/MessageBanner.svelte';
	import * as m from '#lib/paraglide/messages';

	let { data, children }: LayoutProps = $props();
	const app = getAppContext();
	const organization = $derived(data.orgId);
	const routeParts = $derived(page.url.pathname.split('/').filter(Boolean));
	const section = $derived(routeParts[1] ?? 'overview');
	const active = $derived(
		(['overview', 'properties', 'units', 'tenants', 'contracts', 'payments', 'users', 'settings'] as string[]).includes(section)
			? (section as ViewKey)
			: 'overview'
	);
	const title = $derived.by(() => {
		const routeName = routeParts.length > 3
			? routeParts[routeParts.length - 1]
			: routeParts.length === 3
				? 'properties'
				: section;
		switch (routeName) {
			case 'properties': return m.nav_properties();
			case 'units': return m.nav_units();
			case 'tenants': return m.nav_tenants();
			case 'contracts': return m.nav_contracts();
			case 'payments': return m.nav_payments();
			case 'users': return m.nav_users();
			case 'settings': return m.nav_settings();
			default: return m.nav_overview();
		}
	});

	onMount(() => {
		void app.initialize();
	});

	$effect(() => {
		if (app.booting) return;
		if (!app.configured) {
			void goto('/connection', { replaceState: true });
		} else if (!app.currentUser) {
			void goto('/login', { replaceState: true });
		} else if (organization !== app.currentUser.organization_id) {
			void goto(`/${app.currentUser.organization_id}`, { replaceState: true });
		} else if (active === 'users' && app.currentUser.role !== 'admin') {
			void goto(getViewPath(organization, 'overview'), { replaceState: true });
		}
	});

	async function signOut() {
		if (await app.signOut()) await goto('/login', { replaceState: true });
	}

	function selectView(view: ViewKey) {
		if (app.currentUser) void goto(getViewPath(app.currentUser.organization_id, view));
	}
</script>

{#if app.booting || !app.currentUser}
	<main class="grid h-dvh place-content-center justify-items-center gap-3 bg-slate-50 font-sans text-slate-700">
		<p class="text-sm">{m.common_loading()}</p>
	</main>
{:else}
	<AppShell
		{active}
		{title}
		currentUser={app.currentUser}
		connected={app.connected}
		locale={app.locale}
		onSelect={selectView}
		onLogout={signOut}
		onToggleLanguage={() => void app.changeLanguage(app.locale === 'ar' ? 'en' : 'ar')}
	>
		{#if app.error}<MessageBanner message={app.error} />{/if}
		{#if app.notice}<MessageBanner message={app.notice} variant="success" />{/if}
		{@render children()}
	</AppShell>
{/if}
