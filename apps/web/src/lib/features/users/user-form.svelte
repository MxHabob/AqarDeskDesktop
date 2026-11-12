<script lang="ts">
	import { untrack } from 'svelte';
	import { createForm } from '@tanstack/svelte-form';
	import { z } from 'zod';
	import {
		Button,
		Input,
		Select,
		SelectContent,
		SelectItem,
		SelectTrigger,
		SelectValue,
		SheetFooter
	} from '@aqardesk/ui';
	import FormField from '#lib/shared/components/form-field.svelte';
	import type { User } from '#lib/features/auth/types';
	import type { RolePermissionSet } from '#lib/api/desktop';
	import * as m from '#lib/paraglide/messages';

	export type UserFormValues = {
		name: string;
		email: string;
		password: string;
		role_id: string;
	};

	type Props = {
		initial?: User;
		roles: RolePermissionSet[];
		busy: boolean;
		roleLocked?: boolean;
		onSubmit: (values: UserFormValues) => Promise<boolean>;
		onCancel: () => void;
	};

	let { initial, roles, busy, roleLocked = false, onSubmit, onCancel }: Props = $props();

	const defaultValues = untrack(
		(): UserFormValues => ({
			name: initial?.name ?? '',
			email: initial?.email ?? '',
			password: '',
			role_id:
				initial?.role_id ??
				roles.find((role) => role.system_role === 'staff')?.role_id ??
				roles[0]?.role_id ??
				'staff'
		})
	);

	const schema = z.object({
		name: z.string().trim().min(1, m.users_name()),
		email: z.email(m.auth_email()),
		password: z.string(),
		role_id: z.string().min(1)
	}).superRefine((values, context) => {
		if ((!initial || values.password) && values.password.length < 12) {
			context.addIssue({
				code: 'custom',
				path: ['password'],
				message: m.auth_password_hint()
			});
		}
	});

	const form = createForm(() => ({
		defaultValues,
		validators: { onSubmit: schema },
		onSubmit: async ({ value }) => {
			await onSubmit(value);
		}
	}));

	function fieldError(errors: unknown[]): string | undefined {
		const first = errors[0];
		if (!first) return undefined;
		if (typeof first === 'string') return first;
		if (first instanceof Error) return first.message;
		if (typeof first === 'object' && 'message' in first && typeof first.message === 'string') {
			return first.message;
		}
		return m.error_unexpected_description();
	}

	function submit(event: SubmitEvent) {
		event.preventDefault();
		event.stopPropagation();
		void form.handleSubmit();
	}

	function setRole(value: string, change: (value: string) => void) {
		if (roles.some((role) => role.role_id === value)) change(value);
	}

	function roleTitle(role: RolePermissionSet) {
		if (role.system_role === 'admin') return m.users_role_admin();
		if (role.system_role === 'staff') return m.users_role_staff();
		if (role.system_role === 'readonly') return m.users_role_readonly();
		return role.name;
	}
</script>

<form id="user-form" class="flex min-h-0 flex-1 flex-col" onsubmit={submit}>
	<div class="grid min-h-0 flex-1 content-start gap-3 overflow-y-auto p-4 scrollbar-none">
	<form.Field name="name">
		{#snippet children(field)}
			<FormField label={m.users_name()} id="user-name" error={fieldError(field.state.meta.errors)}>
				<Input
					id="user-name"
					value={field.state.value}
					oninput={(event) => field.handleChange(event.currentTarget.value)}
					onblur={field.handleBlur}
					autocomplete="name"
					aria-invalid={field.state.meta.errors.length > 0}
					class="rounded-md"
				/>
			</FormField>
		{/snippet}
	</form.Field>
	<form.Field name="email">
		{#snippet children(field)}
			<FormField label={m.auth_email()} id="user-email" error={fieldError(field.state.meta.errors)}>
				<Input
					id="user-email"
					type="email"
					value={field.state.value}
					oninput={(event) => field.handleChange(event.currentTarget.value)}
					onblur={field.handleBlur}
					dir="ltr"
					autocomplete="email"
					aria-invalid={field.state.meta.errors.length > 0}
					class="rounded-md"
				/>
			</FormField>
		{/snippet}
	</form.Field>
	<form.Field name="password">
		{#snippet children(field)}
			<FormField
				label={m.auth_password()}
				id="user-password"
				description={initial ? m.users_password_optional() : m.auth_password_hint()}
				error={fieldError(field.state.meta.errors)}
			>
				<Input
					id="user-password"
					type="password"
					value={field.state.value}
					oninput={(event) => field.handleChange(event.currentTarget.value)}
					onblur={field.handleBlur}
					dir="ltr"
					autocomplete="new-password"
					aria-invalid={field.state.meta.errors.length > 0}
					placeholder={initial ? m.users_password_unchanged() : undefined}
					class="rounded-md"
				/>
			</FormField>
		{/snippet}
	</form.Field>
	<form.Field name="role_id">
		{#snippet children(field)}
			<FormField
				label={m.users_role()}
				id="user-role"
				description={roleLocked ? m.users_role_current_locked() : m.users_role_permissions_hint()}
			>
				<Select
					type="single"
					value={field.state.value}
					items={roles.map((role) => ({
						value: role.role_id,
						label: roleTitle(role)
					}))}
					disabled={roleLocked}
					onValueChange={(value) => setRole(value, (role) => field.handleChange(role))}
				>
					<SelectTrigger id="user-role" class="h-9 w-full rounded-md text-sm">
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						{#each roles as role (role.role_id)}
							{@const title = roleTitle(role)}
							<SelectItem value={role.role_id} label={title}>{title}</SelectItem>
						{/each}
					</SelectContent>
				</Select>
			</FormField>
		{/snippet}
	</form.Field>
	</div>
</form>
<SheetFooter class="shrink-0 border-t border-border bg-popover px-4 py-3">
	<Button type="button" variant="outline" class="rounded-md" disabled={busy} onclick={onCancel}>
		{m.common_cancel()}
	</Button>
	<Button type="submit" form="user-form" class="rounded-md" disabled={busy}>
		{busy ? m.common_processing() : initial ? m.common_save() : m.users_add()}
	</Button>
</SheetFooter>
