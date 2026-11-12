<script lang="ts">
	import { onMount } from 'svelte';
	import { Button, toast } from '@aqardesk/ui';
	import { desktopApi, type RolePermissionSet } from '#lib/api/desktop';
	import { confirmStore } from '#lib/core/confirm.svelte';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { queryClient } from '#lib/core/query-client';
	import { roleQueryKeys } from '#lib/features/roles/queries';
	import type { RoleFormValues } from '#lib/features/roles/role-form.svelte';
	import RoleSheet from '#lib/features/roles/role-sheet.svelte';
	import RolesDirectory from '#lib/features/roles/roles-directory.svelte';
	import * as m from '#lib/paraglide/messages';

	const app = getAppContext();
	let roles = $state<RolePermissionSet[]>([]);
	let loading = $state(true);
	let savingRole = $state<string | null>(null);
	let deletingRole = $state<string | null>(null);
	let editingRole = $state<RolePermissionSet | undefined>(undefined);
	let roleSheetOpen = $state(false);

	async function load(showLoading = true) {
		if (showLoading) loading = true;
		try {
			roles = await desktopApi.listRolePermissions();
		} catch (cause) {
			app.setError(cause);
		} finally {
			if (showLoading) loading = false;
		}
	}

	async function saveRole(values: RoleFormValues): Promise<boolean> {
		const role = editingRole;
		savingRole = role?.role_id ?? 'new';
		try {
			if (role) {
				await desktopApi.updateRolePermissions(role.role_id, values.permissions);
				toast.success(m.settings_permissions_saved());
			} else {
				await desktopApi.createRole(values.name, values.permissions);
				toast.success(m.settings_permissions_role_created());
			}
			await queryClient.invalidateQueries({
				queryKey: roleQueryKeys.all(app.currentUser?.organization_id ?? '')
			});
			roleSheetOpen = false;
			editingRole = undefined;
			await load(false);
			return true;
		} catch (cause) {
			app.setError(cause);
			return false;
		} finally {
			savingRole = null;
		}
	}

	function addRole() {
		editingRole = undefined;
		roleSheetOpen = true;
	}

	function editRole(role: RolePermissionSet) {
		if (role.system_role === 'admin') return;
		editingRole = role;
		roleSheetOpen = true;
	}

	async function deleteRole(role: RolePermissionSet) {
		const confirmed = await confirmStore.prompt({
			title: m.settings_permissions_delete_role_confirm_title(),
			description: m.settings_permissions_delete_role_confirm_description({ name: role.name }),
			confirmText: m.settings_permissions_delete_role(),
			cancelText: m.common_cancel(),
			variant: 'destructive'
		});
		if (!confirmed) return;

		deletingRole = role.role_id;
		try {
			await desktopApi.deleteRole(role.role_id);
			await queryClient.invalidateQueries({
				queryKey: roleQueryKeys.all(app.currentUser?.organization_id ?? '')
			});
			await load(false);
			toast.success(m.settings_permissions_role_deleted());
		} catch (cause) {
			app.setError(cause);
		} finally {
			deletingRole = null;
		}
	}

	onMount(() => {
		void load();
	});
</script>

<div class="flex min-h-0 flex-1 flex-col gap-3">
	<p class="text-sm text-muted-foreground">{m.settings_permissions_description()}</p>
	<RolesDirectory
		{roles}
		{loading}
		busy={savingRole !== null || deletingRole !== null}
		onEdit={editRole}
		onDelete={(role) => void deleteRole(role)}
	>
		{#snippet toolbarActions()}
			<Button class="shrink-0 rounded-md" onclick={addRole}>
				{m.settings_permissions_add_role()}
			</Button>
		{/snippet}
	</RolesDirectory>
</div>

<RoleSheet
	bind:open={roleSheetOpen}
	role={editingRole}
	busy={savingRole !== null}
	onSubmit={saveRole}
/>
