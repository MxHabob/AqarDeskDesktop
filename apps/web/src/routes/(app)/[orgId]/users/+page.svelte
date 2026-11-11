<script lang="ts">
	import { createQuery } from '@tanstack/svelte-query';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { userQueries, userQueryKeys } from '#lib/features/users/queries';
	import UsersPage from '#lib/features/users/UsersPage.svelte';
	import type { User } from '#lib/features/auth/types';

	const app = getAppContext();
	const usersQuery = createQuery(() => ({
		queryKey: userQueryKeys.all(app.currentUser?.organization_id ?? ''),
		queryFn: userQueries.list,
		enabled: app.currentUser?.role === 'admin'
	}));
	const users = $derived(usersQuery.data ?? []);
</script>

<UsersPage
	{users}
	loading={usersQuery.isLoading}
	queryError={usersQuery.error ? String(usersQuery.error) : ''}
	busy={app.busy}
	name={app.newUserName}
	email={app.newUserEmail}
	password={app.newUserPassword}
	role={app.newUserRole}
	onAdd={app.addUser}
	onToggle={(user: User) => void app.setUserActive(user)}
	onNameChange={(value) => (app.newUserName = value)}
	onEmailChange={(value) => (app.newUserEmail = value)}
	onPasswordChange={(value) => (app.newUserPassword = value)}
	onRoleChange={(value) => (app.newUserRole = value)}
/>
