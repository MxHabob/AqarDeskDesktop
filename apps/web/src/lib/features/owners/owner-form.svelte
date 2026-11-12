<script lang="ts">
	import { createForm } from '@tanstack/svelte-form';
	import { z } from 'zod';
	import { untrack } from 'svelte';
	import type { Owner, OwnerInput } from '@aqardesk/sdk';
	import { Button, Input, SheetFooter } from '@aqardesk/ui';
	import FormField from '#lib/shared/components/form-field.svelte';
	import { getFieldError } from '#lib/shared/form-errors';
	import * as m from '#lib/paraglide/messages';

	type Values = { name: string; phone_number: string; phone_label: string; is_whatsapp: boolean };
	type Props = {
		initial?: Owner;
		busy: boolean;
		onSubmit: (values: OwnerInput) => Promise<boolean>;
		onCancel: () => void;
	};

	let { initial, busy, onSubmit, onCancel }: Props = $props();
	const defaultValues: Values = untrack(() => ({
		name: initial?.name ?? '',
		phone_number: initial?.phone_number ?? '',
		phone_label: initial?.phone_label ?? '',
		is_whatsapp: initial?.is_whatsapp ?? false
	}));
	const schema = z.object({
		name: z.string().trim().min(1, m.owners_invalid_name()),
		phone_number: z.string().trim().min(1, m.owners_invalid_phone()),
		phone_label: z.string(),
		is_whatsapp: z.boolean()
	});
	const form = createForm(() => ({
		defaultValues,
		validators: { onSubmit: schema },
		onSubmit: async ({ value }) => {
			const ok = await onSubmit({
				name: value.name.trim(),
				phone_number: value.phone_number.trim(),
				phone_label: value.phone_label.trim() || null,
				is_whatsapp: value.is_whatsapp
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

<form id="owner-form" class="flex min-h-0 flex-1 flex-col" onsubmit={submit}>
	<div class="grid min-h-0 flex-1 content-start gap-3 overflow-y-auto p-4 scrollbar-none">
		<form.Field name="name">
			{#snippet children(field)}
				<FormField label={m.owners_name()} id="owner-name" error={getFieldError(field.state.meta.errors)}>
					<Input id="owner-name" value={field.state.value} oninput={(event) => field.handleChange(event.currentTarget.value)} onblur={field.handleBlur} autocomplete="name" />
				</FormField>
			{/snippet}
		</form.Field>
		<form.Field name="phone_number">
			{#snippet children(field)}
				<FormField label={m.owners_phone()} id="owner-phone" error={getFieldError(field.state.meta.errors)}>
					<Input id="owner-phone" type="tel" value={field.state.value} oninput={(event) => field.handleChange(event.currentTarget.value)} onblur={field.handleBlur} dir="ltr" autocomplete="tel" />
				</FormField>
			{/snippet}
		</form.Field>
		<form.Field name="phone_label">
			{#snippet children(field)}
				<FormField label={m.owners_phone_label()} id="owner-phone-label">
					<Input id="owner-phone-label" value={field.state.value} oninput={(event) => field.handleChange(event.currentTarget.value)} />
				</FormField>
			{/snippet}
		</form.Field>
		<form.Field name="is_whatsapp">
			{#snippet children(field)}
				<label class="flex items-center gap-2 text-sm">
					<Input
						id="owner-whatsapp"
						type="checkbox"
						class="size-4 shrink-0 rounded p-0 accent-primary"
						checked={field.state.value}
						onchange={(event) => field.handleChange(event.currentTarget.checked)}
					/>
					{m.owners_whatsapp()}
				</label>
			{/snippet}
		</form.Field>
	</div>
</form>
<SheetFooter class="shrink-0 border-t border-border bg-popover px-4 py-3">
	<Button type="button" variant="outline" disabled={busy} onclick={onCancel}>{m.common_cancel()}</Button>
	<Button type="submit" form="owner-form" disabled={busy}>{busy ? m.common_processing() : initial ? m.common_save() : m.owners_add()}</Button>
</SheetFooter>
