<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { deLocalizeUrl } from '#lib/paraglide/runtime';
	import SecondaryMenu from '#lib/shell/secondary-menu.svelte';
	import type { LayoutProps } from './$types';
	import * as m from '#lib/paraglide/messages';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { hasPermission } from '#lib/features/auth/permissions';

	let { data, children }: LayoutProps = $props();
	const app = getAppContext();
	const basePath = $derived(`/${data.organizationId}/properties/${data.propertyId}`);
	const activeTab = $derived(deLocalizeUrl(page.url).pathname.slice(basePath.length).split('/').filter(Boolean)[0] ?? 'overview');
	const tabs = $derived([
		{ key: 'overview', label: m.nav_properties(), href: basePath },
		{ key: 'units', label: m.nav_units(), href: `${basePath}/units` },
		{ key: 'tenants', label: m.nav_tenants(), href: `${basePath}/tenants` },
		{ key: 'contracts', label: m.nav_contracts(), href: `${basePath}/contracts` },
		{ key: 'payments', label: m.nav_payments(), href: `${basePath}/payments` }
	].filter((tab) => canViewTab(tab.key)));

	$effect(() => {
		if (activeTab !== 'overview' && !canViewTab(activeTab)) {
			void goto(basePath, { replaceState: true });
		}
	});

	function canViewTab(key: string) {
		switch (key) {
			case 'units': return hasPermission(app.currentUser?.role, 'view_units', app.currentUser?.permissions);
			case 'tenants':
				return hasPermission(app.currentUser?.role, 'view_tenants', app.currentUser?.permissions) &&
					hasPermission(app.currentUser?.role, 'view_contracts', app.currentUser?.permissions);
			case 'contracts': return hasPermission(app.currentUser?.role, 'view_contracts', app.currentUser?.permissions);
			case 'payments':
				return hasPermission(app.currentUser?.role, 'view_payments', app.currentUser?.permissions) &&
					hasPermission(app.currentUser?.role, 'view_contracts', app.currentUser?.permissions);
			default: return true;
		}
	}

	function selectTab(key: string) {
		const tab = tabs.find((item) => item.key === key);
		if (tab) void goto(tab.href);
	}
</script>

<section class="flex min-h-0 min-w-0 flex-1 flex-col gap-3">
	<div class="flex min-w-0 items-center gap-2">
		<div class="min-w-0 flex-1">
			<SecondaryMenu
				items={tabs.map(({ key, label }) => ({ key, label }))}
				active={activeTab === '' ? 'overview' : activeTab}
				onSelect={selectTab}
			/>
		</div>
	</div>
	{@render children()}
</section>
