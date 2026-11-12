<script lang="ts">
	import { Plus } from '@lucide/svelte';
	import { Button } from '@aqardesk/ui';
	import type { Owner } from '@aqardesk/sdk';
	import * as m from '#lib/paraglide/messages';
	import { getLocale } from '#lib/paraglide/runtime';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { formatGridCell } from '#lib/core/i18n';
	import {
		AqarDeskGrid,
		type AqarDeskColumnDef,
		type GridActionEvent
	} from '@aqardesk/svelte-grid';

	type Props = {
		owners: Owner[];
		loading: boolean;
		canCreate: boolean;
		canUpdate: boolean;
		onAdd: () => void;
		onEdit: (owner: Owner) => void;
	};

	let { owners, loading, canCreate, canUpdate, onAdd, onEdit }: Props = $props();
	const app = getAppContext();
	const rtl = $derived(getLocale() === 'ar');
	const columns: AqarDeskColumnDef[] = $derived([
		{ id: 'id', header: m.settings_audit_entity_id(), width: 220 },
		{ id: 'organization_id', header: m.settings_organization_id(), width: 220 },
		{ id: 'name', header: m.owners_name(), width: 200 },
		{ id: 'phone_number', header: m.owners_phone(), width: 160 },
		{ id: 'phone_label', header: m.owners_phone_label(), width: 140 },
		{
			id: 'is_whatsapp',
			header: m.owners_whatsapp(),
			type: 'status',
			width: 140,
			accessor: (row) => row.is_whatsapp ? 'yes' : 'no',
			options: [
				{ value: 'yes', label: m.common_yes() },
				{ value: 'no', label: m.common_no() }
			]
		},
		{ id: 'created_at', header: m.settings_created_at(), type: 'datetime', width: 190 },
		{ id: 'updated_at', header: m.settings_updated_at(), type: 'datetime', width: 190 },
		{
			id: 'actions',
			header: m.users_action(),
			type: 'action',
			meta: {
				items: [
					{ id: 'edit', icon: 'Pencil', label: m.owners_edit(), variant: 'outline', disabled: () => !canUpdate }
				]
			}
		}
	]);

	function handleGridAction(event: GridActionEvent) {
		const owner = owners.find((item) => item.id === String(event.row.id ?? ''));
		if (!owner) return;
		if ((event.actionId === 'dblclick' || event.actionId === 'edit') && canUpdate) onEdit(owner);
	}
</script>

<div class="min-h-0 min-w-0 flex-1">
	<AqarDeskGrid
		{columns}
		source={owners}
		{loading}
		{rtl}
		formatCellValue={(row, column) => formatGridCell(row, column, app.organizationTimezone)}
		readonly
		height="100%"
		emptyTitle={m.owners_empty()}
		emptyDescription={m.properties_owners()}
		onaction={handleGridAction}
	>
		{#snippet toolbarActions()}
			<Button class="rounded-md" disabled={!canCreate} onclick={onAdd}>
				<Plus size={16} />{m.owners_add()}
			</Button>
		{/snippet}
	</AqarDeskGrid>
</div>
