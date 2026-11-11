<script lang="ts">
	import { Building2 } from '@lucide/svelte';
	import type { Property } from '@aqardesk/sdk';
	import { Card, CardDescription, CardHeader, CardTitle } from '@aqardesk/ui';
	import * as m from '#lib/paraglide/messages';
	import { getLocale } from '#lib/paraglide/runtime';
	import { OrbisGrid, type GridActionEvent, type OrbisColumnDef } from '@aqardesk/svelte-grid';

	type Props = {
		properties: Property[];
		loading: boolean;
		readonly: boolean;
		onEdit: (property: Property) => void;
		onArchive: (property: Property) => void;
	};
	let { properties, loading, readonly, onEdit, onArchive }: Props = $props();
	const rtl = $derived(getLocale() === 'ar');

	const columns: OrbisColumnDef[] = $derived([
		{
			id: 'property',
			header: m.properties_name(),
			width: 220,
			accessor: (row) => [row.name, row.code, row.address_line_1].filter(Boolean).join(' · ')
		},
		{
			id: 'property_type',
			header: m.properties_type(),
			type: 'enum',
			options: [
				{ value: 'residential', label: m.properties_type_residential() },
				{ value: 'commercial', label: m.properties_type_commercial() },
				{ value: 'mixed_use', label: m.properties_type_mixed() },
				{ value: 'land', label: m.properties_type_land() },
				{ value: 'other', label: m.common_other() },
				{ value: 'building', label: m.properties_type_building() },
				{ value: 'house_villa', label: m.properties_type_house_villa() },
				{ value: 'investment', label: m.properties_type_investment() },
				{ value: 'warehouse', label: m.properties_type_warehouse() },
				{ value: 'yard', label: m.properties_type_yard() },
				{ value: 'apartment', label: m.properties_type_apartment() }
			]
		},
		{ id: 'city', header: m.properties_city() },
		{
			id: 'status',
			header: m.properties_status(),
			type: 'status',
			options: [
				{ value: 'active', label: m.common_enabled() },
				{ value: 'inactive', label: m.common_disabled() },
				{ value: 'archived', label: m.common_disabled() }
			]
		},
		{
			id: 'actions',
			header: m.users_action(),
			type: 'action',
			meta: {
				items: [
					{ id: 'edit', icon: 'Pencil', label: m.properties_edit(), variant: 'outline', disabled: () => readonly },
					{
						id: 'archive',
						icon: 'Trash2',
						label: m.properties_archive(),
						variant: 'destructive',
						disabled: (row: Record<string, unknown>) => readonly || row.status === 'archived'
					}
				]
			}
		}
	]);

	function handleGridAction(event: GridActionEvent) {
		const property = properties.find((item) => item.id === String(event.row.id ?? ''));
		if (!property) return;
		if (event.actionId === 'edit') onEdit(property);
		if (event.actionId === 'archive') onArchive(property);
	}
</script>

<Card class="min-w-0 rounded-lg bg-card p-4 shadow-card ring-border/50 sm:p-5">
	<CardHeader class="mb-4 flex-row items-center justify-between p-0">
		<div class="grid gap-1">
			<CardTitle class="text-base">{m.nav_properties()}</CardTitle>
			<CardDescription>{properties.length} {m.nav_properties()}</CardDescription>
		</div>
		<span class="grid size-9 place-items-center rounded-lg bg-accent text-accent-foreground"><Building2 size={18} /></span>
	</CardHeader>
	<OrbisGrid
		{columns}
		source={properties}
		{loading}
		{rtl}
		readonly
		height="360px"
		emptyTitle={m.properties_empty()}
		emptyDescription={m.dashboard_properties_description()}
		onaction={handleGridAction}
	/>
</Card>
