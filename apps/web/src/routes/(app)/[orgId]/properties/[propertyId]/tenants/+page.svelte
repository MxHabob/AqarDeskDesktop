<script lang="ts">
	import { goto } from '$app/navigation';
	import { createQuery } from '@tanstack/svelte-query';
	import type { PageProps } from './$types';
	import { desktopApi } from '#lib/api/desktop';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { hasPermission } from '#lib/features/auth/permissions';
	import { queryClient } from '#lib/core/query-client';
	import TenantSheet from '#lib/features/tenants/tenant-sheet.svelte';
	import { tenantQueryKeys } from '#lib/features/tenants/queries';
	import type { TenantInput } from '@aqardesk/sdk';
	import { contractDocumentQueryKeys } from '#lib/features/contracts/queries';
	import PropertyTenantsDirectory from '#lib/features/tenants/property-tenants-directory.svelte';
	import * as m from '#lib/paraglide/messages';

	let { data }: PageProps = $props();
	const app = getAppContext();
	const organizationId = $derived(data.organizationId);
	const canViewTenants = $derived(
		hasPermission(app.currentUser?.role, 'view_tenants', app.currentUser?.permissions)
	);
	const canViewContracts = $derived(
		hasPermission(app.currentUser?.role, 'view_contracts', app.currentUser?.permissions)
	);
	const canCreateTenants = $derived(
		hasPermission(app.currentUser?.role, 'create_tenants', app.currentUser?.permissions)
	);
	const canCreateContracts = $derived(
		hasPermission(app.currentUser?.role, 'create_contracts', app.currentUser?.permissions)
	);
	const canAddTenant = $derived(canCreateTenants && canCreateContracts);
	const contractsQuery = createQuery(() => ({
		queryKey: contractDocumentQueryKeys.property(organizationId, data.propertyId),
		queryFn: () => desktopApi.listContractDocuments(data.propertyId),
		enabled: !!app.currentUser && canViewTenants && canViewContracts
	}));
	const contracts = $derived(contractsQuery.data ?? []);
	const loading = $derived(contractsQuery.isLoading);
	let tenantSheetOpen = $state(false);
	let savingTenant = $state(false);

	$effect(() => {
		if (contractsQuery.error) app.setError(contractsQuery.error);
	});

	async function saveTenant(values: TenantInput) {
		if (!canAddTenant) {
			app.setError(new Error('Creating a property-linked tenant requires tenant and contract permissions.'));
			return false;
		}
		savingTenant = true;
		try {
			const tenant = await desktopApi.createTenant(values);
			await queryClient.invalidateQueries({ queryKey: tenantQueryKeys.all(organizationId) });
			await goto(
				`/${organizationId}/properties/${data.propertyId}/contracts?tenantId=${encodeURIComponent(tenant.id)}&create=1`
			);
			return true;
		} catch (cause) {
			app.setError(cause);
			return false;
		} finally {
			savingTenant = false;
		}
	}
</script>

{#if app.currentUser && canViewTenants && canViewContracts}
	<div class="flex min-h-0 min-w-0 flex-1 flex-col">
		<PropertyTenantsDirectory
			{contracts}
			{loading}
			canAdd={canAddTenant}
			onAdd={() => (tenantSheetOpen = true)}
			emptyDescription={m.properties_tenants_empty_description()}
		/>
	</div>
{/if}
<TenantSheet
	bind:open={tenantSheetOpen}
	busy={savingTenant}
	onsave={saveTenant}
	onclose={() => (tenantSheetOpen = false)}
/>
