<script lang="ts">
	import { createForm } from '@tanstack/svelte-form';
	import { untrack } from 'svelte';
	import { z } from 'zod';
	import type { ContractTemplate, ContractTemplateInput } from '@aqardesk/sdk';
	import { Button, Input, Select, SelectContent, SelectItem, SelectTrigger, SelectValue, SheetFooter } from '@aqardesk/ui';
	import FormField from '#lib/shared/components/form-field.svelte';
	import { getFieldError } from '#lib/shared/form-errors';
	import * as m from '#lib/paraglide/messages';

	type Values = {
		template_key: string;
		name: string;
		description: string;
		lease_type: string;
		locale: string;
		content_format: string;
		content_body: string;
		placeholders_json: string;
		change_note: string;
	};

	type Props = {
		initial?: ContractTemplate;
		busy: boolean;
		onSubmit: (input: ContractTemplateInput) => Promise<boolean>;
		onCancel: () => void;
	};

	let { initial, busy, onSubmit, onCancel }: Props = $props();
	const defaultValues: Values = untrack(() => ({
		template_key: initial?.template_key ?? '',
		name: initial?.name ?? '',
		description: initial?.description ?? '',
		lease_type: initial?.lease_type ?? 'residential',
		locale: initial?.locale ?? 'ar',
		content_format: initial?.versions[0]?.content_format ?? 'markdown',
		content_body: initial?.versions[0]?.content_body ?? '',
		placeholders_json: initial?.versions[0]?.placeholders_json ?? '{}',
		change_note: ''
	}));
	const schema = z.object({
		template_key: z
			.string()
			.trim()
			.toLowerCase()
			.min(1, m.settings_template_key_invalid())
			.max(80)
			.regex(/^[a-z0-9_-]+$/, m.settings_template_key_invalid()),
		name: z.string().trim().min(1, m.settings_template_content_required()),
		description: z.string(),
		lease_type: z.enum(['residential', 'commercial', 'short_term', 'other']),
		locale: z.string().trim().min(1).max(35),
		content_format: z.enum(['text', 'markdown', 'html']),
		content_body: z.string().trim().min(1, m.settings_template_content_required()),
		placeholders_json: z.string().refine((value) => {
			try {
				JSON.parse(value);
				return true;
			} catch {
				return false;
			}
		}, m.settings_template_json_invalid()),
		change_note: z.string()
	});

	const form = createForm(() => ({
		defaultValues,
		validators: { onSubmit: schema },
		onSubmit: async ({ value }) => {
			const ok = await onSubmit({
				...value,
				description: value.description.trim() || null,
				change_note: value.change_note.trim() || null,
				placeholders_json: value.placeholders_json.trim() || '{}'
			});
			if (ok) onCancel();
		}
	}));

	function fieldError(errors: unknown[]) {
		return getFieldError(errors);
	}

	function submit(event: SubmitEvent) {
		event.preventDefault();
		event.stopPropagation();
		void form.handleSubmit();
	}
</script>

<form id="contract-template-form" class="flex min-h-0 flex-1 flex-col" onsubmit={submit}>
	<div class="grid min-h-0 flex-1 content-start gap-3 overflow-y-auto p-4 scrollbar-none">
		<form.Field name="template_key">
			{#snippet children(field)}
				<FormField label={m.settings_template_key()} id="template-key" error={fieldError(field.state.meta.errors)}>
					<Input
						id="template-key"
						value={field.state.value}
						oninput={(event) => field.handleChange(event.currentTarget.value)}
						onblur={field.handleBlur}
						dir="ltr"
						autocomplete="off"
						disabled={Boolean(initial)}
					/>
				</FormField>
			{/snippet}
		</form.Field>
		<form.Field name="name">
			{#snippet children(field)}
				<FormField label={m.settings_template_name()} id="template-name" error={fieldError(field.state.meta.errors)}>
					<Input
						id="template-name"
						value={field.state.value}
						oninput={(event) => field.handleChange(event.currentTarget.value)}
						onblur={field.handleBlur}
					/>
				</FormField>
			{/snippet}
		</form.Field>
		<form.Field name="description">
			{#snippet children(field)}
				<FormField label={m.settings_template_description()} id="template-description">
					<Input
						id="template-description"
						value={field.state.value}
						oninput={(event) => field.handleChange(event.currentTarget.value)}
					/>
				</FormField>
			{/snippet}
		</form.Field>
		<div class="grid gap-3 sm:grid-cols-2">
			<form.Field name="lease_type">
				{#snippet children(field)}
					<FormField label={m.settings_template_lease_type()} id="template-lease-type">
						<Select type="single" value={field.state.value} onValueChange={(value) => field.handleChange(value)}>
							<SelectTrigger id="template-lease-type" class="h-9 w-full rounded-md text-sm"><SelectValue /></SelectTrigger>
							<SelectContent>
								<SelectItem value="residential">{m.properties_type_residential()}</SelectItem>
								<SelectItem value="commercial">{m.properties_type_commercial()}</SelectItem>
								<SelectItem value="short_term">{m.settings_template_short_term()}</SelectItem>
								<SelectItem value="other">{m.common_other()}</SelectItem>
							</SelectContent>
						</Select>
					</FormField>
				{/snippet}
			</form.Field>
			<form.Field name="locale">
				{#snippet children(field)}
					<FormField label={m.settings_template_locale()} id="template-locale">
						<Input
							id="template-locale"
							value={field.state.value}
							oninput={(event) => field.handleChange(event.currentTarget.value)}
							dir="ltr"
							autocomplete="off"
						/>
					</FormField>
				{/snippet}
			</form.Field>
		</div>
		<div class="grid gap-3 sm:grid-cols-2">
			<form.Field name="content_format">
				{#snippet children(field)}
					<FormField label={m.settings_template_format()} id="template-format">
						<Select type="single" value={field.state.value} onValueChange={(value) => field.handleChange(value)}>
							<SelectTrigger id="template-format" class="h-9 w-full rounded-md text-sm"><SelectValue /></SelectTrigger>
							<SelectContent>
								<SelectItem value="markdown">Markdown</SelectItem>
								<SelectItem value="html">HTML</SelectItem>
								<SelectItem value="text">{m.settings_template_text()}</SelectItem>
							</SelectContent>
						</Select>
					</FormField>
				{/snippet}
			</form.Field>
			<form.Field name="placeholders_json">
				{#snippet children(field)}
					<FormField label="JSON" id="template-placeholders" error={fieldError(field.state.meta.errors)}>
						<Input
							id="template-placeholders"
							value={field.state.value}
							oninput={(event) => field.handleChange(event.currentTarget.value)}
							dir="ltr"
							autocomplete="off"
						/>
					</FormField>
				{/snippet}
			</form.Field>
		</div>
		<form.Field name="content_body">
			{#snippet children(field)}
				<FormField label={m.settings_template_content()} id="template-content" error={fieldError(field.state.meta.errors)}>
					<textarea
						id="template-content"
						value={field.state.value}
						oninput={(event) => field.handleChange(event.currentTarget.value)}
						onblur={field.handleBlur}
						rows="10"
						dir="auto"
						class="min-h-48 w-full rounded-md border border-input bg-card px-3 py-2 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring/40"
					></textarea>
				</FormField>
			{/snippet}
		</form.Field>
		<form.Field name="change_note">
			{#snippet children(field)}
				<FormField label={m.settings_template_change_note()} id="template-change-note">
					<Input
						id="template-change-note"
						value={field.state.value}
						oninput={(event) => field.handleChange(event.currentTarget.value)}
					/>
				</FormField>
			{/snippet}
		</form.Field>
	</div>
</form>
<SheetFooter class="shrink-0 border-t border-border bg-popover px-4 py-3">
	<Button type="button" variant="outline" disabled={busy} onclick={onCancel}>{m.common_cancel()}</Button>
	<Button type="submit" form="contract-template-form" disabled={busy}>
		{busy ? m.common_processing() : m.settings_save_changes()}
	</Button>
</SheetFooter>
