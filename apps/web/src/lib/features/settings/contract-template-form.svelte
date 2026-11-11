<script lang="ts">
	import { createForm } from '@tanstack/svelte-form';
	import { tick, untrack } from 'svelte';
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
		description: string;
		lease_type: string;
		locale: string;
		content_body: string;
		change_note: string;
	};

	type Props = {
		initial?: ContractTemplate;
		name?: string;
		busy: boolean;
		onSubmit: (input: ContractTemplateInput) => Promise<boolean>;
		onCancel: () => void;
	};

	let { initial, name = $bindable(''), busy, onSubmit, onCancel }: Props = $props();
	const defaultValues: Values = untrack(() => ({
		description: initial?.description ?? '',
		lease_type: initial?.lease_type ?? 'residential',
		locale: initial?.locale ?? 'ar',
		content_body: initial?.versions[0]?.content_body ?? '',
		change_note: ''
	}));
	let preview = $state<Values>({ ...defaultValues });
	let contentEditor = $state<HTMLTextAreaElement>();
	let contentArea = $state<HTMLDivElement>();
	let documentMode = $state<'edit' | 'preview'>(
		untrack(() => (initial?.versions[0]?.content_body?.trim() ? 'preview' : 'edit'))
	);
	let titleError = $state('');
	let variableMenuOpen = $state(false);
	let variableQuery = $state('');
	let variableTriggerStart = 0;
	let activeVariableIndex = $state(0);
	let variableMenuPosition = $state({ top: 0, left: 0 });
	const previewDirection = $derived(preview.locale.toLowerCase().startsWith('ar') ? 'rtl' : 'ltr');
	const templateVariables = [
		{ group: 'tenant', fields: [
			{ key: 'tenant.full_name', label: m.contracts_variable_tenant_name() },
			{ key: 'tenant.email', label: m.contracts_variable_tenant_email() },
			{ key: 'tenant.phone', label: m.contracts_variable_tenant_phone() },
			{ key: 'tenant.address', label: m.contracts_variable_tenant_address() }
		] },
		{ group: 'property', fields: [
			{ key: 'property.name', label: m.contracts_variable_property_name() },
			{ key: 'property.code', label: m.contracts_variable_property_code() },
			{ key: 'property.address', label: m.contracts_variable_property_address() },
			{ key: 'property.city', label: m.contracts_variable_property_city() }
		] },
		{ group: 'unit', fields: [
			{ key: 'unit.number', label: m.contracts_variable_unit_number() },
			{ key: 'unit.type', label: m.contracts_variable_unit_type() },
			{ key: 'unit.floor', label: m.contracts_variable_unit_floor() },
			{ key: 'unit.area', label: m.contracts_variable_unit_area() },
			{ key: 'unit.bedrooms', label: m.contracts_variable_unit_bedrooms() },
			{ key: 'unit.bathrooms', label: m.contracts_variable_unit_bathrooms() },
			{ key: 'unit.rent', label: m.contracts_variable_unit_rent() },
			{ key: 'unit.currency', label: m.contracts_variable_unit_currency() }
		] },
		{ group: 'contract', fields: [
			{ key: 'contract.number', label: m.contracts_variable_contract_number() },
			{ key: 'contract.date', label: m.contracts_variable_contract_date() },
			{ key: 'contract.start_date', label: m.contracts_variable_contract_start() },
			{ key: 'contract.end_date', label: m.contracts_variable_contract_end() },
			{ key: 'contract.deposit', label: m.contracts_variable_contract_deposit() }
		] },
		{ group: 'organization', fields: [
			{ key: 'organization.name', label: m.contracts_variable_organization_name() }
		] }
	];
	const allTemplateVariables = $derived(templateVariables.flatMap((group) => group.fields));
	const filteredTemplateVariables = $derived(
		(() => {
			const query = variableQuery.normalize('NFKC').toLocaleLowerCase().trim();
			return allTemplateVariables.filter((item) =>
				item.label.normalize('NFKC').toLocaleLowerCase().startsWith(query) ||
				item.key.normalize('NFKC').toLocaleLowerCase().startsWith(query)
			);
		})()
	);

	function updatePreview<K extends keyof Values>(key: K, value: Values[K]) {
		preview[key] = value;
	}

	function escapeHtml(value: string) {
		return value
			.replaceAll('&', '&amp;')
			.replaceAll('<', '&lt;')
			.replaceAll('>', '&gt;')
			.replaceAll('"', '&quot;')
			.replaceAll("'", '&#39;');
	}

	function renderInlineMarkdown(value: string) {
		return escapeHtml(value)
			.replace(/\{\{([a-zA-Z0-9_.-]+)\}\}/g, '<code class="rounded bg-primary/10 px-1 py-0.5 font-mono text-primary" dir="ltr">{{$1}}</code>')
			.replace(/`([^`]+)`/g, '<code class="rounded bg-muted px-1 py-0.5 font-mono">$1</code>')
			.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
			.replace(/\*(.+?)\*/g, '<em>$1</em>');
	}

	function renderMarkdown(content: string) {
		return content
			.split(/\r?\n/)
			.map((rawLine) => {
				const line = rawLine.trim();
				if (!line) return '';
				const heading = /^(#{1,3})\s+(.+)$/.exec(line);
				if (heading) {
					const level = heading[1].length;
					const size = level === 1 ? 'text-2xl' : level === 2 ? 'text-lg' : 'text-base';
					return `<h${level} class="${size} pt-3 font-semibold leading-relaxed">${renderInlineMarkdown(heading[2])}</h${level}>`;
				}
				const listItem = /^(?:[-*+]\s+|\d+[.)]\s+)(.+)$/.exec(line);
				if (listItem) {
					return `<p class="ps-5 before:-ms-4 before:me-2 before:text-primary before:content-['•']">${renderInlineMarkdown(listItem[1])}</p>`;
				}
				return `<p class="whitespace-pre-wrap">${renderInlineMarkdown(line)}</p>`;
			})
			.filter(Boolean)
			.join('');
	}

	$effect(() => {
		if (name.trim()) titleError = '';
	});

	function updateVariableMenuPosition(editor: HTMLTextAreaElement, caret: number) {
		const wrapper = contentArea;
		if (!wrapper) return;
		const rect = editor.getBoundingClientRect();
		const computed = getComputedStyle(editor);
		const mirror = document.createElement('div');
		const marker = document.createElement('span');
		mirror.style.position = 'fixed';
		mirror.style.visibility = 'hidden';
		mirror.style.whiteSpace = 'pre-wrap';
		mirror.style.overflowWrap = 'break-word';
		mirror.style.top = `${rect.top}px`;
		mirror.style.left = `${rect.left}px`;
		mirror.style.width = `${rect.width}px`;
		for (const property of [
			'boxSizing', 'fontFamily', 'fontSize', 'fontWeight', 'fontStyle', 'letterSpacing',
			'lineHeight', 'padding', 'border', 'textIndent', 'wordSpacing', 'tabSize'
		] as const) {
			mirror.style[property] = computed[property];
		}
		mirror.textContent = editor.value.slice(0, caret);
		marker.textContent = '\u200b';
		mirror.append(marker);
		document.body.append(mirror);
		const markerRect = marker.getBoundingClientRect();
		const wrapperRect = wrapper.getBoundingClientRect();
		const menuWidth = Math.min(360, wrapper.clientWidth - 16);
		const menuHeight = Math.min(280, Math.max(130, filteredTemplateVariables.length * 38));
		const left = Math.max(8, Math.min(
			markerRect.left - wrapperRect.left - editor.scrollLeft,
			wrapper.clientWidth - menuWidth - 8
		));
		const below = markerRect.bottom - wrapperRect.top - editor.scrollTop + 6;
		const top = below + menuHeight <= wrapper.clientHeight
			? below
			: Math.max(8, markerRect.top - wrapperRect.top - editor.scrollTop - menuHeight - 6);
		variableMenuPosition = { top, left };
		mirror.remove();
	}

	function syncVariableMenu(editor: HTMLTextAreaElement) {
		const caret = editor.selectionStart;
		const preceding = editor.value.slice(0, caret);
		const slash = preceding.lastIndexOf('/');
		const at = preceding.lastIndexOf('@');
		const trigger = Math.max(slash, at);
		const charBeforeTrigger = trigger > 0 ? preceding[trigger - 1] : '';
		const query = trigger >= 0 ? preceding.slice(trigger + 1) : '';
		if (
			trigger < 0 ||
			(charBeforeTrigger && !/\s|[(\[{,:;]/.test(charBeforeTrigger)) ||
			/[\s{}]/.test(query)
		) {
			variableMenuOpen = false;
			return;
		}
		variableTriggerStart = trigger;
		variableQuery = query;
		activeVariableIndex = 0;
		variableMenuOpen = filteredTemplateVariables.length > 0;
		if (variableMenuOpen) updateVariableMenuPosition(editor, caret);
	}

	async function insertVariable(variable: string, handleChange: (value: string) => void) {
		const editor = contentEditor;
		if (!editor || !variable) return;
		const token = `{{${variable}}}`;
		const start = variableMenuOpen ? variableTriggerStart : editor.selectionStart;
		const end = editor.selectionEnd;
		const caret = editor.selectionStart;
		const value = `${editor.value.slice(0, start)}${token}${editor.value.slice(caret, end)}${editor.value.slice(end)}`;
		handleChange(value);
		updatePreview('content_body', value);
		variableMenuOpen = false;
		await tick();
		editor.focus();
		const nextCaret = start + token.length;
		editor.setSelectionRange(nextCaret, nextCaret);
	}

	function handleContentKeydown(event: KeyboardEvent, handleChange: (value: string) => void) {
		if (!variableMenuOpen) return;
		if (event.key === 'ArrowDown') {
			event.preventDefault();
			activeVariableIndex = (activeVariableIndex + 1) % filteredTemplateVariables.length;
		} else if (event.key === 'ArrowUp') {
			event.preventDefault();
			activeVariableIndex = (activeVariableIndex - 1 + filteredTemplateVariables.length) % filteredTemplateVariables.length;
		} else if (event.key === 'Enter' || event.key === 'Tab') {
			event.preventDefault();
			const selected = filteredTemplateVariables[activeVariableIndex];
			if (selected) void insertVariable(selected.key, handleChange);
		} else if (event.key === 'Escape') {
			event.preventDefault();
			variableMenuOpen = false;
		}
	}

	async function printTemplate() {
		const previousMode = documentMode;
		documentMode = 'preview';
		await tick();
		const printStyles = document.createElement('style');
		printStyles.textContent = `
			@media print {
				@page { size: A4; margin: 18mm; }
				body * { visibility: hidden !important; overflow: visible !important; }
				#contract-template-preview,
				#contract-template-preview * { visibility: visible !important; }
				#contract-template-preview .contract-template-toolbar { display: none !important; }
				#contract-template-preview .template-document-preview { display: block !important; }
				#contract-template-preview .template-edit-surface { display: none !important; }
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
				#template-content {
					min-height: 0 !important;
					height: auto !important;
					resize: none !important;
					color: #172033 !important;
				}
			}`;
		document.head.append(printStyles);
		window.addEventListener('afterprint', () => {
			printStyles.remove();
			documentMode = previousMode;
		}, { once: true });
		window.print();
	}

	const schema = z.object({
		description: z.string(),
		lease_type: z.enum(['residential', 'commercial', 'short_term', 'other']),
		locale: z.string().trim().min(1).max(35),
		content_body: z.string(),
		change_note: z.string()
	});

	const form = createForm(() => ({
		defaultValues,
		validators: { onSubmit: schema },
		onSubmit: async ({ value }) => {
			if (!name.trim()) {
				titleError = m.settings_template_content_required();
				return;
			}
			const ok = await onSubmit({
				...value,
				template_key: '',
				name: name.trim(),
				content_format: initial?.versions[0]?.content_format ?? '',
				placeholders_json: initial?.versions[0]?.placeholders_json ?? null,
				description: value.description.trim() || null,
				change_note: value.change_note.trim() || null
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
		<div class="grid content-start gap-4">
		<form.Field name="content_body">
			{#snippet children(field)}
				<div class="grid gap-3">
					<article
						id="contract-template-preview"
						class="w-full rounded-lg border border-border/70 bg-background shadow-sm"
						dir={previewDirection}
					>
						<div class="contract-template-toolbar flex items-center justify-end gap-2 border-b border-border/70 px-4 py-2">
							<Button
								type="button"
								size="sm"
								variant={documentMode === 'edit' ? 'secondary' : 'ghost'}
								onclick={() => (documentMode = 'edit')}
							>
								{m.settings_template_edit_document()}
							</Button>
							<Button
								type="button"
								size="sm"
								variant={documentMode === 'preview' ? 'secondary' : 'ghost'}
								onclick={() => {
									variableMenuOpen = false;
									documentMode = 'preview';
								}}
							>
								{m.settings_template_render_preview()}
							</Button>
						</div>
						<div bind:this={contentArea} class="relative">
							{#if documentMode === 'edit'}
								<div class="template-edit-surface">
									<textarea
										bind:this={contentEditor}
										id="template-content"
										aria-label={m.settings_template_content()}
										aria-autocomplete="list"
										value={field.state.value}
										oninput={(event) => {
											const value = event.currentTarget.value;
											updatePreview('content_body', value);
											field.handleChange(value);
											syncVariableMenu(event.currentTarget);
										}}
										onkeydown={(event) => handleContentKeydown(event, field.handleChange)}
										onclick={(event) => {
											if (variableMenuOpen) syncVariableMenu(event.currentTarget);
										}}
										onblur={field.handleBlur}
										placeholder={m.settings_template_preview_empty()}
										dir="auto"
										class="min-h-[calc(100dvh-18rem)] w-full resize-y border-0 bg-transparent px-6 py-7 text-sm leading-8 text-foreground outline-none focus-visible:ring-0 sm:px-10 sm:py-9"
									></textarea>
									{#if variableMenuOpen}
										<div
											role="listbox"
											aria-label={m.settings_template_insert_field()}
											class="absolute z-30 max-h-64 overflow-y-auto scrollbar-none rounded-lg border border-border bg-popover p-1 shadow-lg"
											style={`top:${variableMenuPosition.top}px;left:${variableMenuPosition.left}px;width:min(22rem,calc(100% - 1rem))`}
										>
											{#each filteredTemplateVariables as item, index (item.key)}
												<button
													type="button"
													role="option"
													aria-selected={index === activeVariableIndex}
													onpointerdown={(event) => event.preventDefault()}
													onclick={() => void insertVariable(item.key, field.handleChange)}
													class={`flex w-full items-center justify-between gap-3 rounded-md px-3 py-2 text-start text-sm ${index === activeVariableIndex ? 'bg-accent text-accent-foreground' : 'hover:bg-accent/60'}`}
												>
													<span>{item.label}</span>
													<code class="shrink-0 text-xs text-muted-foreground" dir="ltr">{`{{${item.key}}}`}</code>
												</button>
											{/each}
										</div>
									{/if}
								</div>
							{:else}
								<div
									class="template-document-preview min-h-[calc(100dvh-18rem)] space-y-3 px-8 py-8 text-sm leading-8 sm:px-16 sm:py-12"
									dir={previewDirection}
								>
									{@html renderMarkdown(preview.content_body)}
								</div>
							{/if}
						</div>
					</article>
					{#if field.state.meta.errors.length}
						<p class="text-sm text-destructive">{fieldError(field.state.meta.errors)}</p>
					{/if}
					{#if titleError}
						<p class="text-sm text-destructive" role="alert">{titleError}</p>
					{/if}
				</div>
			{/snippet}
		</form.Field>
		</div>
	</div>
	<SheetFooter class="sticky bottom-0 z-20 !mt-0 !flex !shrink-0 !flex-col !justify-normal gap-3 border-t border-border bg-popover px-4 py-3 shadow-[0_-8px_20px_-16px_rgba(0,0,0,0.5)]">
		<div class="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
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
		<div class="flex flex-wrap justify-end gap-2">
			<Button type="button" variant="outline" disabled={busy} onclick={printTemplate}>
				<Printer size={15} class="me-2" />
				{m.settings_template_print()}
			</Button>
			<Button type="button" variant="outline" disabled={busy} onclick={onCancel}>{m.common_cancel()}</Button>
			<Button type="submit" disabled={busy}>
				{busy ? m.common_processing() : m.settings_save_changes()}
			</Button>
		</div>
	</SheetFooter>
</form>
