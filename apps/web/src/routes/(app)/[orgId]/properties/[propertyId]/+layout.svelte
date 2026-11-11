<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { deLocalizeUrl } from '#lib/paraglide/runtime';
	import SecondaryMenu from '#lib/shell/secondary-menu.svelte';
	import type { LayoutProps } from './$types';
	import * as m from '#lib/paraglide/messages';

	let { data, children }: LayoutProps = $props();
	const basePath = $derived(`/${data.organizationId}/properties/${data.propertyId}`);
	const activeTab = $derived(deLocalizeUrl(page.url).pathname.slice(basePath.length).split('/').filter(Boolean)[0] ?? 'overview');
	const tabs = $derived([
		{ key: 'overview', label: m.nav_properties(), href: basePath },
		{ key: 'units', label: m.nav_units(), href: `${basePath}/units` },
		{ key: 'tenants', label: m.nav_tenants(), href: `${basePath}/tenants` },
		{ key: 'contracts', label: m.nav_contracts(), href: `${basePath}/contracts` },
		{ key: 'payments', label: m.nav_payments(), href: `${basePath}/payments` }
	]);

	function selectTab(key: string) {
		const tab = tabs.find((item) => item.key === key);
		if (tab) void goto(tab.href);
	}
</script>

<section class="grid gap-4">
	<SecondaryMenu
		items={tabs.map(({ key, label }) => ({ key, label }))}
		active={activeTab === '' ? 'overview' : activeTab}
		onSelect={selectTab}
	/>
	{@render children()}
</section>
