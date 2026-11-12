<script lang="ts">
	import { Plus, Printer } from '@lucide/svelte';
	import {
		Button,
		Sheet,
		SheetContent,
		SheetFooter,
		SheetHeader,
		SheetTitle,
		toast
	} from '@aqardesk/ui';
	import type { ContractDocument } from '#lib/api/desktop';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { formatGridCell } from '#lib/core/i18n';
	import { getLocale } from '#lib/paraglide/runtime';
	import * as m from '#lib/paraglide/messages';
	import {
		buildPrintDocumentHtml,
		contractDocumentFont,
		printHtmlDocument
	} from '#lib/features/contracts/document-design';
	import {
		AqarDeskGrid,
		type AqarDeskColumnDef,
		type GridActionEvent
	} from '@aqardesk/svelte-grid';

	type Props = {
		contracts: ContractDocument[];
		loading: boolean;
		hasMore?: boolean;
		onloadmore?: () => Promise<unknown>;
		selectedContract?: ContractDocument | null;
		canCreate?: boolean;
		createDisabled?: boolean;
		onCreate?: () => void;
	};

	let {
		contracts,
		loading,
		hasMore = false,
		onloadmore,
		selectedContract = null,
		canCreate = false,
		createDisabled = false,
		onCreate
	}: Props = $props();
	const app = getAppContext();
	const rtl = $derived(getLocale() === 'ar');
	let selected = $state<ContractDocument | undefined>();
	let viewerOpen = $state(false);

	$effect(() => {
		if (selectedContract) openContract(selectedContract);
	});

	const columns: AqarDeskColumnDef[] = $derived([
		{ id: 'contract_number', header: m.contracts_number(), width: 200 },
		{ id: 'tenant_name', header: m.payments_tenant(), width: 190 },
		{ id: 'property_name', header: m.properties_name(), width: 180 },
		{ id: 'unit_number', header: m.contracts_select_unit(), width: 140 },
		{ id: 'template_name', header: m.contracts_template(), width: 190 },
		{ id: 'start_date', header: m.contracts_start_date(), width: 130 },
		{ id: 'end_date', header: m.contracts_end_date(), width: 130 },
		{
			id: 'rent_amount_minor',
			header: m.contracts_rent(),
			width: 150,
			accessor: (row) =>
				`${(Number(row.rent_amount_minor) / 10 ** Number(row.currency_decimal_digits)).toFixed(Number(row.currency_decimal_digits))} ${String(row.currency_code)}`
		},
		{ id: 'created_at', header: m.settings_created_at(), type: 'datetime', width: 190 },
		{
			id: 'actions',
			header: m.users_action(),
			type: 'action',
			meta: {
				items: [{ id: 'open', icon: 'Eye', label: m.contracts_open(), variant: 'outline' }]
			}
		}
	]);

	function openContract(contract: ContractDocument | undefined) {
		if (!contract) return;
		selected = contract;
		viewerOpen = true;
	}

	function handleGridAction(event: GridActionEvent) {
		const contract = contracts.find((item) => item.id === String(event.row.id ?? ''));
		if (event.actionId === 'open' || event.actionId === 'dblclick') openContract(contract);
	}

	function documentBlocks(content: string) {
		return content
			.split(/\r?\n/)
			.map((line) => line.trim())
			.filter(Boolean)
			.map((line) => {
				const heading = /^(#{1,3})\s+(.+)$/.exec(line);
				if (heading) return { kind: 'heading', text: heading[2] };
				const item = /^(?:[-*+]\s+|\d+[.)]\s+)(.+)$/.exec(line);
				if (item) return { kind: 'list-item', text: item[1] };
				return { kind: 'paragraph', text: line };
			});
	}

	function escapeHtml(value: string) {
		return value
			.replaceAll('&', '&amp;')
			.replaceAll('<', '&lt;')
			.replaceAll('>', '&gt;')
			.replaceAll('"', '&quot;')
			.replaceAll("'", '&#39;');
	}

	async function printDocument() {
		if (!selected) return;
		const design = selected.document_style;
		const hasHeader =
			(design.show_branding && design.brand_label.trim()) || design.show_contract_number;
		const headerHtml = hasHeader
			? `<header>${design.show_branding && design.brand_label.trim() ? `<p class="brand">${escapeHtml(design.brand_label)}</p>` : ''}${design.show_contract_number ? `<p class="contract-number">${escapeHtml(selected.contract_number)}</p>` : ''}</header>`
			: '';
		const bodyHtml = documentBlocks(selected.document_body)
			.map((block) => {
				if (block.kind === 'heading') {
					return `<h2>${escapeHtml(block.text.replace(/^#{1,3}\s+/, ''))}</h2>`;
				}
				if (block.kind === 'list-item') {
					return `<p><span class="bullet">• </span>${escapeHtml(block.text.replace(/^(?:[-*+]\s+|\d+[.)]\s+)/, ''))}</p>`;
				}
				return `<p>${escapeHtml(block.text)}</p>`;
			})
			.join('');
		const footerHtml = design.show_footer
			? `<footer>${escapeHtml(design.footer_text.trim() || m.contracts_disclaimer())}</footer>`
			: '';
		try {
			await printHtmlDocument(
				buildPrintDocumentHtml({
					title: selected.template_name,
					direction: selected.template_locale.toLowerCase().startsWith('ar') ? 'rtl' : 'ltr',
					design,
					headerHtml,
					bodyHtml,
					footerHtml
				})
			);
		} catch (cause) {
			toast.error(cause instanceof Error ? cause.message : String(cause));
		}
	}
</script>

<div class="min-h-0 min-w-0 flex-1">
	<AqarDeskGrid
		columns={columns}
		source={contracts}
		{loading}
		{rtl}
		formatCellValue={(row, column) => formatGridCell(row, column, app.organizationTimezone)}
		readonly
		height="100%"
		{hasMore}
		{onloadmore}
		emptyTitle={m.contracts_empty_title()}
		emptyDescription={m.contracts_empty_description()}
		onaction={handleGridAction}
	>
		{#snippet toolbarActions()}
			{#if canCreate && onCreate}
				<Button class="rounded-md" disabled={createDisabled} onclick={onCreate}>
					<Plus size={16} />
					{m.contracts_new_contract()}
				</Button>
			{/if}
		{/snippet}
	</AqarDeskGrid>
</div>

<Sheet bind:open={viewerOpen}>
	<SheetContent
		side="end"
		class="!p-0 max-w-none"
		contentClass="!data-[side=end]:end-0 !data-[side=end]:right-0 !data-[side=end]:h-dvh !data-[side=end]:w-screen"
	>
		<SheetHeader class="shrink-0 px-4 py-4">
			<SheetTitle>{selected?.template_name ?? m.contracts_preview()}</SheetTitle>
			{#if selected}
				<p class="text-xs text-muted-foreground" dir="ltr">{selected.contract_number}</p>
				<p class="text-xs text-muted-foreground">{m.contracts_design_snapshot_notice()}</p>
			{/if}
		</SheetHeader>
		{#if selected}
			<div class="min-h-0 flex-1 overflow-y-auto p-4 scrollbar-none">
				<article
					id="issued-contract"
					dir={selected.template_locale.toLowerCase().startsWith('ar') ? 'rtl' : 'ltr'}
					class="mx-auto w-full rounded-md border border-border/70 bg-white text-slate-900 shadow-sm"
					style={`max-width:${selected.document_style.paper_size === 'a4' ? '210mm' : '215.9mm'};min-height:${selected.document_style.paper_size === 'a4' ? '297mm' : '279.4mm'};padding:${selected.document_style.page_margin_mm}mm;font-family:${contractDocumentFont(selected.document_style.font_family)};font-size:${selected.document_style.font_size}px;text-align:${selected.document_style.alignment};background:#fff;color:#172033`}
				>
					{#if (selected.document_style.show_branding && selected.document_style.brand_label.trim()) || selected.document_style.show_contract_number}
						<header class="mb-8 border-b-2 pb-5" style={`border-color:${selected.document_style.accent_color}`}>
							{#if selected.document_style.show_branding && selected.document_style.brand_label.trim()}
								<p class="text-sm font-bold" style={`color:${selected.document_style.accent_color}`}>
									{selected.document_style.brand_label}
								</p>
							{/if}
							{#if selected.document_style.show_contract_number}
								<p class="text-sm text-muted-foreground" dir="ltr">{selected.contract_number}</p>
							{/if}
						</header>
					{/if}
					<div class="grid gap-3 leading-8">
						{#each documentBlocks(selected.document_body) as block, index (`${block.kind}-${index}`)}
							{#if block.kind === 'heading'}
								<h4 class="border-b pb-2 pt-4 text-lg font-semibold" style={`border-color:${selected.document_style.accent_color};color:${selected.document_style.accent_color}`}>{block.text.replace(/^#{1,3}\s+/, '')}</h4>
							{:else if block.kind === 'list-item'}
								<p><span style={`color:${selected.document_style.accent_color}`}>• </span>{block.text.replace(/^(?:[-*+]\s+|\d+[.)]\s+)/, '')}</p>
							{:else}
								<p class="whitespace-pre-wrap">{block.text}</p>
							{/if}
						{/each}
					</div>
					{#if selected.document_style.show_footer}
						<footer class="mt-12 border-t pt-3 text-xs leading-6 text-muted-foreground" style={`border-color:${selected.document_style.accent_color}`}>
							{selected.document_style.footer_text.trim() || m.contracts_disclaimer()}
						</footer>
					{/if}
				</article>
			</div>
			<SheetFooter class="shrink-0 border-t border-border bg-popover px-4 py-3">
				<Button variant="outline" onclick={() => (viewerOpen = false)}>{m.common_cancel()}</Button>
				<Button onclick={printDocument}>
					<Printer size={15} class="me-2" />
					{m.contracts_print()}
				</Button>
			</SheetFooter>
		{/if}
	</SheetContent>
</Sheet>
