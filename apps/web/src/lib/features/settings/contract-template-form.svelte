<script lang="ts">
	import { createForm } from '@tanstack/svelte-form';
	import { onMount, tick, untrack } from 'svelte';
	import { z } from 'zod';
	import type { ContractTemplate, ContractTemplateInput } from '@aqardesk/sdk';
	import type { ContractDocumentStyle } from '#lib/api/desktop';
	import { desktopApi } from '#lib/api/desktop';
	import {
		buildPrintDocumentHtml,
		contractDocumentFont,
		contractDocumentStyleFromPlaceholders,
		contractTemplatePlaceholdersWithDesign,
		printHtmlDocument
	} from '#lib/features/contracts/document-design';
	import {
		docForgeDocumentFromPlaceholders,
		docForgeDocumentToTemplateContent,
		serializeDocForgeTemplate
	} from '#lib/features/contracts/docforge-template';
	import {
		Button,
		Input,
		Select,
		SelectContent,
		SelectItem,
		SelectTrigger,
		SelectValue,
		toast
	} from '@aqardesk/ui';
	import { Eye, Printer } from '@lucide/svelte';
	import FormField from '#lib/shared/components/form-field.svelte';
	import { getFieldError } from '#lib/shared/form-errors';
	import DocForgeTemplateEditor from '#lib/features/settings/docforge-template-editor.svelte';
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
	let design = $state<ContractDocumentStyle>(
		untrack(() =>
			contractDocumentStyleFromPlaceholders(
				initial?.versions[0]?.placeholders_json,
				initial?.lease_type === 'receipt' ? 'a5' : 'a4'
			)
		)
	);
	let docForgeDocument = $state(
		untrack(() =>
			docForgeDocumentFromPlaceholders(
				initial?.versions[0]?.placeholders_json,
				initial?.versions[0]?.content_body ?? '',
				defaultValues.locale,
				initial?.lease_type === 'receipt' ? 'a5' : 'a4'
			)
		)
	);
	let organizationName = $state('');
	let organizationLogo = $state<string | null>(null);
	let printPreviewHtml = $state<string | null>(null);
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

	onMount(() => {
		void desktopApi.getOrganizationSettings().then(
			(settings) => {
				organizationName = settings.name;
				organizationLogo = settings.logo_data_url ?? null;
			},
			(cause) => toast.error(cause instanceof Error ? cause.message : String(cause))
		);
	});

	function isDocumentFont(value: string): value is ContractDocumentStyle['font_family'] {
		return ['Tahoma', 'Arial', 'Georgia', 'Noto Naskh Arabic'].includes(value);
	}

	function isDocumentFontSize(value: number): value is ContractDocumentStyle['font_size'] {
		return value === 12 || value === 14 || value === 16 || value === 18;
	}

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
		] },
		{ group: 'payment', fields: [
			{ key: 'payment.receipt_number', label: m.contracts_variable_payment_receipt_number() },
			{ key: 'payment.date', label: m.contracts_variable_payment_date() },
			{ key: 'payment.status', label: m.contracts_variable_payment_status() },
			{ key: 'payment.tenant', label: m.contracts_variable_payment_tenant() },
			{ key: 'payment.amount', label: m.contracts_variable_payment_amount() },
			{ key: 'payment.allocations', label: m.contracts_variable_payment_allocations() },
			{ key: 'payment.currency', label: m.contracts_variable_payment_currency() },
			{ key: 'payment.method', label: m.contracts_variable_payment_method() },
			{ key: 'payment.reference', label: m.contracts_variable_payment_reference() },
			{ key: 'payment.notes', label: m.contracts_variable_payment_notes() }
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
					return `<h${level} class="${size} pt-3 font-semibold leading-relaxed" style="color:${design.accent_color}">${renderInlineMarkdown(heading[2])}</h${level}>`;
				}
				const listItem = /^(?:[-*+]\s+|\d+[.)]\s+)(.+)$/.exec(line);
				if (listItem) {
					return `<p><span style="color:${design.accent_color}">• </span>${renderInlineMarkdown(listItem[1])}</p>`;
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

	function openPrintPreview() {
		const hasHeader =
			organizationLogo ||
			(design.show_branding && design.brand_label.trim()) ||
			design.show_contract_number;
		const headerHtml = hasHeader
			? `<header><div class="brand-row">${organizationLogo ? `<img class="brand-logo" src="${escapeHtml(organizationLogo)}" alt="">` : ''}${design.show_branding && design.brand_label.trim() ? `<p class="brand">${escapeHtml(design.brand_label)}</p>` : ''}</div>${design.show_contract_number ? '<p class="contract-number">AQD-20261112-EXAMPLE</p>' : ''}</header>`
			: '';
		const footerHtml = design.show_footer
			? `<footer>${escapeHtml(design.footer_text.trim() || m.settings_template_preview_disclaimer())}</footer>`
			: '';
		printPreviewHtml = buildPrintDocumentHtml({
			title: name || m.settings_template_new(),
			direction: previewDirection,
			design,
			headerHtml,
			bodyHtml: renderMarkdown(preview.content_body),
			footerHtml
		});
	}

	async function printTemplate() {
		if (!printPreviewHtml) return;
		try {
			await printHtmlDocument(printPreviewHtml);
		} catch (cause) {
			toast.error(cause instanceof Error ? cause.message : String(cause));
		}
	}

	const schema = z.object({
		description: z.string(),
		lease_type: z.enum(['residential', 'commercial', 'short_term', 'other', 'receipt']),
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
			await onSubmit({
				...value,
				template_key: '',
				name: name.trim(),
				content_format: initial?.versions[0]?.content_format ?? '',
				placeholders_json: contractTemplatePlaceholdersWithDesign(
					initial?.versions[0]?.placeholders_json,
					design,
					serializeDocForgeTemplate(docForgeDocument)
				),
				description: value.description.trim() || null,
				change_note: value.change_note.trim() || null
			});
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
	{#if printPreviewHtml}
		<section class="flex min-h-0 flex-1 flex-col overflow-hidden">
			<div class="flex shrink-0 flex-wrap items-center justify-between gap-2 border-b border-border bg-card px-4 py-3">
				<Button type="button" variant="outline" onclick={() => (printPreviewHtml = null)}>
					{m.settings_template_back_to_editor()}
				</Button>
				<Button type="button" onclick={printTemplate}>
					<Printer size={15} class="me-2" />
					{m.settings_template_print()}
				</Button>
			</div>
			<iframe
				title={m.settings_template_preview()}
				srcdoc={printPreviewHtml}
				class="min-h-0 flex-1 border-0 bg-slate-200"
			></iframe>
		</section>
	{:else}
	<div class="grid min-h-0 flex-1 content-start gap-3 overflow-y-auto p-4 scrollbar-none">
		<div class="grid content-start gap-4">
		<form.Field name="content_body">
			{#snippet children(field)}
				<div class="grid gap-3">
					<DocForgeTemplateEditor
						document={docForgeDocument}
						brandLabel={design.brand_label}
						footerText={design.footer_text}
						{organizationName}
						{organizationLogo}
						variables={allTemplateVariables}
						onDocumentChange={(next) => {
							docForgeDocument = next;
							const content = docForgeDocumentToTemplateContent(next);
							updatePreview('content_body', content);
							field.handleChange(content);
						}}
						onBrandLabelChange={(value) => (design = { ...design, brand_label: value })}
						onFooterTextChange={(value) => (design = { ...design, footer_text: value })}
						onPaperSizeChange={(paper_size) => (design = { ...design, paper_size })}
					/>
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
	<section class="grid gap-3 rounded-lg border border-border/70 p-4">
		<h3 class="text-sm font-semibold">{m.contract_design_settings()}</h3>
		<div class="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
			<FormField label={m.contract_design_paper()} id="contract-design-paper">
				<Select
					type="single"
					value={design.paper_size}
					items={[
						{ value: 'a4', label: m.contract_design_paper_a4() },
						{ value: 'a5', label: m.contract_design_paper_a5() },
						{ value: 'letter', label: m.contract_design_paper_letter() }
					]}
					onValueChange={(value) => {
						if (value === 'a4' || value === 'a5' || value === 'letter') {
							design = { ...design, paper_size: value };
						}
					}}
				>
					<SelectTrigger id="contract-design-paper" class="h-9 w-full rounded-md text-sm"><SelectValue /></SelectTrigger>
					<SelectContent>
						<SelectItem value="a4" label={m.contract_design_paper_a4()}>{m.contract_design_paper_a4()}</SelectItem>
						<SelectItem value="a5" label={m.contract_design_paper_a5()}>{m.contract_design_paper_a5()}</SelectItem>
						<SelectItem value="letter" label={m.contract_design_paper_letter()}>{m.contract_design_paper_letter()}</SelectItem>
					</SelectContent>
				</Select>
			</FormField>
			<FormField label={m.contract_design_accent_color()} id="contract-design-color">
				<Input
					id="contract-design-color"
					type="color"
					value={design.accent_color}
					class="h-9 w-full cursor-pointer p-1"
					oninput={(event) => (design = { ...design, accent_color: event.currentTarget.value })}
				/>
			</FormField>
			<FormField label={m.contract_design_font()} id="contract-design-font">
				<Select
					type="single"
					value={design.font_family}
					items={[
						{ value: 'Tahoma', label: m.contract_design_font_tahoma() },
						{ value: 'Arial', label: m.contract_design_font_arial() },
						{ value: 'Georgia', label: m.contract_design_font_georgia() },
						{ value: 'Noto Naskh Arabic', label: m.contract_design_font_naskh() }
					]}
					onValueChange={(value) => {
						if (isDocumentFont(value)) {
							design = { ...design, font_family: value };
						}
					}}
				>
					<SelectTrigger id="contract-design-font" class="h-9 w-full rounded-md text-sm"><SelectValue /></SelectTrigger>
					<SelectContent>
						<SelectItem value="Tahoma" label={m.contract_design_font_tahoma()}>{m.contract_design_font_tahoma()}</SelectItem>
						<SelectItem value="Arial" label={m.contract_design_font_arial()}>{m.contract_design_font_arial()}</SelectItem>
						<SelectItem value="Georgia" label={m.contract_design_font_georgia()}>{m.contract_design_font_georgia()}</SelectItem>
						<SelectItem value="Noto Naskh Arabic" label={m.contract_design_font_naskh()}>{m.contract_design_font_naskh()}</SelectItem>
					</SelectContent>
				</Select>
			</FormField>
			<FormField label={m.contract_design_font_size()} id="contract-design-font-size">
				<Select
					type="single"
					value={String(design.font_size)}
					items={[12, 14, 16, 18].map((value) => ({ value: String(value), label: `${value}px` }))}
					onValueChange={(value) => {
						const size = Number(value);
						if (isDocumentFontSize(size)) {
							design = { ...design, font_size: size };
						}
					}}
				>
					<SelectTrigger id="contract-design-font-size" class="h-9 w-full rounded-md text-sm"><SelectValue /></SelectTrigger>
					<SelectContent>
						{#each [12, 14, 16, 18] as value}
							<SelectItem value={String(value)} label={`${value}px`}>{value}px</SelectItem>
						{/each}
					</SelectContent>
				</Select>
			</FormField>
			<FormField label={m.contract_design_alignment()} id="contract-design-alignment">
				<Select
					type="single"
					value={design.alignment}
					items={[
						{ value: 'start', label: m.contract_design_alignment_start() },
						{ value: 'justify', label: m.contract_design_alignment_justify() }
					]}
					onValueChange={(value) => {
						if (value === 'start' || value === 'justify') design = { ...design, alignment: value };
					}}
				>
					<SelectTrigger id="contract-design-alignment" class="h-9 w-full rounded-md text-sm"><SelectValue /></SelectTrigger>
					<SelectContent>
						<SelectItem value="start" label={m.contract_design_alignment_start()}>{m.contract_design_alignment_start()}</SelectItem>
						<SelectItem value="justify" label={m.contract_design_alignment_justify()}>{m.contract_design_alignment_justify()}</SelectItem>
					</SelectContent>
				</Select>
			</FormField>
			<FormField label={m.contract_design_page_margin()} id="contract-design-margin">
				<Input
					id="contract-design-margin"
					type="number"
					min="12"
					max="30"
					step="1"
					value={design.page_margin_mm}
					oninput={(event) => {
						const margin = Number(event.currentTarget.value);
						if (Number.isInteger(margin) && margin >= 12 && margin <= 30) {
							design = { ...design, page_margin_mm: margin };
						}
					}}
				/>
			</FormField>
			<FormField label={m.contract_design_brand_label()} id="contract-design-brand-label">
				<Input
					id="contract-design-brand-label"
					value={design.brand_label}
					maxlength={100}
					autocomplete="organization"
					oninput={(event) => (design = { ...design, brand_label: event.currentTarget.value })}
				/>
			</FormField>
			<FormField label={m.contract_design_footer_text()} id="contract-design-footer-text">
				<Input
					id="contract-design-footer-text"
					value={design.footer_text}
					maxlength={400}
					oninput={(event) => (design = { ...design, footer_text: event.currentTarget.value })}
				/>
			</FormField>
		</div>
		<div class="flex flex-wrap gap-5">
			<label class="flex items-center gap-2 text-sm">
				<input type="checkbox" checked={design.show_branding} onchange={(event) => (design = { ...design, show_branding: event.currentTarget.checked })} />
				{m.contract_design_show_branding()}
			</label>
			<label class="flex items-center gap-2 text-sm">
				<input type="checkbox" checked={design.show_contract_number} onchange={(event) => (design = { ...design, show_contract_number: event.currentTarget.checked })} />
				{m.contract_design_show_contract_number()}
			</label>
			<label class="flex items-center gap-2 text-sm">
				<input type="checkbox" checked={design.show_footer} onchange={(event) => (design = { ...design, show_footer: event.currentTarget.checked })} />
				{m.contract_design_show_footer()}
			</label>
		</div>
	</section>
	</div>
	<div class="sticky bottom-0 z-20 !mt-0 !flex !shrink-0 !flex-col !justify-normal gap-3 border-t border-border bg-popover px-4 py-3 shadow-[0_-8px_20px_-16px_rgba(0,0,0,0.5)]">
		<div class="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
			<form.Field name="lease_type">
				{#snippet children(field)}
					<FormField label={m.settings_template_lease_type()} id="template-lease-type">
						<Select
							type="single"
							value={field.state.value}
							items={[
								{ value: 'residential', label: m.properties_type_residential() },
								{ value: 'commercial', label: m.properties_type_commercial() },
								{ value: 'short_term', label: m.settings_template_short_term() },
								{ value: 'other', label: m.common_other() },
								{ value: 'receipt', label: m.settings_template_receipt() }
							]}
							onValueChange={(value) => {
								updatePreview('lease_type', value);
								field.handleChange(value);
								if (value === 'receipt') design = { ...design, paper_size: 'a5' };
							}}
						>
							<SelectTrigger id="template-lease-type" class="h-9 w-full rounded-md text-sm"><SelectValue /></SelectTrigger>
							<SelectContent>
								<SelectItem value="residential" label={m.properties_type_residential()}>{m.properties_type_residential()}</SelectItem>
								<SelectItem value="commercial" label={m.properties_type_commercial()}>{m.properties_type_commercial()}</SelectItem>
								<SelectItem value="short_term" label={m.settings_template_short_term()}>{m.settings_template_short_term()}</SelectItem>
								<SelectItem value="other" label={m.common_other()}>{m.common_other()}</SelectItem>
								<SelectItem value="receipt" label={m.settings_template_receipt()}>{m.settings_template_receipt()}</SelectItem>
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
			<Button type="button" variant="outline" disabled={busy} onclick={openPrintPreview}>
				<Eye size={15} class="me-2" />
				{m.settings_template_preview()}
			</Button>
			<Button type="button" variant="outline" disabled={busy} onclick={onCancel}>{m.common_cancel()}</Button>
			<Button type="submit" disabled={busy}>
				{busy ? m.common_processing() : m.settings_save_changes()}
			</Button>
		</div>
	</div>
	{/if}
</form>
