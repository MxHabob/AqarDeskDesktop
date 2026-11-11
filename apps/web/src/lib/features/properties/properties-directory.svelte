<script lang="ts">
	import { Plus } from '@lucide/svelte';
	import type { Snippet } from 'svelte';
	import type { Property } from '@aqardesk/sdk';
	import { Button } from '@aqardesk/ui';
	import * as m from '#lib/paraglide/messages';
	import { getLocale } from '#lib/paraglide/runtime';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { formatGridCell } from '#lib/core/i18n';
	import { OrbisGrid, type GridActionEvent, type OrbisColumnDef } from '@aqardesk/svelte-grid';

	type Props = {
		properties: Property[];
		loading: boolean;
		readonly: boolean;
		onAdd: () => void;
		onEdit: (property: Property) => void;
		onArchive: (property: Property) => void;
	};
	let { properties, loading, readonly, onAdd, onEdit, onArchive }: Props = $props();
	const app = getAppContext();
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

<div class="min-h-0 min-w-0 flex-1">
	<OrbisGrid
		{columns}
		source={properties}
		{loading}
		{rtl}
		formatCellValue={(row, column) => formatGridCell(row, column, app.organizationTimezone)}
		readonly
		height="100%"
		emptyTitle={m.properties_empty()}
		emptyDescription={m.dashboard_properties_description()}
		onaction={handleGridAction}
	>
		{#snippet toolbarActions()}
			<Button class="rounded-md" disabled={readonly} onclick={onAdd}>
				<Plus size={16} />{m.properties_add()}
			</Button>
		{/snippet}
	</OrbisGrid>
</div>
