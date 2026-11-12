<script lang="ts">
	import { createInfiniteQuery, createQuery } from '@tanstack/svelte-query';
	import { toast } from '@aqardesk/ui';
	import type { PageProps } from './$types';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { hasPermission } from '#lib/features/auth/permissions';
	import { desktopApi } from '#lib/api/desktop';
	import { fetchNextPageOrThrow } from '#lib/core/infinite-query';
	import { confirmStore } from '#lib/core/confirm.svelte';
	import { queryClient } from '#lib/core/query-client';
	import { propertyQueryKeys } from '#lib/features/properties/queries';
	import UnitSheet from '#lib/features/units/unit-sheet.svelte';
	import UnitsDirectory from '#lib/features/units/units-directory.svelte';
	import { unitQueryKeys } from '#lib/features/units/queries';
	import type { PropertyUnit, PropertyUnitInput } from '@aqardesk/sdk';
	import * as m from '#lib/paraglide/messages';

	let { data }: PageProps = $props();
	const app = getAppContext();
	const organizationId = $derived(data.orgId);
	const canView = $derived(
		hasPermission(app.currentUser?.role, 'view_units', app.currentUser?.permissions)
	);
	const canViewProperties = $derived(
		hasPermission(app.currentUser?.role, 'view_properties', app.currentUser?.permissions)
	);
	const canCreate = $derived(
		hasPermission(app.currentUser?.role, 'create_units', app.currentUser?.permissions)
	);
	const canUpdate = $derived(
		hasPermission(app.currentUser?.role, 'update_units', app.currentUser?.permissions)
	);
	const canArchive = $derived(
		hasPermission(app.currentUser?.role, 'archive_units', app.currentUser?.permissions)
	);
	const pageSize = 50;
	const unitsQuery = createInfiniteQuery(() => ({
		queryKey: unitQueryKeys.pages(organizationId),
		queryFn: ({ pageParam }) => desktopApi.listUnitsPage(pageSize, pageParam),
		initialPageParam: 0,
		getNextPageParam: (lastPage, pages) =>
			lastPage.has_more ? pages.length * pageSize : undefined,
		enabled: !!app.currentUser && canView
	}));
	const propertiesQuery = createQuery(() => ({
		queryKey: propertyQueryKeys.all(organizationId),
		queryFn: desktopApi.listProperties,
		enabled: !!app.currentUser && canViewProperties
	}));
	let open = $state(false);
	let selectedUnit = $state<PropertyUnit | undefined>();
	let saving = $state(false);
	const units = $derived(unitsQuery.data?.pages.flatMap((page) => page.items) ?? []);
	const properties = $derived(propertiesQuery.data ?? []);
	$effect(() => {
		if (unitsQuery.error) app.setError(unitsQuery.error);
		if (canViewProperties && propertiesQuery.error) app.setError(propertiesQuery.error);
	});

	async function saveUnit(values: PropertyUnitInput) {
		saving = true;
		try {
			if (selectedUnit) await desktopApi.updateUnit(selectedUnit.id, values);
			else await desktopApi.createUnit(values);
			await queryClient.invalidateQueries({ queryKey: unitQueryKeys.all(organizationId) });
			return true;
		} catch (cause) {
			app.setError(cause);
			return false;
		} finally {
			saving = false;
		}
	}

	async function archiveUnit(unit: PropertyUnit) {
		const confirmed = await confirmStore.prompt({
			title: m.units_archive_confirm_title(),
			description: m.units_archive_confirm_description({ name: unit.unit_number }),
			confirmText: m.units_archive(),
			cancelText: m.common_cancel(),
			variant: 'destructive'
		});
		if (!confirmed) return;
		try {
			await desktopApi.archiveUnit(unit.id);
			await queryClient.invalidateQueries({ queryKey: unitQueryKeys.all(organizationId) });
			toast.success(m.units_archived());
		} catch (cause) {
			app.setError(cause);
		}
	}
</script>

{#if app.currentUser && canView}
	<div class="flex min-h-0 flex-1 flex-col">
		<UnitsDirectory
			{units}
			{properties}
			loading={unitsQuery.isLoading || propertiesQuery.isLoading}
			hasMore={unitsQuery.hasNextPage}
			onloadmore={() => fetchNextPageOrThrow(() => unitsQuery.fetchNextPage())}
			{canUpdate}
			{canArchive}
			canAdd={canCreate && properties.length > 0}
			onAdd={() => { selectedUnit = undefined; open = true; }}
			onEdit={(unit) => { selectedUnit = unit; open = true; }}
			onArchive={(unit) => void archiveUnit(unit)}
		/>
		<UnitSheet bind:open initial={selectedUnit} {properties} busy={saving} onsave={saveUnit} onclose={() => (open = false)} />
	</div>
{/if}
