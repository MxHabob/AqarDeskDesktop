<script lang="ts">
	import {
		addNode,
		createNode,
		getPageDimensions,
		getPages,
		removeNode,
		updateNodeProps,
		type TextNode,
		type TemplateDocument
	} from '@docforge/core';
	import { Button, Input, Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@aqardesk/ui';
	import { contractDocumentPaperDimensions } from '#lib/features/contracts/document-design';
	import * as m from '#lib/paraglide/messages';

	type Variable = { key: string; label: string };
	type Props = {
		document: TemplateDocument;
		brandLabel: string;
		footerText: string;
		organizationName: string;
		organizationLogo: string | null;
		variables: Variable[];
		onDocumentChange: (value: TemplateDocument) => void;
		onBrandLabelChange: (value: string) => void;
		onFooterTextChange: (value: string) => void;
		onPaperSizeChange: (value: 'a4' | 'a5' | 'letter') => void;
	};

	let {
		document,
		brandLabel,
		footerText,
		organizationName,
		organizationLogo,
		variables,
		onDocumentChange,
		onBrandLabelChange,
		onFooterTextChange,
		onPaperSizeChange
	}: Props = $props();
	let selectedId = $state('');
	const page = $derived(getPages(document)[0]!);
	const bodyNodes = $derived(
		page.children.filter((node) => node.type !== 'Header' && node.type !== 'Footer')
	);
	const selectedNode = $derived(
		bodyNodes.find((node): node is TextNode => node.id === selectedId && node.type === 'Text')
	);
	const dimensions = $derived(
		getPageDimensions(document.meta.pageSetup.size, document.meta.pageSetup.orientation)
	);

	function addTextBlock(kind: 'Title' | 'Heading' | 'Paragraph' | 'List item') {
		const node = createNode<TextNode>('Text', {
			name: kind,
			props: {
				content: '',
				fontSize: kind === 'Title' ? 22 : kind === 'Heading' ? 17 : 14,
				fontFamily: document.meta.locale === 'ar' ? 'Tahoma' : 'Arial',
				fontWeight: kind === 'Paragraph' ? 'normal' : 'bold',
				color: kind === 'Title' || kind === 'Heading' ? '#17324d' : '#253244',
				textAlign: 'start',
				direction: document.meta.locale === 'ar' ? 'rtl' : 'ltr',
				lineHeight: 1.7
			}
		});
		node.positionMode = 'flow';
		onDocumentChange(addNode(document, page.id, node));
		selectedId = node.id;
	}

	function updateTextNode(props: Record<string, unknown>) {
		if (!selectedNode) return;
		onDocumentChange(updateNodeProps(document, selectedNode.id, { props }));
	}

	function updatePageSize(size: 'A4' | 'A5' | 'Letter') {
		const paperSize = size === 'A5' ? 'a5' : size === 'Letter' ? 'letter' : 'a4';
		const next = {
			...document,
			meta: {
				...document.meta,
				pageSetup: { ...document.meta.pageSetup, size }
			}
		};
		onDocumentChange(next);
		onPaperSizeChange(paperSize);
	}

	function insertVariable(key: string) {
		if (!selectedNode) return;
		updateTextNode({ ...selectedNode.props, content: `${selectedNode.props.content}{{${key}}}` });
	}

	function removeSelected() {
		if (!selectedNode) return;
		onDocumentChange(removeNode(document, selectedNode.id));
		selectedId = '';
	}

	function blockStyle(node: TextNode) {
		return [
			`font-size:${node.props.fontSize}px`,
			`font-family:${node.props.fontFamily}`,
			`font-weight:${node.props.fontWeight ?? 'normal'}`,
			`color:${node.props.color ?? '#253244'}`,
			`text-align:${node.props.textAlign ?? 'start'}`,
			`direction:${node.props.direction ?? 'auto'}`,
			`line-height:${node.props.lineHeight ?? 1.7}`
		].join(';');
	}
</script>

<div class="grid min-h-[min(74vh,860px)] min-w-0 overflow-hidden rounded-xl border border-border bg-muted/40 lg:grid-cols-[220px_minmax(0,1fr)_260px]">
	<aside class="flex flex-col gap-5 border-b border-border bg-card p-4 lg:border-b-0 lg:border-e">
		<section class="grid gap-2">
			<h3 class="text-xs font-semibold uppercase tracking-wide text-muted-foreground">{m.contract_editor_add_content()}</h3>
			<Button variant="outline" class="justify-start" onclick={() => addTextBlock('Title')}>{m.contract_editor_title()}</Button>
			<Button variant="outline" class="justify-start" onclick={() => addTextBlock('Heading')}>{m.contract_editor_heading()}</Button>
			<Button variant="outline" class="justify-start" onclick={() => addTextBlock('Paragraph')}>{m.contract_editor_paragraph()}</Button>
			<Button variant="outline" class="justify-start" onclick={() => addTextBlock('List item')}>{m.contract_editor_list_item()}</Button>
		</section>

		<section class="grid gap-2">
			<h3 class="text-xs font-semibold uppercase tracking-wide text-muted-foreground">{m.contract_design_paper()}</h3>
			<Select
				type="single"
				value={document.meta.pageSetup.size}
				items={[
					{ value: 'A4', label: m.contract_design_paper_a4() },
					{ value: 'A5', label: m.contract_design_paper_a5() },
					{ value: 'Letter', label: m.contract_design_paper_letter() }
				]}
				onValueChange={(value) => {
					if (value === 'A4' || value === 'A5' || value === 'Letter') updatePageSize(value);
				}}
			>
				<SelectTrigger class="h-9 w-full"><SelectValue /></SelectTrigger>
				<SelectContent>
					<SelectItem value="A4" label={m.contract_design_paper_a4()}>{m.contract_design_paper_a4()}</SelectItem>
					<SelectItem value="A5" label={m.contract_design_paper_a5()}>{m.contract_design_paper_a5()}</SelectItem>
					<SelectItem value="Letter" label={m.contract_design_paper_letter()}>{m.contract_design_paper_letter()}</SelectItem>
				</SelectContent>
			</Select>
			<div class="grid grid-cols-2 gap-2">
				<Button
					size="sm"
					variant={document.meta.pageSetup.orientation === 'portrait' ? 'secondary' : 'outline'}
					onclick={() => onDocumentChange({
						...document,
						meta: { ...document.meta, pageSetup: { ...document.meta.pageSetup, orientation: 'portrait' } }
					})}
				>{m.contract_editor_portrait()}</Button>
				<Button
					size="sm"
					variant={document.meta.pageSetup.orientation === 'landscape' ? 'secondary' : 'outline'}
					onclick={() => onDocumentChange({
						...document,
						meta: { ...document.meta, pageSetup: { ...document.meta.pageSetup, orientation: 'landscape' } }
					})}
				>{m.contract_editor_landscape()}</Button>
			</div>
		</section>

		<section class="grid gap-2">
			<h3 class="text-xs font-semibold uppercase tracking-wide text-muted-foreground">{m.contract_editor_header()}</h3>
			{#if organizationLogo}
				<img src={organizationLogo} alt={m.contract_editor_organization_logo()} class="max-h-12 max-w-36 rounded-md border border-border bg-white p-1 object-contain" />
			{/if}
			<Input value={brandLabel} aria-label={m.contract_editor_header()} oninput={(event) => onBrandLabelChange(event.currentTarget.value)} />
		</section>

		<section class="grid gap-2">
			<h3 class="text-xs font-semibold uppercase tracking-wide text-muted-foreground">{m.contract_editor_footer()}</h3>
			<textarea
				value={footerText}
				aria-label={m.contract_editor_footer()}
				oninput={(event) => onFooterTextChange(event.currentTarget.value)}
				class="min-h-20 resize-y rounded-md border border-input bg-background px-3 py-2 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring"
			></textarea>
		</section>
	</aside>

	<div class="min-w-0 overflow-auto p-5 sm:p-8">
		<div class="mx-auto w-fit min-w-full" style={`max-width:${contractDocumentPaperDimensions(document.meta.pageSetup.size === 'A5' ? 'a5' : document.meta.pageSetup.size === 'Letter' ? 'letter' : 'a4').width}`}>
			<article
				dir={document.meta.locale === 'ar' ? 'rtl' : 'ltr'}
				class="mx-auto min-h-[65vh] overflow-hidden rounded-md border border-slate-200 bg-white shadow-xl"
				style={`width:${dimensions.width}mm;min-height:${dimensions.height}mm;max-width:100%;padding:${document.meta.pageSetup.margins.top}mm ${document.meta.pageSetup.margins.right}mm ${document.meta.pageSetup.margins.bottom}mm ${document.meta.pageSetup.margins.left}mm;color:#253244`}
			>
				<header class="mb-7 flex items-center justify-between gap-4 border-b-2 border-amber-500 pb-4">
					<div class="flex min-w-0 items-center gap-3">
						{#if organizationLogo}
							<img src={organizationLogo} alt={m.contract_editor_organization_logo()} class="max-h-12 max-w-24 object-contain" />
						{:else}
							<div class="grid size-12 shrink-0 place-items-center rounded-lg bg-slate-100 text-xs font-bold text-slate-500">{organizationName.slice(0, 2)}</div>
						{/if}
						<div class="min-w-0">
							<p class="truncate text-sm font-bold text-slate-900">{brandLabel || organizationName}</p>
							<p class="text-xs text-slate-500">{m.contract_design_settings()}</p>
						</div>
					</div>
					<span class="shrink-0 rounded-full bg-slate-100 px-3 py-1 font-mono text-xs text-slate-600" dir="ltr">DOC · 001</span>
				</header>

				<div class="grid gap-3">
					{#each bodyNodes as node (node.id)}
						{#if node.type === 'Text'}
							<div class={`group rounded-md border p-1 transition-colors ${selectedId === node.id ? 'border-blue-500 bg-blue-50/40' : 'border-transparent hover:border-slate-200'}`}>
								{#if node.name === 'List item'}
									<div class="flex items-start gap-2">
										<span class="pt-2 text-amber-600">•</span>
										<textarea
											value={node.props.content}
											aria-label={m.contract_editor_block_content()}
											onfocus={() => (selectedId = node.id)}
											oninput={(event) => onDocumentChange(updateNodeProps(document, node.id, { props: { ...node.props, content: event.currentTarget.value } }))}
											class="min-h-8 w-full resize-y border-0 bg-transparent p-1 outline-none"
											style={blockStyle(node)}
										></textarea>
									</div>
								{:else}
									<textarea
										value={node.props.content}
										aria-label={m.contract_editor_block_content()}
										onfocus={() => (selectedId = node.id)}
										oninput={(event) => onDocumentChange(updateNodeProps(document, node.id, { props: { ...node.props, content: event.currentTarget.value } }))}
										class="min-h-8 w-full resize-y border-0 bg-transparent p-1 outline-none"
										style={blockStyle(node)}
										placeholder={m.contract_editor_block_content()}
									></textarea>
								{/if}
							</div>
						{/if}
					{/each}
				</div>

				<footer class="mt-10 border-t border-slate-200 pt-3 text-xs leading-6 text-slate-500">
					{footerText || m.contract_editor_footer_preview()}
				</footer>
			</article>
			<p class="mt-3 text-center text-xs text-muted-foreground">{dimensions.width} × {dimensions.height} mm</p>
		</div>
	</div>

	<aside class="flex flex-col gap-4 border-t border-border bg-card p-4 lg:border-t-0 lg:border-s">
		<div class="flex items-center justify-between gap-2">
			<h3 class="text-sm font-semibold">{m.contract_editor_properties()}</h3>
			{#if selectedNode}
				<Button size="sm" variant="ghost" class="text-destructive" onclick={removeSelected}>{m.contract_editor_remove()}</Button>
			{/if}
		</div>
		{#if selectedNode}
			<label class="grid gap-1 text-xs font-medium text-muted-foreground">
				{m.contract_editor_block_content()}
				<textarea
					value={selectedNode.props.content}
					oninput={(event) => updateTextNode({ ...selectedNode.props, content: event.currentTarget.value })}
					class="min-h-28 rounded-md border border-input bg-background px-3 py-2 text-sm font-normal text-foreground"
				></textarea>
			</label>
			<label class="grid gap-1 text-xs font-medium text-muted-foreground">
				{m.contract_design_font_size()}
				<Input
					type="number"
					min="10"
					max="40"
					value={selectedNode.props.fontSize}
					oninput={(event) => updateTextNode({ ...selectedNode.props, fontSize: Math.min(40, Math.max(10, Number(event.currentTarget.value) || 14)) })}
				/>
			</label>
			<label class="grid gap-1 text-xs font-medium text-muted-foreground">
				{m.contract_design_alignment()}
				<select
					value={selectedNode.props.textAlign ?? 'start'}
					onchange={(event) => updateTextNode({ ...selectedNode.props, textAlign: event.currentTarget.value })}
					class="h-9 rounded-md border border-input bg-background px-3 text-sm text-foreground"
				>
					<option value="start">{m.contract_editor_align_start()}</option>
					<option value="center">{m.contract_editor_align_center()}</option>
					<option value="end">{m.contract_editor_align_end()}</option>
					<option value="justify">{m.contract_editor_align_justify()}</option>
				</select>
			</label>
			<div class="grid gap-2 pt-2">
				<h4 class="text-xs font-semibold text-muted-foreground">{m.settings_template_insert_field()}</h4>
				<div class="max-h-56 overflow-auto rounded-md border border-border">
					{#each variables as variable (variable.key)}
						<button
							type="button"
							onclick={() => insertVariable(variable.key)}
							class="flex w-full items-center justify-between gap-2 border-b border-border/60 px-3 py-2 text-start text-xs last:border-0 hover:bg-accent"
						>
							<span>{variable.label}</span>
							<code class="font-mono text-[10px] text-muted-foreground" dir="ltr">{`{{${variable.key}}}`}</code>
						</button>
					{/each}
				</div>
			</div>
		{:else}
			<p class="rounded-lg border border-dashed border-border p-4 text-sm leading-6 text-muted-foreground">{m.contract_editor_select_block()}</p>
		{/if}
	</aside>
</div>
