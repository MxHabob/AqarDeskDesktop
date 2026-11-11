<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { LoaderCircle } from '@lucide/svelte';
	import { deLocalizeUrl } from '#lib/paraglide/runtime';
	import type { LayoutProps } from './$types';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import type { ViewKey } from '#lib/shell/navigation';
	import { getViewPath } from '#lib/shell/navigation';
	import AppShell from '#lib/shell/app-shell.svelte';
	import * as m from '#lib/paraglide/messages';

	let { data, children }: LayoutProps = $props();
	const app = getAppContext();
	const organization = $derived(data.orgId);
	const routeParts = $derived(deLocalizeUrl(page.url).pathname.split('/').filter(Boolean).slice(1));
	const section = $derived(routeParts[0] ?? 'overview');
	const active = $derived(
		(['properties', 'units', 'tenants', 'contracts', 'payments', 'users', 'settings', 'account'] as string[]).includes(section)
			? (section as ViewKey)
			: 'overview'
	);
	const title = $derived.by(() => {
		const known = ['properties', 'units', 'tenants', 'contracts', 'payments', 'users', 'settings', 'account', 'organization', 'profile', 'audit', 'currencies', 'contract-templates'];
		const routeName = [...routeParts].reverse().find((part) => known.includes(part)) ?? section;
		switch (routeName) {
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

	$effect(() => {
		if (app.booting) return;
		if (!app.configured || !app.connected) {
			void goto('/connection', { replaceState: true });
		} else if (!app.currentUser) {
			void goto('/login', { replaceState: true });
		} else if (organization !== app.currentUser.organization_id) {
			void goto(`/${app.currentUser.organization_id}`, { replaceState: true });
		} else if (active === 'users' && app.currentUser.role !== 'admin') {
			void goto(getViewPath(organization, 'overview'), { replaceState: true });
		} else if (
			active === 'settings' &&
			['', 'users', 'audit', 'currencies', 'contract-templates'].includes(routeParts.slice(1).join('/')) &&
			app.currentUser.role !== 'admin'
		) {
			void goto(`/${organization}/settings/general`, { replaceState: true });
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
	<main class="grid h-dvh place-content-center justify-items-center gap-3 bg-background font-sans text-muted-foreground">
		<LoaderCircle size={22} class="animate-spin text-primary" />
		<p class="text-sm">{m.common_loading()}</p>
	</main>
{:else}
	<AppShell
		{active}
		{title}
		organizationName={app.organizationName}
		currentUser={app.currentUser}
		connected={app.connected}
		onSelect={selectView}
		onLogout={signOut}
	>
		{@render children()}
	</AppShell>
{/if}
