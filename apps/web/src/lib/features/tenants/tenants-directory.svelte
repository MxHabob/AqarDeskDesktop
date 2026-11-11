<script lang="ts">
	import { UsersRound } from '@lucide/svelte';
	import type { Tenant } from '@aqardesk/sdk';
	import { Card, CardDescription, CardHeader, CardTitle } from '@aqardesk/ui';
	import * as m from '#lib/paraglide/messages';
	import { getLocale } from '#lib/paraglide/runtime';
	import { OrbisGrid, type GridActionEvent, type OrbisColumnDef } from '@aqardesk/svelte-grid';
	type Props = {
		tenants: Tenant[];
		loading: boolean;
		readonly: boolean;
		onEdit: (tenant: Tenant) => void;
		onArchive: (tenant: Tenant) => void;
	};
	let { tenants, loading, readonly, onEdit, onArchive }: Props = $props();
	const rtl = $derived(getLocale() === 'ar');
	const columns: OrbisColumnDef[] = $derived([
		{ id: 'full_name', header: m.tenants_name(), width: 180 },
		{ id: 'email', header: m.auth_email(), width: 220 },
		{ id: 'phone', header: m.tenants_phone(), width: 140 },
		{
			id: 'actions',
			header: m.users_action(),
			type: 'action',
			meta: {
				items: [
					{ id: 'edit', icon: 'Pencil', label: m.tenants_edit(), variant: 'outline', disabled: () => readonly },
					{ id: 'archive', icon: 'Trash2', label: m.tenants_archive(), variant: 'destructive', disabled: () => readonly }
				]
			}
		}
	]);

	function handleGridAction(event: GridActionEvent) {
		const tenant = tenants.find((item) => item.id === String(event.row.id ?? ''));
		if (!tenant) return;
		if (event.actionId === 'edit') onEdit(tenant);
		if (event.actionId === 'archive') onArchive(tenant);
	}
</script>
<Card class="min-w-0 rounded-lg bg-card p-4 shadow-card ring-border/50 sm:p-5">
	<CardHeader class="mb-4 flex-row items-center justify-between p-0"><div class="grid gap-1"><CardTitle class="text-base">{m.nav_tenants()}</CardTitle><CardDescription>{tenants.length} {m.nav_tenants()}</CardDescription></div><span class="grid size-9 place-items-center rounded-lg bg-accent text-accent-foreground"><UsersRound size={18} /></span></CardHeader>
	<OrbisGrid
		{columns}
		source={tenants}
		{loading}
		{rtl}
		readonly
		height="360px"
		emptyTitle={m.tenants_empty()}
		emptyDescription={m.dashboard_tenants_description()}
		onaction={handleGridAction}
	/>
</Card>
