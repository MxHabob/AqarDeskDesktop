<script lang="ts">
	import { createForm } from '@tanstack/svelte-form';
	import { z } from 'zod';
	import { untrack } from 'svelte';
	import type { Tenant, TenantInput } from '@aqardesk/sdk';
	import { Button, Input, SheetFooter } from '@aqardesk/ui';
	import FormField from '#lib/shared/components/form-field.svelte';
	import { getFieldError } from '#lib/shared/form-errors';
	import * as m from '#lib/paraglide/messages';

	type Values = { full_name: string; email: string; phone: string; mailing_address: string; notes: string };
	type Props = {
		initial?: Tenant;
		busy: boolean;
		onSubmit: (values: TenantInput) => Promise<boolean>;
		onCancel: () => void;
	};
	let { initial, busy, onSubmit, onCancel }: Props = $props();
	const defaultValues: Values = untrack(() => ({
		full_name: initial?.full_name ?? '',
		email: initial?.email ?? '',
		phone: initial?.phone ?? '',
		mailing_address: initial?.mailing_address ?? '',
		notes: initial?.notes ?? ''
	}));
	const schema = z.object({
		full_name: z.string().trim().min(1, m.tenants_name()),
		email: z.union([z.email(m.auth_email()), z.literal('')]),
		phone: z.string(),
		mailing_address: z.string(),
		notes: z.string()
	}).refine((value) => value.email.trim() || value.phone.trim(), {
		message: m.tenants_contact_required(),
		path: ['phone']
	});
	const form = createForm(() => ({
		defaultValues,
		validators: { onSubmit: schema },
		onSubmit: async ({ value }) => {
			const ok = await onSubmit({
				full_name: value.full_name.trim(),
				email: value.email.trim() || null,
				phone: value.phone.trim() || null,
				mailing_address: value.mailing_address.trim() || null,
				notes: value.notes.trim() || null
			});
			if (ok) onCancel();
		}
	}));
	function submit(event: SubmitEvent) {
		event.preventDefault();
		event.stopPropagation();
		void form.handleSubmit();
	}
</script>

<form id="tenant-form" class="flex min-h-0 flex-1 flex-col" onsubmit={submit}>
	<div class="grid min-h-0 flex-1 content-start gap-3 overflow-y-auto p-4 scrollbar-none">
	<form.Field name="full_name">{#snippet children(field)}<FormField label={m.tenants_name()} id="tenant-name" error={getFieldError(field.state.meta.errors)}><Input id="tenant-name" value={field.state.value} oninput={(event) => field.handleChange(event.currentTarget.value)} onblur={field.handleBlur} autocomplete="name" /></FormField>{/snippet}</form.Field>
	<form.Field name="email">{#snippet children(field)}<FormField label={m.auth_email()} id="tenant-email" error={getFieldError(field.state.meta.errors)}><Input id="tenant-email" type="email" value={field.state.value} oninput={(event) => field.handleChange(event.currentTarget.value)} onblur={field.handleBlur} dir="ltr" autocomplete="email" /></FormField>{/snippet}</form.Field>
	<form.Field name="phone">{#snippet children(field)}<FormField label={m.tenants_phone()} id="tenant-phone" description={m.tenants_contact_required()} error={getFieldError(field.state.meta.errors)}><Input id="tenant-phone" type="tel" value={field.state.value} oninput={(event) => field.handleChange(event.currentTarget.value)} onblur={field.handleBlur} dir="ltr" autocomplete="tel" /></FormField>{/snippet}</form.Field>
	<form.Field name="mailing_address">{#snippet children(field)}<FormField label={m.tenants_address()} id="tenant-address"><Input id="tenant-address" value={field.state.value} oninput={(event) => field.handleChange(event.currentTarget.value)} /></FormField>{/snippet}</form.Field>
	<form.Field name="notes">{#snippet children(field)}<FormField label={m.common_notes()} id="tenant-notes"><textarea id="tenant-notes" value={field.state.value} oninput={(event) => field.handleChange(event.currentTarget.value)} rows="3" class="min-h-20 w-full rounded-md border border-input bg-card px-3 py-2 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring/40"></textarea></FormField>{/snippet}</form.Field>
	</div>
</form>
<SheetFooter class="shrink-0 border-t border-border bg-popover px-4 py-3">
	<Button type="button" variant="outline" disabled={busy} onclick={onCancel}>{m.common_cancel()}</Button>
	<Button type="submit" form="tenant-form" disabled={busy}>{busy ? m.common_processing() : initial ? m.common_save() : m.tenants_add()}</Button>
</SheetFooter>
