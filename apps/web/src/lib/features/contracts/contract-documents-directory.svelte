<script lang="ts">
	import { Plus, Printer } from '@lucide/svelte';
	import {
		Button,
		Sheet,
		SheetContent,
		SheetFooter,
		SheetHeader,
		SheetTitle
	} from '@aqardesk/ui';
	import type { ContractDocument } from '#lib/api/desktop';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { formatGridCell } from '#lib/core/i18n';
	import { getLocale } from '#lib/paraglide/runtime';
	import * as m from '#lib/paraglide/messages';
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

	function printDocument() {
		if (!selected) return;
		const printStyles = document.createElement('style');
		printStyles.textContent = `
			@media print {
				@page { size: A4; margin: 0; }
				body * { visibility: hidden !important; overflow: visible !important; }
				#issued-contract, #issued-contract * { visibility: visible !important; }
				#issued-contract {
					position: absolute !important; inset: 0 !important; width: 100% !important;
					max-width: none !important; min-height: 0 !important; margin: 0 !important;
					padding: 18mm !important; border: 0 !important; border-radius: 0 !important;
					box-shadow: none !important; background: #fff !important; color: #172033 !important;
				}
			}`;
		document.head.append(printStyles);
		window.addEventListener('afterprint', () => printStyles.remove(), { once: true });
		window.print();
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
			{/if}
		</SheetHeader>
		{#if selected}
			<div class="min-h-0 flex-1 overflow-y-auto p-4 scrollbar-none">
				<article
					id="issued-contract"
					dir={selected.template_locale.toLowerCase().startsWith('ar') ? 'rtl' : 'ltr'}
					class="mx-auto min-h-[29.7cm] w-full max-w-[21cm] rounded-md border border-border/70 bg-background px-8 py-10 shadow-sm sm:px-14 sm:py-16"
				>
					<header class="mb-8 border-b-2 border-primary/60 pb-5">
						<p class="text-[10px] font-bold uppercase tracking-[0.16em] text-primary">AqarDesk</p>
						<h3 class="mt-3 text-2xl font-semibold leading-relaxed">{selected.template_name}</h3>
						<p class="mt-2 text-xs text-muted-foreground" dir="ltr">{selected.contract_number}</p>
					</header>
					<div class="grid gap-3 text-sm leading-8">
						{#each documentBlocks(selected.document_body) as block, index (`${block.kind}-${index}`)}
							{#if block.kind === 'heading'}
								<h4 class="pt-2 text-base font-semibold">{block.text.replace(/^#{1,3}\s+/, '')}</h4>
							{:else if block.kind === 'list-item'}
								<p class="before:me-2 before:text-primary before:content-['•']">{block.text.replace(/^(?:[-*+]\s+|\d+[.)]\s+)/, '')}</p>
							{:else}
								<p class="whitespace-pre-wrap">{block.text}</p>
							{/if}
						{/each}
					</div>
					<footer class="mt-12 border-t border-border pt-3 text-[10px] leading-5 text-muted-foreground">
						{m.contracts_disclaimer()}
					</footer>
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
