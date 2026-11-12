<script lang="ts">
	import { Plus } from '@lucide/svelte';
	import { Button } from '@aqardesk/ui';
	import type { Tenant } from '@aqardesk/sdk';
	import * as m from '#lib/paraglide/messages';
	import { getLocale } from '#lib/paraglide/runtime';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { formatGridCell } from '#lib/core/i18n';
	import { AqarDeskGrid, type GridActionEvent, type AqarDeskColumnDef } from '@aqardesk/svelte-grid';
	type Props = {
		tenants: Tenant[];
		loading: boolean;
		canCreate: boolean;
		canUpdate: boolean;
		canArchive: boolean;
		onAdd: () => void;
		onEdit: (tenant: Tenant) => void;
		onArchive: (tenant: Tenant) => void;
	};
	let { tenants, loading, canCreate, canUpdate, canArchive, onAdd, onEdit, onArchive }: Props = $props();
	const app = getAppContext();
	const rtl = $derived(getLocale() === 'ar');
	const columns: AqarDeskColumnDef[] = $derived([
		{ id: 'full_name', header: m.tenants_name(), width: 180 },
		{ id: 'email', header: m.auth_email(), width: 220 },
		{ id: 'phone', header: m.tenants_phone(), width: 140 },
		{
			id: 'actions',
			header: m.users_action(),
			type: 'action',
			meta: {
				items: [
					{ id: 'edit', icon: 'Pencil', label: m.tenants_edit(), variant: 'outline', disabled: () => !canUpdate },
					{ id: 'archive', icon: 'Trash2', label: m.tenants_archive(), variant: 'destructive', disabled: () => !canArchive }
				]
			}
		}
	]);

	function handleGridAction(event: GridActionEvent) {
		const tenant = tenants.find((item) => item.id === String(event.row.id ?? ''));
		if (!tenant) return;
		if (event.actionId === 'dblclick' && canUpdate) onEdit(tenant);
		if (event.actionId === 'edit' && canUpdate) onEdit(tenant);
		if (event.actionId === 'archive' && canArchive) onArchive(tenant);
	}
</script>
<div class="min-h-0 min-w-0 flex-1">
	<AqarDeskGrid
		{columns}
		source={tenants}
		{loading}
		{rtl}
		formatCellValue={(row, column) => formatGridCell(row, column, app.organizationTimezone)}
		readonly
		height="100%"
		emptyTitle={m.tenants_empty()}
		emptyDescription={m.dashboard_tenants_description()}
		onaction={handleGridAction}
	>
		{#snippet toolbarActions()}
			<Button class="rounded-md" disabled={!canCreate} onclick={onAdd}>
				<Plus size={16} />{m.tenants_add()}
			</Button>
		{/snippet}
	</AqarDeskGrid>
</div>
