<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { deLocalizeUrl } from '#lib/paraglide/runtime';
	import { createQuery } from '@tanstack/svelte-query';
	import { Button } from '@aqardesk/ui';
	import { Pencil } from '@lucide/svelte';
	import SecondaryMenu from '#lib/shell/secondary-menu.svelte';
	import type { LayoutProps } from './$types';
	import * as m from '#lib/paraglide/messages';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { hasPermission } from '#lib/features/auth/permissions';
	import { desktopApi } from '#lib/api/desktop';
	import { queryClient } from '#lib/core/query-client';
	import { propertyQueryKeys } from '#lib/features/properties/queries';
	import { ownerQueryKeys } from '#lib/features/owners/queries';
	import PropertySheet from '#lib/features/properties/property-sheet.svelte';
	import type { PropertyInput } from '@aqardesk/sdk';

	let { data, children }: LayoutProps = $props();
	const app = getAppContext();
	let propertySheetOpen = $state(false);
	let savingProperty = $state(false);
	const basePath = $derived(`/${data.organizationId}/properties/${data.propertyId}`);
	const canUpdateProperty = $derived(
		hasPermission(app.currentUser?.role, 'update_properties', app.currentUser?.permissions)
	);
	const canViewOwners = $derived(
		hasPermission(app.currentUser?.role, 'view_owners', app.currentUser?.permissions)
	);
	const propertiesQuery = createQuery(() => ({
		queryKey: propertyQueryKeys.all(data.organizationId),
		queryFn: desktopApi.listProperties,
		enabled: !!app.currentUser
	}));
	const ownersQuery = createQuery(() => ({
		queryKey: ownerQueryKeys.all(data.organizationId),
		queryFn: desktopApi.listOwners,
		enabled: !!app.currentUser && canViewOwners && propertySheetOpen
	}));
	const property = $derived(
		(propertiesQuery.data ?? []).find((item) => item.id === data.propertyId)
	);
	const owners = $derived(ownersQuery.data ?? []);
	const activeTab = $derived(deLocalizeUrl(page.url).pathname.slice(basePath.length).split('/').filter(Boolean)[0] ?? 'overview');
	const tabs = $derived([
		{ key: 'overview', label: m.nav_properties(), href: basePath },
		{ key: 'units', label: m.nav_units(), href: `${basePath}/units` },
		{ key: 'tenants', label: m.nav_tenants(), href: `${basePath}/tenants` },
		{ key: 'contracts', label: m.nav_contracts(), href: `${basePath}/contracts` },
		{ key: 'payments', label: m.nav_payments(), href: `${basePath}/payments` }
	].filter((tab) => canViewTab(tab.key)));

	$effect(() => {
		if (propertiesQuery.error) app.setError(propertiesQuery.error);
		if (canViewOwners && propertySheetOpen && ownersQuery.error) app.setError(ownersQuery.error);
	});

	$effect(() => {
		if (activeTab !== 'overview' && !canViewTab(activeTab)) {
			void goto(basePath, { replaceState: true });
		}
	});

	async function saveProperty(values: PropertyInput) {
		if (!property) {
			app.setError(new Error('The property is not available for editing.'));
			return false;
		}
		savingProperty = true;
		try {
			await desktopApi.updateProperty(property.id, values);
			await Promise.all([
				queryClient.invalidateQueries({ queryKey: propertyQueryKeys.all(data.organizationId) }),
				queryClient.invalidateQueries({ queryKey: ownerQueryKeys.all(data.organizationId) })
			]);
			return true;
		} catch (cause) {
			app.setError(cause);
			return false;
		} finally {
			savingProperty = false;
		}
	}

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
		<Button
			class="shrink-0"
			variant="outline"
			disabled={!canUpdateProperty || !property}
			onclick={() => (propertySheetOpen = true)}
		>
			<Pencil size={16} />{m.properties_edit()}
		</Button>
	</div>
	{@render children()}
</section>

<PropertySheet
	bind:open={propertySheetOpen}
	initial={property}
	ownerOptions={owners}
	busy={savingProperty}
	onsave={saveProperty}
	onclose={() => (propertySheetOpen = false)}
/>
