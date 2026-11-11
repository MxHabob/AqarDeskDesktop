<script lang="ts">
	import { createQuery } from '@tanstack/svelte-query';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { userQueries, userQueryKeys } from '#lib/features/users/queries';
	import AddUserSheet from '#lib/features/users/add-user-sheet.svelte';
	import EditUserSheet from '#lib/features/users/edit-user-sheet.svelte';
	import UsersDirectory from '#lib/features/users/users-directory.svelte';
	import type { User } from '#lib/features/auth/types';
	import type { UserEditorInput } from '#lib/api/desktop';
	import type { UserFormValues } from '#lib/features/users/user-form.svelte';
	import { toast } from '@aqardesk/ui';

	const app = getAppContext();
	const usersQuery = createQuery(() => ({
		queryKey: userQueryKeys.all(app.currentUser?.organization_id ?? ''),
		queryFn: userQueries.list,
		enabled: app.currentUser?.role === 'admin'
	}));
	const users = $derived(usersQuery.data ?? []);
	let editingUser = $state<User | null>(null);
	let editSheetOpen = $state(false);
	let lastNotifiedError = '';
	$effect(() => {
		const error = usersQuery.error ? String(usersQuery.error) : '';
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
			{#if app.currentUser?.role === 'admin'}
				<AddUserSheet busy={app.busy} onAdd={app.addUser} />
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
				role: values.role,
				...(values.password ? { password: values.password } : {})
			};
			return app.updateUser(user.id, input);
		}}
	/>
{/if}
