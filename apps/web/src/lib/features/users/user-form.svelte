<script lang="ts">
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
	import * as m from '#lib/paraglide/messages';

	export type UserFormValues = {
		name: string;
		email: string;
		password: string;
		role: 'admin' | 'staff';
	};

	type Props = {
		busy: boolean;
		onSubmit: (values: UserFormValues) => Promise<boolean>;
		onCancel: () => void;
	};

	let { busy, onSubmit, onCancel }: Props = $props();

	const defaultValues: UserFormValues = {
		name: '',
		email: '',
		password: '',
		role: 'staff'
	};

	const schema = z.object({
		name: z.string().trim().min(1, m.users_name()),
		email: z.email(m.auth_email()),
		password: z.string().min(12, m.auth_password_hint()),
		role: z.enum(['admin', 'staff'])
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

	function setRole(value: string, change: (value: UserFormValues['role']) => void) {
		if (value === 'admin' || value === 'staff') change(value);
	}
</script>

<form id="user-form" class="flex min-h-0 flex-1 flex-col" onsubmit={submit}>
	<div class="grid min-h-0 flex-1 content-start gap-3 overflow-y-auto p-4 scrollbar-none">
	<form.Field name="name">
		{#snippet children(field)}
			<FormField label={m.users_name()} id="new-user-name" error={fieldError(field.state.meta.errors)}>
				<Input
					id="new-user-name"
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
			<FormField label={m.auth_email()} id="new-user-email" error={fieldError(field.state.meta.errors)}>
				<Input
					id="new-user-email"
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
				id="new-user-password"
				description={m.auth_password_hint()}
				error={fieldError(field.state.meta.errors)}
			>
				<Input
					id="new-user-password"
					type="password"
					value={field.state.value}
					oninput={(event) => field.handleChange(event.currentTarget.value)}
					onblur={field.handleBlur}
					dir="ltr"
					autocomplete="new-password"
					aria-invalid={field.state.meta.errors.length > 0}
					class="rounded-md"
				/>
			</FormField>
		{/snippet}
	</form.Field>
	<form.Field name="role">
		{#snippet children(field)}
			<FormField label={m.users_role()} id="new-user-role">
				<Select
					type="single"
					value={field.state.value}
					onValueChange={(value) => setRole(value, (role) => field.handleChange(role))}
				>
					<SelectTrigger id="new-user-role" class="h-9 w-full rounded-md text-sm">
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						<SelectItem value="staff">{m.users_role_staff()}</SelectItem>
						<SelectItem value="admin">{m.users_role_admin()}</SelectItem>
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
		{busy ? m.common_processing() : m.users_add()}
	</Button>
</SheetFooter>
