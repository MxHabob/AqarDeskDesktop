<script lang="ts">
	import type { Snippet } from 'svelte';
	import { getLocale } from '#lib/paraglide/runtime';
	import { formatGridCell } from '#lib/core/i18n';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import type { RolePermissionSet } from '#lib/api/desktop';
	import { permissionLabel } from '#lib/features/auth/permissions';
	import { AqarDeskGrid, type AqarDeskColumnDef, type GridActionEvent } from '@aqardesk/svelte-grid';
	import * as m from '#lib/paraglide/messages';

	type RoleRow = {
		id: string;
		role_id: string;
		name: string;
		type: string;
		permissions: string;
		is_system: boolean;
		system_role: RolePermissionSet['system_role'];
	};

	type Props = {
		roles: RolePermissionSet[];
		loading: boolean;
		busy: boolean;
		toolbarActions?: Snippet;
		onEdit: (role: RolePermissionSet) => void;
		onDelete: (role: RolePermissionSet) => void;
	};

	let { roles, loading, busy, toolbarActions, onEdit, onDelete }: Props = $props();
	const app = getAppContext();
	const rtl = $derived(getLocale() === 'ar');

	function title(role: RolePermissionSet) {
		switch (role.system_role) {
			case 'admin': return m.settings_permissions_admin();
			case 'staff': return m.settings_permissions_staff();
			case 'readonly': return m.settings_permissions_readonly();
			default: return role.name;
		}
	}

	function permissionNames(role: RolePermissionSet) {
		return role.permissions.map(permissionLabel).join(' · ');
	}

	const rows = $derived(
		roles.map((role): RoleRow => ({
			id: role.role_id,
			role_id: role.role_id,
			name: title(role),
			type: role.is_system
				? m.settings_permissions_system_role()
				: m.settings_permissions_custom_role(),
			permissions: permissionNames(role),
			is_system: role.is_system,
			system_role: role.system_role
		}))
	);

	const columns: AqarDeskColumnDef[] = $derived([
		{ id: 'name', header: m.users_role(), width: 180 },
		{ id: 'type', header: m.settings_permissions_role_type(), width: 140 },
		{ id: 'permissions', header: m.settings_permissions(), width: 360 },
		{
			id: 'actions',
			header: m.users_action(),
			type: 'action',
			meta: {
				items: [
					{
						id: 'edit',
						icon: 'Pencil',
						label: m.settings_permissions_edit_role(),
						variant: 'outline',
						visible: (row: Record<string, unknown>) => row.system_role !== 'admin',
						disabled: () => busy
					},
					{
						id: 'delete',
						icon: 'Trash2',
						label: m.settings_permissions_delete_role(),
						variant: 'outline',
						visible: (row: Record<string, unknown>) => row.is_system === false,
						disabled: () => busy
					}
				]
			}
		}
	]);

	function handleAction(event: GridActionEvent) {
		const role = roles.find((item) => item.role_id === String(event.row.id ?? ''));
		if (!role) return;
		if (event.actionId === 'edit' || event.actionId === 'dblclick') onEdit(role);
		if (event.actionId === 'delete') onDelete(role);
	}
</script>

<div class="min-h-0 min-w-0 flex-1">
	<AqarDeskGrid
		{columns}
		source={rows}
		{loading}
		{rtl}
		formatCellValue={(row, column) => formatGridCell(row, column, app.organizationTimezone)}
		readonly
		height="100%"
		emptyTitle={m.settings_permissions_roles_empty()}
		emptyDescription={m.settings_permissions_description()}
		onaction={handleAction}
		{toolbarActions}
	/>
</div>
