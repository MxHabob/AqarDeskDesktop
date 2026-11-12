<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { User } from '#lib/features/auth/types';
	import * as m from '#lib/paraglide/messages';
	import { getLocale } from '#lib/paraglide/runtime';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { formatGridCell } from '#lib/core/i18n';
	import { isPermission, permissionLabel } from '#lib/features/auth/permissions';
	import { AqarDeskGrid, type GridActionEvent, type AqarDeskColumnDef } from '@aqardesk/svelte-grid';

	type Props = {
		users: User[];
		loading: boolean;
		busy: boolean;
		hasMore?: boolean;
		onloadmore?: () => Promise<unknown>;
		toolbarActions?: Snippet;
		onToggle: (user: User) => void;
		onEdit: (user: User) => void;
	};

	let {
		users,
		loading,
		busy,
		hasMore = false,
		onloadmore,
		toolbarActions,
		onToggle,
		onEdit
	}: Props = $props();
	const app = getAppContext();
	const rtl = $derived(getLocale() === 'ar');
	const columns: AqarDeskColumnDef[] = $derived([
		{ id: 'id', header: m.settings_audit_entity_id(), width: 220 },
		{ id: 'organization_id', header: m.settings_organization_id(), width: 220 },
		{ id: 'name', header: m.users_name(), width: 180 },
		{ id: 'email', header: m.auth_email(), width: 220 },
		{ id: 'role_id', header: m.settings_role_id(), width: 220 },
		{ id: 'role_name', header: m.users_role(), width: 180 },
		{
			id: 'role',
			header: m.settings_system_role(),
			accessor: (row: Record<string, unknown>) => {
				if (typeof row.role_name === 'string') return row.role_name;
				if (row.role === 'admin') return m.users_role_admin();
				if (row.role === 'readonly') return m.users_role_readonly();
				return m.users_role_staff();
			}
		},
		{
			id: 'permissions',
			header: m.settings_permissions(),
			width: 280,
			accessor: (row: Record<string, unknown>) => formatPermissions(row.permissions)
		},
		{
			id: 'is_active',
			header: m.users_status(),
			type: 'status',
			accessor: (row) => row.is_active ? 'active' : 'inactive',
			options: [
				{ value: 'active', label: m.common_enabled() },
				{ value: 'inactive', label: m.common_disabled() }
			]
		},
		{ id: 'created_at', header: m.settings_created_at(), type: 'datetime', width: 190 },
		{
			id: 'actions',
			header: m.users_action(),
			type: 'action',
			meta: {
				items: [
					{
						id: 'edit',
						icon: 'Pencil',
						label: m.users_edit(),
						variant: 'outline',
						disabled: () => busy
					},
					{
						id: 'deactivate',
						icon: 'Shield',
						label: m.users_deactivate(),
						variant: 'outline',
						visible: (row: Record<string, unknown>) => row.is_active === true,
						disabled: () => busy
					},
					{
						id: 'activate',
						icon: 'Check',
						label: m.users_activate(),
						variant: 'outline',
						visible: (row: Record<string, unknown>) => row.is_active !== true,
						disabled: () => busy
					}
				]
			}
		}
	]);

	function handleGridAction(event: GridActionEvent) {
		const user = users.find((item) => item.id === String(event.row.id ?? ''));
		if (!user) return;
		if (event.actionId === 'dblclick') {
			onEdit(user);
		} else if (event.actionId === 'edit') {
			onEdit(user);
		} else if (event.actionId === 'activate' || event.actionId === 'deactivate') {
			onToggle(user);
		}
	}

	function formatPermissions(value: unknown): string {
		if (!Array.isArray(value)) return '';
		return value
			.map((permission: unknown) => isPermission(permission) ? permissionLabel(permission) : '')
			.filter(Boolean)
			.join(' · ');
	}

</script>

<div class="min-h-0 min-w-0 flex-1">
	<AqarDeskGrid
		{columns}
		source={users}
		{loading}
		{rtl}
		formatCellValue={(row, column) => formatGridCell(row, column, app.organizationTimezone)}
		readonly
		height="100%"
		{hasMore}
		{onloadmore}
		emptyTitle={m.users_empty()}
		emptyDescription={m.users_description()}
		onaction={handleGridAction}
		{toolbarActions}
	/>
</div>
