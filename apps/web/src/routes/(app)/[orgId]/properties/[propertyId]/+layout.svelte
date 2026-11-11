<script lang="ts">
	import { page } from '$app/state';
	import type { LayoutProps } from './$types';
	import * as m from '#lib/paraglide/messages';

	let { data, children }: LayoutProps = $props();
	const basePath = $derived(`/${data.organizationId}/properties/${data.propertyId}`);
	const activeTab = $derived(page.url.pathname.slice(basePath.length).split('/').filter(Boolean)[0] ?? 'overview');
	const tabs = $derived([
		{ key: 'overview', label: m.nav_properties(), href: basePath },
		{ key: 'units', label: m.nav_units(), href: `${basePath}/units` },
		{ key: 'tenants', label: m.nav_tenants(), href: `${basePath}/tenants` },
		{ key: 'contracts', label: m.nav_contracts(), href: `${basePath}/contracts` },
		{ key: 'payments', label: m.nav_payments(), href: `${basePath}/payments` }
	]);
</script>

<section class="grid gap-4">
	<nav class="flex gap-1 overflow-x-auto border-b border-slate-200 scrollbar-none" aria-label={m.nav_properties()}>
		{#each tabs as tab (tab.key)}
			<a
				href={tab.href}
				aria-current={activeTab === tab.key ? 'page' : undefined}
				class={activeTab === tab.key
					? 'shrink-0 border-b-2 border-emerald-800 px-3 py-2 text-sm font-medium text-emerald-900'
					: 'shrink-0 border-b-2 border-transparent px-3 py-2 text-sm text-slate-600 hover:text-slate-900'}
			>
				{tab.label}
			</a>
		{/each}
	</nav>
	{@render children()}
</section>
