<script lang="ts">
	import { onDestroy } from 'svelte';
	import { MoreHorizontal, Plus, Printer } from '@lucide/svelte';
	import {
		Button,
		Sheet,
		SheetContent,
		SheetFooter,
		SheetHeader,
		SheetTitle,
		Input,
		Textarea,
		toast
	} from '@aqardesk/ui';
	import {
		DropdownMenu,
		DropdownMenuContent,
		DropdownMenuItem,
		DropdownMenuSeparator,
		DropdownMenuTrigger
	} from '@aqardesk/ui/components/ui/dropdown-menu';
	import type { ContractDocument } from '#lib/api/desktop';
	import { desktopApi } from '#lib/api/desktop';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { formatGridCell } from '#lib/core/i18n';
	import { getLocale } from '#lib/paraglide/runtime';
	import * as m from '#lib/paraglide/messages';
	import {
		docForgeDocumentFromContract,
		serializeDocForgeTemplate
	} from '#lib/features/contracts/docforge-template';
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
		onEditDraft?: (contract: ContractDocument) => void;
		onIssueDraft?: (contract: ContractDocument) => Promise<boolean>;
		onDeleteDraft?: (contract: ContractDocument) => Promise<boolean>;
		onAmend?: (contract: ContractDocument) => void;
		onEnd?: (contractId: string, effectiveDate: string, reason: string) => Promise<boolean>;
	};

	let {
		contracts,
		loading,
		hasMore = false,
		onloadmore,
		selectedContract = null,
		canCreate = false,
		createDisabled = false,
		onCreate,
		onEditDraft,
		onIssueDraft,
		onDeleteDraft,
		onAmend,
		onEnd
	}: Props = $props();
	const app = getAppContext();
	const rtl = $derived(getLocale() === 'ar');
	let selected = $state<ContractDocument | undefined>();
	let viewerOpen = $state(false);
	let endSheetOpen = $state(false);
	let effectiveDate = $state('');
	let endReason = $state('');
	let ending = $state(false);
	let contractPdfUrl = $state<string | null>(null);
	let contractPdfLoading = $state(false);
	let contractPdfError = $state<string | null>(null);
	let contractPdfFrame = $state<HTMLIFrameElement | null>(null);
	let contractPdfRequest = 0;

	$effect(() => {
		if (selectedContract && selectedContract.id !== selected?.id) {
			openContract(selectedContract);
		}
	});

	onDestroy(() => {
		contractPdfRequest += 1;
		revokeContractPdf();
	});

	const columns: AqarDeskColumnDef[] = $derived([
		{ id: 'contract_number', header: m.contracts_number(), width: 200 },
		{ id: 'tenant_name', header: m.payments_tenant(), width: 190 },
		{ id: 'property_name', header: m.properties_name(), width: 180 },
		{ id: 'unit_number', header: m.contracts_select_unit(), width: 140 },
		{ id: 'template_name', header: m.contracts_template(), width: 190 },
		{
			id: 'status',
			header: m.contracts_status(),
			width: 130,
			accessor: (row) => contractStatusLabel(String(row.status ?? ''))
		},
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

	function contractStatusLabel(status: string) {
		switch (status) {
			case 'draft': return m.contracts_status_draft();
			case 'active': return m.contracts_status_active();
			case 'amended': return m.contracts_status_amended();
			case 'terminated': return m.contracts_status_terminated();
			case 'cancelled': return m.contracts_status_cancelled();
			default: return status;
		}
	}

	function openContract(contract: ContractDocument | undefined) {
		if (!contract) return;
		selected = contract;
		viewerOpen = true;
		void generateContractPdf(contract);
	}

	async function generateContractPdf(contract = selected) {
		if (!contract) return;
		const request = ++contractPdfRequest;
		contractPdfLoading = true;
		contractPdfError = null;
		revokeContractPdf();
		try {
			const documentJson = serializeDocForgeTemplate(
				docForgeDocumentFromContract(contract, m.contracts_disclaimer())
			);
			const bytes = await desktopApi.documentPreviewPdf(documentJson);
			if (request !== contractPdfRequest) return;
			const pdfBytes = new Uint8Array(bytes);
			if (
				pdfBytes.length < 5 ||
				pdfBytes[0] !== 0x25 ||
				pdfBytes[1] !== 0x50 ||
				pdfBytes[2] !== 0x44 ||
				pdfBytes[3] !== 0x46 ||
				pdfBytes[4] !== 0x2d
			) {
				throw new Error('DocForge returned an invalid PDF document.');
			}
			contractPdfUrl = URL.createObjectURL(
				new Blob([pdfBytes.buffer as ArrayBuffer], { type: 'application/pdf' })
			);
		} catch (cause) {
			if (request !== contractPdfRequest) return;
			contractPdfError = cause instanceof Error ? cause.message : String(cause);
			toast.error(contractPdfError);
		} finally {
			if (request === contractPdfRequest) contractPdfLoading = false;
		}
	}

	function revokeContractPdf() {
		if (contractPdfUrl) URL.revokeObjectURL(contractPdfUrl);
		contractPdfUrl = null;
	}

	function printDocument() {
		if (!contractPdfUrl) return;
		if (!contractPdfFrame?.contentWindow) {
			toast.error(m.documents_pdf_viewer_unavailable());
			return;
		}
		contractPdfFrame.contentWindow.print();
	}

	function handleGridAction(event: GridActionEvent) {
		const contract = contracts.find((item) => item.id === String(event.row.id ?? ''));
		if (event.actionId === 'open' || event.actionId === 'dblclick') openContract(contract);
	}

	async function issueDraft() {
		if (!selected || !onIssueDraft) return;
		if (await onIssueDraft(selected)) viewerOpen = false;
	}

	function editDraft() {
		if (!selected || !onEditDraft) return;
		onEditDraft(selected);
		viewerOpen = false;
	}

	function startAmendment() {
		if (!selected || !onAmend) return;
		onAmend(selected);
		viewerOpen = false;
	}

	async function deleteDraft() {
		if (!selected || !onDeleteDraft) return;
		if (!window.confirm(m.contracts_delete_draft_confirm())) return;
		if (await onDeleteDraft(selected)) viewerOpen = false;
	}

	async function endContract() {
		if (!selected || !onEnd) return;
		if (!effectiveDate || !endReason.trim()) {
			toast.error(m.contracts_end_required());
			return;
		}
		ending = true;
		try {
			if (!(await onEnd(selected.id, effectiveDate, endReason.trim()))) return;
			endSheetOpen = false;
			viewerOpen = false;
			effectiveDate = '';
			endReason = '';
		} finally {
			ending = false;
		}
	}

	function openEndSheet() {
		effectiveDate = '';
		endReason = '';
		endSheetOpen = true;
		viewerOpen = false;
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
			<div class="relative min-h-0 flex-1 overflow-hidden bg-muted/40">
				{#if contractPdfLoading}
					<div class="flex h-full items-center justify-center text-sm text-muted-foreground">
						{m.common_loading()}
					</div>
				{:else if contractPdfError}
					<div class="flex h-full flex-col items-center justify-center gap-3 p-6 text-center">
						<p class="text-sm text-destructive" role="alert">{contractPdfError}</p>
						<Button variant="outline" onclick={() => void generateContractPdf()}>
							{m.common_retry()}
						</Button>
					</div>
				{:else if contractPdfUrl}
					<iframe
						bind:this={contractPdfFrame}
						src={contractPdfUrl}
						title={selected.template_name}
						class="h-full w-full border-0"
					></iframe>
				{/if}
			</div>
			<SheetFooter class="shrink-0 flex-row flex-wrap items-center justify-between gap-2 border-t border-border bg-popover px-4 py-3">
				<Button variant="outline" onclick={() => (viewerOpen = false)}>{m.common_cancel()}</Button>
				<DropdownMenu dir={rtl ? 'rtl' : 'ltr'}>
					<DropdownMenuTrigger>
						{#snippet child({ props }: { props: Record<string, unknown> })}
							<Button {...props} variant="default" class="min-w-32">
								<MoreHorizontal size={16} class="me-2" />
								{m.contracts_actions()}
							</Button>
						{/snippet}
					</DropdownMenuTrigger>
					<DropdownMenuContent align="end" side="top" class="min-w-52">
						{#if canCreate && selected.status === 'draft'}
							<DropdownMenuItem onclick={editDraft}>{m.contracts_edit_draft()}</DropdownMenuItem>
							<DropdownMenuItem onclick={issueDraft}>{m.contracts_issue_draft()}</DropdownMenuItem>
							<DropdownMenuItem class="text-destructive focus:text-destructive" onclick={deleteDraft}>
								{m.contracts_delete_draft()}
							</DropdownMenuItem>
							<DropdownMenuSeparator />
						{:else if canCreate && selected.status === 'active'}
							<DropdownMenuItem onclick={startAmendment}>{m.contracts_amend()}</DropdownMenuItem>
							<DropdownMenuItem class="text-destructive focus:text-destructive" onclick={openEndSheet}>
								{m.contracts_end()}
							</DropdownMenuItem>
							<DropdownMenuSeparator />
						{/if}
						<DropdownMenuItem onclick={printDocument}>
							<Printer size={15} class="me-2" />
							{m.contracts_print()}
						</DropdownMenuItem>
					</DropdownMenuContent>
				</DropdownMenu>
			</SheetFooter>
		{/if}
	</SheetContent>
</Sheet>

<Sheet bind:open={endSheetOpen}>
	<SheetContent side="end" class="max-w-lg">
		<SheetHeader class="shrink-0 px-4 py-4">
			<SheetTitle>{m.contracts_end()}</SheetTitle>
		</SheetHeader>
		<div class="grid gap-4 p-4">
			<label class="grid gap-2 text-sm" for="contract-end-effective-date">
				<span>{m.contracts_end_date()}</span>
				<Input id="contract-end-effective-date" type="date" bind:value={effectiveDate} />
			</label>
			<label class="grid gap-2 text-sm" for="contract-end-reason">
				<span>{m.contracts_end_reason()}</span>
				<Textarea id="contract-end-reason" bind:value={endReason} maxlength={1000} rows={4} />
			</label>
		</div>
		<SheetFooter class="shrink-0 border-t border-border bg-popover px-4 py-3">
			<Button variant="outline" onclick={() => (endSheetOpen = false)} disabled={ending}>{m.common_cancel()}</Button>
			<Button variant="destructive" onclick={endContract} disabled={ending}>
				{ending ? m.common_processing() : m.contracts_end()}
			</Button>
		</SheetFooter>
	</SheetContent>
</Sheet>
