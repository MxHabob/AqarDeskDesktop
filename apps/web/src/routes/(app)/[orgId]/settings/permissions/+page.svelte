<script lang="ts">
	import { createInfiniteQuery } from '@tanstack/svelte-query';
	import { Button, toast } from '@aqardesk/ui';
	import { desktopApi, type RolePermissionSet } from '#lib/api/desktop';
	import { fetchNextPageOrThrow } from '#lib/core/infinite-query';
	import { confirmStore } from '#lib/core/confirm.svelte';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { queryClient } from '#lib/core/query-client';
	import { roleQueryKeys } from '#lib/features/roles/queries';
	import type { RoleFormValues } from '#lib/features/roles/role-form.svelte';
	import RoleSheet from '#lib/features/roles/role-sheet.svelte';
	import RolesDirectory from '#lib/features/roles/roles-directory.svelte';
	import * as m from '#lib/paraglide/messages';

	const app = getAppContext();
	let savingRole = $state<string | null>(null);
	let deletingRole = $state<string | null>(null);
	let editingRole = $state<RolePermissionSet | undefined>(undefined);
	let roleSheetOpen = $state(false);
	const pageSize = 50;
	const rolesQuery = createInfiniteQuery(() => ({
		queryKey: roleQueryKeys.pages(app.currentUser?.organization_id ?? ''),
		queryFn: ({ pageParam }) => desktopApi.listRolePermissionsPage(pageSize, pageParam),
		initialPageParam: 0,
		getNextPageParam: (lastPage, pages) =>
			lastPage.has_more ? pages.length * pageSize : undefined,
		enabled: !!app.currentUser
	}));
	const roles = $derived(rolesQuery.data?.pages.flatMap((page) => page.items) ?? []);
	const loading = $derived(rolesQuery.isLoading);

	async function load() {
		await queryClient.invalidateQueries({
			queryKey: roleQueryKeys.pages(app.currentUser?.organization_id ?? '')
		});
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
			await load();
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
			await load();
			toast.success(m.settings_permissions_role_deleted());
		} catch (cause) {
			app.setError(cause);
		} finally {
			deletingRole = null;
		}
	}

</script>

<div class="flex min-h-0 flex-1 flex-col gap-3">
	<RolesDirectory
		{roles}
		{loading}
		hasMore={rolesQuery.hasNextPage}
		onloadmore={() => fetchNextPageOrThrow(() => rolesQuery.fetchNextPage())}
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
