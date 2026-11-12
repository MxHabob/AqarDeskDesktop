<script lang="ts">
	import { createQuery } from '@tanstack/svelte-query';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { hasPermission } from '#lib/features/auth/permissions';
	import { userQueries, userQueryKeys } from '#lib/features/users/queries';
	import AddUserSheet from '#lib/features/users/add-user-sheet.svelte';
	import EditUserSheet from '#lib/features/users/edit-user-sheet.svelte';
	import UsersDirectory from '#lib/features/users/users-directory.svelte';
	import type { User } from '#lib/features/auth/types';
	import type { UserEditorInput } from '#lib/api/desktop';
	import type { UserFormValues } from '#lib/features/users/user-form.svelte';
	import { toast } from '@aqardesk/ui';
	import { desktopApi } from '#lib/api/desktop';

	const app = getAppContext();
	const usersQuery = createQuery(() => ({
		queryKey: userQueryKeys.all(app.currentUser?.organization_id ?? ''),
		queryFn: userQueries.list,
		enabled: hasPermission(app.currentUser?.role, 'manage_administration', app.currentUser?.permissions)
	}));
	const users = $derived(usersQuery.data ?? []);
	const rolesQuery = createQuery(() => ({
		queryKey: ['role-permissions', app.currentUser?.organization_id ?? ''],
		queryFn: desktopApi.listRolePermissions,
		enabled: hasPermission(app.currentUser?.role, 'manage_administration', app.currentUser?.permissions)
	}));
	const roles = $derived(rolesQuery.data ?? []);
	let editingUser = $state<User | null>(null);
	let editSheetOpen = $state(false);
	let lastNotifiedError = '';
	$effect(() => {
		const error = usersQuery.error || rolesQuery.error
			? String(usersQuery.error ?? rolesQuery.error)
			: '';
		if (error && error !== lastNotifiedError) toast.error(error);
		lastNotifiedError = error;
	});
</script>

<div class="flex min-h-0 flex-1 flex-col">
	<UsersDirectory
		{users}
		loading={usersQuery.isLoading}
		busy={app.busy}
		onToggle={(user: User) => void app.setUserActive(user)}
		onEdit={(user: User) => {
			editingUser = user;
			editSheetOpen = true;
		}}
	>
		{#snippet toolbarActions()}
			{#if hasPermission(app.currentUser?.role, 'manage_administration', app.currentUser?.permissions)}
				<AddUserSheet {roles} busy={app.busy} onAdd={app.addUser} />
			{/if}
		{/snippet}
	</UsersDirectory>
</div>
{#if editingUser}
	<EditUserSheet
		user={editingUser}
		busy={app.busy}
		bind:open={editSheetOpen}
		onUpdate={(user: User, values: UserFormValues) => {
			const input: UserEditorInput = {
				name: values.name,
				email: values.email,
				role_id: values.role_id,
				...(values.password ? { password: values.password } : {})
			};
			return app.updateUser(user.id, input);
		}}
		{roles}
	/>
{/if}
