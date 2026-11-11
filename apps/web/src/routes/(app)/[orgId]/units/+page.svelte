<script lang="ts">
	import { createQuery } from '@tanstack/svelte-query';
	import { Plus } from '@lucide/svelte';
	import { Button, toast } from '@aqardesk/ui';
	import type { PageProps } from './$types';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { desktopApi } from '#lib/api/desktop';
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
	const unitsQuery = createQuery(() => ({
		queryKey: unitQueryKeys.all(organizationId),
		queryFn: desktopApi.listUnits,
		enabled: !!app.currentUser
	}));
	const propertiesQuery = createQuery(() => ({
		queryKey: propertyQueryKeys.all(organizationId),
		queryFn: desktopApi.listProperties,
		enabled: !!app.currentUser
	}));
	let open = $state(false);
	let selectedUnit = $state<PropertyUnit | undefined>();
	let saving = $state(false);
	const units = $derived(unitsQuery.data ?? []);
	const properties = $derived(propertiesQuery.data ?? []);
	$effect(() => {
		if (unitsQuery.error) app.setError(unitsQuery.error);
		if (propertiesQuery.error) app.setError(propertiesQuery.error);
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

{#if app.currentUser}
	<div class="grid gap-4">
		<div class="flex justify-end"><Button class="rounded-md" disabled={app.currentUser.role === 'readonly' || properties.length === 0} onclick={() => { selectedUnit = undefined; open = true; }}><Plus size={16} />{m.units_add()}</Button></div>
		<UnitsDirectory
			{units}
			{properties}
			loading={unitsQuery.isLoading || propertiesQuery.isLoading}
			readonly={app.currentUser.role === 'readonly'}
			onEdit={(unit) => { selectedUnit = unit; open = true; }}
			onArchive={(unit) => void archiveUnit(unit)}
		/>
		<UnitSheet bind:open initial={selectedUnit} {properties} busy={saving} onsave={saveUnit} onclose={() => (open = false)} />
	</div>
{/if}
