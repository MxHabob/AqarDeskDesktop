<script lang="ts">
	import { UsersRound } from '@lucide/svelte';
	import {
		Button,
		Card,
		CardDescription,
		CardHeader,
		CardTitle,
		Input
	} from '@aqardesk/ui';
	import type { User } from '#lib/features/auth/types';
	import FormField from '#lib/shared/components/FormField.svelte';
	import PageHeader from '#lib/shared/components/PageHeader.svelte';
	import EmptyState from '#lib/shared/components/EmptyState.svelte';
	import StatusBadge from '#lib/shared/components/StatusBadge.svelte';
	import MessageBanner from '#lib/shared/components/MessageBanner.svelte';
	import * as m from '#lib/paraglide/messages';

	type Props = {
		users: User[];
		loading: boolean;
		queryError: string;
		busy: boolean;
		name: string;
		email: string;
		password: string;
		role: string;
		onAdd: (event: SubmitEvent) => void;
		onToggle: (user: User) => void;
		onNameChange: (value: string) => void;
		onEmailChange: (value: string) => void;
		onPasswordChange: (value: string) => void;
		onRoleChange: (value: string) => void;
	};

	let {
		users,
		loading,
		queryError,
		busy,
		name,
		email,
		password,
		role,
		onAdd,
		onToggle,
		onNameChange,
		onEmailChange,
		onPasswordChange,
		onRoleChange
	}: Props = $props();
</script>

<PageHeader title={m.users_title()} description={m.users_description()} eyebrow={m.nav_workspace()} />
{#if queryError}<MessageBanner message={queryError} />{/if}

<div class="grid items-start gap-4 lg:grid-cols-[minmax(0,1.5fr)_minmax(17rem,0.8fr)]">
	<Card class="min-w-0 rounded-lg bg-white p-4 shadow-sm ring-slate-200 sm:p-5">
		<CardHeader class="mb-4 flex-row items-center justify-between p-0">
			<div class="grid gap-1"><CardTitle class="text-base">{m.nav_users()}</CardTitle><CardDescription>{users.length} {m.nav_users()}</CardDescription></div>
			<span class="grid size-9 place-items-center rounded-lg bg-emerald-50 text-emerald-800"><UsersRound size={18} /></span>
		</CardHeader>
		{#if loading}
			<p class="py-12 text-center text-sm text-slate-500">{m.common_loading()}</p>
		{:else if users.length === 0}
			<EmptyState title={m.users_empty()} description={m.users_description()} />
		{:else}
			<div class="overflow-x-auto scrollbar-none">
				<table class="w-full min-w-[35rem] border-collapse text-start text-sm">
					<thead class="border-b border-slate-200 text-xs font-medium text-slate-500">
						<tr><th class="px-2 py-3 text-start">{m.users_name()}</th><th class="px-2 py-3 text-start">{m.users_role()}</th><th class="px-2 py-3 text-start">{m.users_status()}</th><th class="px-2 py-3 text-start">{m.users_action()}</th></tr>
					</thead>
					<tbody>
						{#each users as user (user.id)}
							<tr class="border-b border-slate-100 last:border-0">
								<td class="px-2 py-3"><div class="grid gap-1"><strong class="font-medium text-slate-800">{user.name}</strong><span class="text-xs text-slate-500" dir="ltr">{user.email}</span></div></td>
								<td class="px-2 py-3 text-slate-700">{user.role === 'admin' ? m.users_role_admin() : m.users_role_staff()}</td>
								<td class="px-2 py-3"><StatusBadge label={user.is_active ? m.common_enabled() : m.common_disabled()} active={user.is_active} /></td>
								<td class="px-2 py-3"><Button variant="outline" class="h-8 rounded-md" disabled={busy} onclick={() => onToggle(user)}>{user.is_active ? m.users_deactivate() : m.users_activate()}</Button></td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/if}
	</Card>

	<Card class="rounded-lg bg-white p-4 shadow-sm ring-slate-200 sm:p-5">
		<CardHeader class="mb-4 p-0"><CardTitle class="text-base">{m.users_add_title()}</CardTitle><CardDescription>{m.users_description()}</CardDescription></CardHeader>
		<form class="grid gap-4" onsubmit={onAdd}>
			<FormField label={m.users_name()} id="new-user-name"><Input id="new-user-name" value={name} oninput={(event) => onNameChange(event.currentTarget.value)} required autocomplete="name" class="rounded-md" /></FormField>
			<FormField label={m.auth_email()} id="new-user-email"><Input id="new-user-email" type="email" value={email} oninput={(event) => onEmailChange(event.currentTarget.value)} required dir="ltr" autocomplete="email" class="rounded-md" /></FormField>
			<FormField label={m.auth_password()} id="new-user-password" description={m.auth_password_hint()}><Input id="new-user-password" type="password" value={password} oninput={(event) => onPasswordChange(event.currentTarget.value)} required minlength={12} dir="ltr" autocomplete="new-password" class="rounded-md" /></FormField>
			<FormField label={m.users_role()} id="new-user-role">
				<select id="new-user-role" value={role} onchange={(event) => onRoleChange(event.currentTarget.value)} class="h-9 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus-visible:ring-2 focus-visible:ring-emerald-700">
					<option value="staff">{m.users_role_staff()}</option><option value="admin">{m.users_role_admin()}</option>
				</select>
			</FormField>
			<Button class="h-10 rounded-md bg-emerald-800 text-white hover:bg-emerald-900" type="submit" disabled={busy}>{m.users_add()}</Button>
		</form>
	</Card>
</div>
