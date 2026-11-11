<script lang="ts">
	import { createForm } from '@tanstack/svelte-form';
	import { untrack } from 'svelte';
	import { z } from 'zod';
	import type { ContractTemplate, ContractTemplateInput } from '@aqardesk/sdk';
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
	import { Printer } from '@lucide/svelte';
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
	let preview = $state<Values>({ ...defaultValues });
	const previewDirection = $derived(preview.locale.toLowerCase().startsWith('ar') ? 'rtl' : 'ltr');

	type PreviewBlock =
		| { kind: 'heading'; level: number; text: string }
		| { kind: 'paragraph' | 'list-item'; text: string };

	function updatePreview<K extends keyof Values>(key: K, value: Values[K]) {
		preview[key] = value;
	}

	function parsePreviewBody(body: string): PreviewBlock[] {
		return body
			.split(/\r?\n/)
			.map((line) => line.trim())
			.filter(Boolean)
			.map((line) => {
				const heading = /^(#{1,3})\s+(.+)$/.exec(line);
				if (heading) {
					return {
						kind: 'heading',
						level: heading[1].length,
						text: heading[2]
					};
				}
				const listItem = /^(?:[-*+]\s+|\d+[.)]\s+)(.+)$/.exec(line);
				if (listItem) return { kind: 'list-item', text: listItem[1] };
				return { kind: 'paragraph', text: line };
			});
	}

	function printTemplate() {
		const printStyles = document.createElement('style');
		printStyles.textContent = `
			@media print {
				@page { size: A4; margin: 18mm; }
				body * { visibility: hidden !important; overflow: visible !important; }
				#contract-template-preview,
				#contract-template-preview * { visibility: visible !important; }
				#contract-template-preview {
					position: absolute !important;
					inset: 0 !important;
					width: 100% !important;
					max-width: none !important;
					min-height: 0 !important;
					margin: 0 !important;
					padding: 0 !important;
					border: 0 !important;
					border-radius: 0 !important;
					box-shadow: none !important;
					background: #fff !important;
					color: #172033 !important;
				}
			}`;
		document.head.append(printStyles);
		window.addEventListener('afterprint', () => printStyles.remove(), { once: true });
		window.print();
	}

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
		content_body: z.string(),
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
		<div class="grid min-h-0 gap-6 xl:grid-cols-[minmax(0,1fr)_minmax(20rem,0.9fr)]">
			<div class="grid content-start gap-3">
		<form.Field name="template_key">
			{#snippet children(field)}
				<FormField label={m.settings_template_key()} id="template-key" error={fieldError(field.state.meta.errors)}>
					<Input
						id="template-key"
						value={field.state.value}
						oninput={(event) => {
							const value = event.currentTarget.value;
							updatePreview('template_key', value);
							field.handleChange(value);
						}}
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
						oninput={(event) => {
							const value = event.currentTarget.value;
							updatePreview('name', value);
							field.handleChange(value);
						}}
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
						oninput={(event) => {
							const value = event.currentTarget.value;
							updatePreview('description', value);
							field.handleChange(value);
						}}
					/>
				</FormField>
			{/snippet}
		</form.Field>
		<div class="grid gap-3 sm:grid-cols-2">
			<form.Field name="lease_type">
				{#snippet children(field)}
					<FormField label={m.settings_template_lease_type()} id="template-lease-type">
						<Select type="single" value={field.state.value} onValueChange={(value) => {
							updatePreview('lease_type', value);
							field.handleChange(value);
						}}>
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
							oninput={(event) => {
								const value = event.currentTarget.value;
								updatePreview('locale', value);
								field.handleChange(value);
							}}
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
						<Select type="single" value={field.state.value} onValueChange={(value) => {
							updatePreview('content_format', value);
							field.handleChange(value);
						}}>
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
							oninput={(event) => {
								const value = event.currentTarget.value;
								updatePreview('placeholders_json', value);
								field.handleChange(value);
							}}
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
						oninput={(event) => {
							const value = event.currentTarget.value;
							updatePreview('content_body', value);
							field.handleChange(value);
						}}
						onblur={field.handleBlur}
						rows="10"
						dir="auto"
						class="min-h-48 w-full rounded-md border border-input bg-card px-3 py-2 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring/40"
					></textarea>
				</FormField>
			{/snippet}
		</form.Field>
			</div>

			<aside class="min-h-0 rounded-xl border border-border/70 bg-muted/30 p-4 sm:p-5" aria-label={m.settings_template_preview()}>
				<div class="mb-4 flex items-center justify-between gap-3">
					<div class="grid gap-1">
						<h2 class="text-sm font-semibold">{m.settings_template_preview()}</h2>
						<p class="text-xs text-muted-foreground">{m.settings_template_preview_disclaimer()}</p>
					</div>
					<Button type="button" variant="outline" size="sm" onclick={printTemplate}>
						<Printer size={15} class="me-2" />
						{m.settings_template_print()}
					</Button>
				</div>
				<article
					id="contract-template-preview"
					class="mx-auto min-h-[28rem] max-w-[38rem] rounded-lg border border-border/70 bg-background px-6 py-7 shadow-sm sm:px-9 sm:py-10"
					dir={previewDirection}
				>
					<header class="mb-6 border-b-2 border-primary/60 pb-4">
						<p class="text-[10px] font-bold uppercase tracking-[0.16em] text-primary">AqarDesk</p>
						<h3 class="mt-3 text-xl font-semibold leading-relaxed">
							{preview.name.trim() || m.settings_template_preview()}
						</h3>
						<p class="mt-1 text-xs text-muted-foreground" dir="ltr">
							{preview.template_key || 'template-key'} · {preview.locale.toUpperCase() || 'AR'}
						</p>
					</header>
					<div class="grid gap-3 text-sm leading-8">
						{#if preview.content_body.trim()}
							{#if preview.content_format !== 'markdown'}
								<pre class="whitespace-pre-wrap break-words font-mono text-xs leading-6">{preview.content_body}</pre>
							{:else}
								{#each parsePreviewBody(preview.content_body) as block, index (`${block.kind}-${index}`)}
									{#if block.kind === 'heading'}
										<h4 class="pt-2 font-semibold">
											{#each block.text.split(/({{[a-zA-Z0-9_.-]+}})/g) as part, partIndex (`${part}-${partIndex}`)}
												{#if part.startsWith('{{') && part.endsWith('}}')}
													<code class="rounded bg-primary/10 px-1 text-primary">{part}</code>
												{:else}
													{part}
												{/if}
											{/each}
										</h4>
									{:else if block.kind === 'list-item'}
										<p class="before:me-2 before:text-primary before:content-['•']">
											{#each block.text.split(/({{[a-zA-Z0-9_.-]+}})/g) as part, partIndex (`${part}-${partIndex}`)}
												{#if part.startsWith('{{') && part.endsWith('}}')}
													<code class="rounded bg-primary/10 px-1 text-primary">{part}</code>
												{:else}
													{part}
												{/if}
											{/each}
										</p>
									{:else}
										<p class="whitespace-pre-wrap">
											{#each block.text.split(/({{[a-zA-Z0-9_.-]+}})/g) as part, partIndex (`${part}-${partIndex}`)}
												{#if part.startsWith('{{') && part.endsWith('}}')}
													<code class="rounded bg-primary/10 px-1 text-primary">{part}</code>
												{:else}
													{part}
												{/if}
											{/each}
										</p>
									{/if}
								{/each}
							{/if}
						{:else}
							<p class="rounded-md border border-dashed border-border px-4 py-6 text-center text-sm text-muted-foreground">
								{m.settings_template_preview_empty()}
							</p>
						{/if}
					</div>
					<footer class="mt-10 border-t border-border pt-3 text-[10px] leading-5 text-muted-foreground">
						{m.settings_template_preview_disclaimer()}
					</footer>
				</article>
			</aside>
		</div>
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
