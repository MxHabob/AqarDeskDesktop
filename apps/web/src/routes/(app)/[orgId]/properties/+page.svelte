<script lang="ts">
	import { createQuery } from '@tanstack/svelte-query';
	import { toast } from '@aqardesk/ui';
	import type { PageProps } from './$types';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { hasPermission } from '#lib/features/auth/permissions';
	import { confirmStore } from '#lib/core/confirm.svelte';
	import { desktopApi } from '#lib/api/desktop';
	import { queryClient } from '#lib/core/query-client';
	import PropertySheet from '#lib/features/properties/property-sheet.svelte';
	import PropertiesDirectory from '#lib/features/properties/properties-directory.svelte';
	import { propertyQueryKeys } from '#lib/features/properties/queries';
	import { ownerQueryKeys } from '#lib/features/owners/queries';
	import type { Owner, Property, PropertyInput } from '@aqardesk/sdk';
	import * as m from '#lib/paraglide/messages';

	let { data }: PageProps = $props();
	const app = getAppContext();
	const organizationId = $derived(data.orgId);
	const canView = $derived(
		hasPermission(app.currentUser?.role, 'view_properties', app.currentUser?.permissions)
	);
	const canCreate = $derived(
		hasPermission(app.currentUser?.role, 'create_properties', app.currentUser?.permissions)
	);
	const canUpdate = $derived(
		hasPermission(app.currentUser?.role, 'update_properties', app.currentUser?.permissions)
	);
	const canArchive = $derived(
		hasPermission(app.currentUser?.role, 'archive_properties', app.currentUser?.permissions)
	);
	const propertiesQuery = createQuery(() => ({
		queryKey: propertyQueryKeys.all(organizationId),
		queryFn: desktopApi.listProperties,
		enabled: !!app.currentUser && canView
	}));
	const canViewOwners = $derived(
		hasPermission(app.currentUser?.role, 'view_owners', app.currentUser?.permissions)
	);
	const ownersQuery = createQuery(() => ({
		queryKey: ownerQueryKeys.all(organizationId),
		queryFn: desktopApi.listOwners,
		enabled: !!app.currentUser && canViewOwners
	}));
	let open = $state(false);
	let selectedProperty = $state<Property | undefined>();
	let saving = $state(false);
	const properties = $derived(propertiesQuery.data ?? []);
	const owners = $derived(ownersQuery.data ?? []);
	$effect(() => {
		if (propertiesQuery.error) app.setError(propertiesQuery.error);
		if (canViewOwners && ownersQuery.error) app.setError(ownersQuery.error);
	});

	async function saveProperty(values: PropertyInput) {
		saving = true;
		try {
			if (selectedProperty) {
				await desktopApi.updateProperty(selectedProperty.id, values);
			} else {
				await desktopApi.createProperty(values);
			}
			await Promise.all([
				queryClient.invalidateQueries({ queryKey: propertyQueryKeys.all(organizationId) }),
				queryClient.invalidateQueries({ queryKey: ownerQueryKeys.all(organizationId) })
			]);
			return true;
		} catch (cause) {
			app.setError(cause);
			return false;
		} finally {
			saving = false;
		}
	}

	async function archiveProperty(property: Property) {
		const confirmed = await confirmStore.prompt({
			title: m.properties_archive_confirm_title(),
			description: m.properties_archive_confirm_description({ name: property.name }),
			confirmText: m.properties_archive(),
			cancelText: m.common_cancel(),
			variant: 'destructive'
		});
		if (!confirmed) return;
		try {
			await desktopApi.archiveProperty(property.id);
			await Promise.all([
				queryClient.invalidateQueries({ queryKey: propertyQueryKeys.all(organizationId) }),
				queryClient.invalidateQueries({ queryKey: ownerQueryKeys.all(organizationId) })
			]);
			toast.success(m.properties_archived());
		} catch (cause) {
			app.setError(cause);
		}
	}
</script>

{#if app.currentUser && canView}
	<div class="flex min-h-0 flex-1 flex-col">
		<PropertiesDirectory
			properties={properties}
			loading={propertiesQuery.isLoading}
			{canCreate}
			{canUpdate}
			{canArchive}
			onAdd={() => { selectedProperty = undefined; open = true; }}
			onEdit={(property) => { selectedProperty = property; open = true; }}
			onArchive={(property) => void archiveProperty(property)}
		/>
		<PropertySheet
			bind:open
			ownerOptions={owners}
			initial={selectedProperty}
			busy={saving}
			onsave={saveProperty}
			onclose={() => (open = false)}
		/>
	</div>
{/if}
