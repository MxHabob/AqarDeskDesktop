<script lang="ts">
	import { createQuery } from '@tanstack/svelte-query';
	import type { PageProps } from './$types';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { hasPermission } from '#lib/features/auth/permissions';
	import { desktopApi } from '#lib/api/desktop';
	import { queryClient } from '#lib/core/query-client';
	import { propertyQueryKeys } from '#lib/features/properties/queries';
	import { ownerQueryKeys } from '#lib/features/owners/queries';
	import OwnerSheet from '#lib/features/owners/owner-sheet.svelte';
	import OwnersDirectory from '#lib/features/owners/owners-directory.svelte';
	import type { Owner, OwnerInput } from '@aqardesk/sdk';

	let { data }: PageProps = $props();
	const app = getAppContext();
	const organizationId = $derived(data.orgId);
	const canView = $derived(
		hasPermission(app.currentUser?.role, 'view_owners', app.currentUser?.permissions)
	);
	const canCreate = $derived(
		hasPermission(app.currentUser?.role, 'create_owners', app.currentUser?.permissions)
	);
	const canUpdate = $derived(
		hasPermission(app.currentUser?.role, 'update_owners', app.currentUser?.permissions)
	);
	const ownersQuery = createQuery(() => ({
		queryKey: ownerQueryKeys.all(organizationId),
		queryFn: desktopApi.listOwners,
		enabled: !!app.currentUser && canView
	}));
	let open = $state(false);
	let selectedOwner = $state<Owner | undefined>();
	let saving = $state(false);
	const owners = $derived(ownersQuery.data ?? []);

	async function saveOwner(values: OwnerInput) {
		saving = true;
		try {
			if (selectedOwner) await desktopApi.updateOwner(selectedOwner.id, values);
			else await desktopApi.createOwner(values);
			await Promise.all([
				queryClient.invalidateQueries({ queryKey: ownerQueryKeys.all(organizationId) }),
				queryClient.invalidateQueries({ queryKey: propertyQueryKeys.all(organizationId) })
			]);
			return true;
		} catch (cause) {
			app.setError(cause);
			return false;
		} finally {
			saving = false;
		}
	}

	$effect(() => {
		if (ownersQuery.error) app.setError(ownersQuery.error);
	});
</script>

{#if app.currentUser && canView}
	<div class="flex min-h-0 flex-1 flex-col">
		<OwnersDirectory
			{owners}
			loading={ownersQuery.isLoading}
			{canCreate}
			{canUpdate}
			onAdd={() => { selectedOwner = undefined; open = true; }}
			onEdit={(owner) => { selectedOwner = owner; open = true; }}
		/>
		<OwnerSheet bind:open initial={selectedOwner} busy={saving} onsave={saveOwner} onclose={() => (open = false)} />
	</div>
{/if}
