<script lang="ts">
	import { onDestroy } from 'svelte';
	import { Printer } from '@lucide/svelte';
	import type { Payment, CurrencySettings, OrganizationSettings, ContractTemplate } from '@aqardesk/sdk';
	import { Button, Sheet, SheetContent, SheetFooter, SheetHeader, SheetTitle, toast } from '@aqardesk/ui';
	import type { TemplateDocument } from '@docforge/core';
	import { desktopApi } from '#lib/api/desktop';
	import { formatCurrency, formatDate } from '#lib/core/i18n';
	import { getLocale } from '#lib/paraglide/runtime';
	import * as m from '#lib/paraglide/messages';
	import {
		docForgeDocumentFromPlaceholders,
		docForgeDocumentJsonFromPlaceholders,
		resolveDocForgeDocument,
		serializeDocForgeTemplate
	} from '#lib/features/contracts/docforge-template';

	type Props = {
		payment: Payment | null;
		open?: boolean;
	};

	let { payment, open = $bindable(false) }: Props = $props();
	let organization = $state<OrganizationSettings | null>(null);
	let currencies = $state<CurrencySettings[]>([]);
	let receiptTemplate = $state<ContractTemplate | null>(null);
	let receiptDocForge = $state<TemplateDocument | null>(null);
	let receiptTemplateError = $state<string | null>(null);
	let receiptPdfUrl = $state<string | null>(null);
	let receiptPdfLoading = $state(false);
	let receiptPdfError = $state<string | null>(null);
	let receiptPdfFrame = $state<HTMLIFrameElement | null>(null);
	let loading = $state(false);
	let loadRequest = 0;
	let pdfRequest = 0;
	let lastLoadedPayment: Payment | null = null;
	let lastPdfSignature = '';
	const locale = $derived(getLocale());
	const docForgeValues = $derived.by(() => {
		if (!payment) return {};
		const currentPayment = payment;
		const digits =
			currencies.find((currency) => currency.code === currentPayment.currency_code)?.decimal_digits ?? 2;
		const allocationRows = currentPayment.allocations.map((allocation) => {
			const amount = formatCurrency(allocation.amount_minor, currentPayment.currency_code, digits);
			return {
				contract_number: allocation.contract_number,
				property_name: allocation.property_name,
				unit_number: allocation.unit_number,
				amount,
				amount_minor: allocation.amount_minor,
				display: `${allocation.contract_number} · ${allocation.property_name} (${allocation.unit_number}): ${amount}`
			};
		});
		const firstAllocation = currentPayment.allocations[0];
		const organizationName = organization?.name ?? m.app_name();
		const organizationLegalName = organization?.legal_name ?? organizationName;
		const amount = formatMoney(currentPayment);
		const method = methodLabel(currentPayment.payment_method);
		const reference = currentPayment.external_reference || '—';
		const notes = currentPayment.notes || '—';
		return {
			page: 1,
			items: allocationRows,
			payments: allocationRows,
			payment: {
				id: currentPayment.id,
				number: currentPayment.id,
				receipt_number: currentPayment.id,
				date: formatDate(currentPayment.received_at, 'datetime', undefined, organization?.timezone),
				status:
					currentPayment.status === 'posted'
						? m.payments_status_posted()
						: m.payments_status_void(),
				tenant: currentPayment.tenant_name,
				amount,
				amount_minor: currentPayment.amount_minor,
				currency: currentPayment.currency_code,
				method,
				reference,
				description: notes,
				notes,
				allocations: allocationRows
			},
			tenant: { name: currentPayment.tenant_name, phone: '', email: '', national_id: '' },
			contract: {
				number: firstAllocation?.contract_number ?? '',
				start_date: '',
				end_date: '',
				rent: '',
				deposit: ''
			},
			property: { name: firstAllocation?.property_name ?? '', address: '' },
			unit: { number: firstAllocation?.unit_number ?? '' },
			organization: {
				name: organizationName,
				name_ar: organizationLegalName,
				phone: '',
				website: '',
				address: organization?.legal_name ?? '',
				logo_url: organization?.logo_data_url ?? ''
			}
		};
	});
	const renderedDocForge = $derived.by(() =>
		receiptDocForge
			? resolveDocForgeDocument(receiptDocForge, docForgeValues)
			: { document: null, unresolved: [] as string[] }
	);
	const renderedTemplate = $derived.by(() => {
		if (!receiptTemplate || !payment) {
			return { content: '', unresolved: [] as string[] };
		}
		const currentPayment = payment;
		const templateContent = receiptTemplate.versions[0]?.content_body ?? '';
		const firstAllocation = currentPayment.allocations[0];
		const paymentNotes = currentPayment.notes || '—';
		const values: Record<string, string> = {
			'payment.number': currentPayment.id,
			'payment.receipt_number': currentPayment.id,
			'payment.date': formatDate(currentPayment.received_at, 'datetime', undefined, organization?.timezone),
			'payment.status':
				currentPayment.status === 'posted' ? m.payments_status_posted() : m.payments_status_void(),
			'payment.tenant': currentPayment.tenant_name,
			'payment.amount': formatMoney(currentPayment),
			'payment.allocations': currentPayment.allocations
				.map(
					(allocation) =>
						`${allocation.contract_number} · ${allocation.property_name} (${allocation.unit_number}): ${formatCurrency(
							allocation.amount_minor,
							currentPayment.currency_code,
							currencies.find((currency) => currency.code === currentPayment.currency_code)?.decimal_digits ?? 2
						)}`
				)
				.join('\n') || '—',
			'payment.currency': currentPayment.currency_code,
			'payment.method': methodLabel(currentPayment.payment_method),
			'payment.reference': currentPayment.external_reference || '—',
			'payment.description': paymentNotes,
			'payment.notes': paymentNotes,
			'tenant.name': currentPayment.tenant_name,
			'tenant.phone': '',
			'tenant.email': '',
			'tenant.national_id': '',
			'contract.number': firstAllocation?.contract_number ?? '',
			'contract.start_date': '',
			'contract.end_date': '',
			'contract.rent': '',
			'contract.deposit': '',
			'property.name': firstAllocation?.property_name ?? '',
			'property.address': '',
			'unit.number': firstAllocation?.unit_number ?? '',
			'organization.name': organization?.name ?? m.app_name(),
			'organization.name_ar': organization?.legal_name ?? organization?.name ?? m.app_name(),
			'organization.address': organization?.legal_name ?? '',
			'organization.phone': '',
			'organization.website': ''
		};
		const unresolved = new Set<string>();
		const content = templateContent.replace(
			/\{\{\s*([a-zA-Z0-9_.-]+)\s*\}\}/g,
			(token, key: string) => {
				if (!(key in values)) {
					unresolved.add(key);
					return token;
				}
				return values[key]!;
			}
		);
		return { content, unresolved: [...unresolved] };
	});
	const receiptVariablesUnresolved = $derived(
		(receiptDocForge ? renderedDocForge.unresolved : renderedTemplate.unresolved).length > 0
	);

	$effect(() => {
		const currentPayment = payment;
		if (!open || !currentPayment || currentPayment === lastLoadedPayment) return;
		lastLoadedPayment = currentPayment;
		void loadReceipt(currentPayment);
	});

	$effect(() => {
		if (!open || loading || !payment || !receiptTemplate || receiptTemplateError || receiptVariablesUnresolved) {
			return;
		}
		const signature = JSON.stringify([
			payment.id,
			receiptTemplate.id,
			receiptDocForge ? renderedDocForge.document : renderedTemplate.content
		]);
		if (signature !== lastPdfSignature) {
			lastPdfSignature = signature;
			void generateReceiptPdf();
		}
	});

	onDestroy(() => {
		loadRequest += 1;
		pdfRequest += 1;
		revokeReceiptPdf();
	});

	async function loadReceipt(currentPayment: Payment) {
		const request = ++loadRequest;
		pdfRequest += 1;
		lastPdfSignature = '';
		loading = true;
		receiptTemplate = null;
		receiptDocForge = null;
		receiptTemplateError = null;
		receiptPdfError = null;
		revokeReceiptPdf();
		try {
			const [currentOrganization, availableCurrencies, availableTemplates] = await Promise.all([
				desktopApi.getOrganizationSettings(),
				desktopApi.listCurrencies(),
				desktopApi.listContractTemplates()
			]);
			if (request !== loadRequest || currentPayment !== payment) return;
			organization = currentOrganization;
			currencies = availableCurrencies;
			receiptTemplate = availableTemplates.find((template) => {
				if (
					!template.is_active ||
					template.lease_type !== 'receipt' ||
					!template.locale.toLowerCase().startsWith(locale)
				) {
					return false;
				}
				const version = template.versions[0];
				return Boolean(
					version?.content_body?.trim() ||
						getTemplateDocForgeJson(version?.placeholders_json)
				);
			}) ?? null;
			if (receiptTemplate) {
				const version = receiptTemplate.versions[0];
				const documentJson = getTemplateDocForgeJson(version?.placeholders_json);
				if (documentJson) {
					receiptDocForge = docForgeDocumentFromPlaceholders(
						version?.placeholders_json,
						version?.content_body ?? '',
						receiptTemplate.locale,
						'a5'
					);
					receiptDocForge.meta.documentType = 'receipt';
					receiptDocForge.meta.entityType = 'receipt';
					receiptDocForge.meta.locale = receiptTemplate.locale.toLowerCase().startsWith('ar')
						? 'ar'
						: 'en';
					receiptDocForge.meta.pageSetup.size = 'A5';
					receiptDocForge.meta.pageSetup.orientation = 'portrait';
				}
			}
		} catch (cause) {
			if (request !== loadRequest || currentPayment !== payment) return;
			lastLoadedPayment = null;
			receiptTemplateError = cause instanceof Error ? cause.message : String(cause);
			toast.error(receiptTemplateError);
		} finally {
			if (request === loadRequest && currentPayment === payment) loading = false;
		}
	}

	function formatMoney(item: Payment) {
		const digits =
			currencies.find((currency) => currency.code === item.currency_code)?.decimal_digits ?? 2;
		return formatCurrency(item.amount_minor, item.currency_code, digits);
	}

	function getTemplateDocForgeJson(placeholders: string | null | undefined) {
		if (!placeholders?.includes('"docforge_document"')) return null;
		return docForgeDocumentJsonFromPlaceholders(placeholders);
	}

	function methodLabel(method: string) {
		switch (method) {
			case 'cash': return m.payments_method_cash();
			case 'bank_transfer': return m.payments_method_bank_transfer();
			case 'card': return m.payments_method_card();
			case 'cheque': return m.payments_method_cheque();
			case 'online': return m.payments_method_online();
			default: return m.payments_method_other();
		}
	}

	async function generateReceiptPdf() {
		const currentPayment = payment;
		const currentTemplate = receiptTemplate;
		if (
			!currentPayment ||
			!currentTemplate ||
			receiptTemplateError ||
			receiptVariablesUnresolved
		) {
			return;
		}
		const request = ++pdfRequest;
		receiptPdfLoading = true;
		receiptPdfError = null;
		revokeReceiptPdf();

		try {
			let document: TemplateDocument;
			if (receiptDocForge) {
				const resolved = renderedDocForge.document;
				if (!resolved) throw new Error('The saved DocForge receipt template is invalid.');
				document = resolved;
			} else {
				document = docForgeDocumentFromPlaceholders(
					currentTemplate.versions[0]?.placeholders_json,
					renderedTemplate.content,
					currentTemplate.locale,
					'a5'
				);
			}
			document.meta.documentType = 'receipt';
			document.meta.entityType = 'receipt';
			document.meta.locale = currentTemplate.locale.toLowerCase().startsWith('ar') ? 'ar' : 'en';
			document.meta.pageSetup.size = 'A5';
			document.meta.pageSetup.orientation = 'portrait';

			const bytes = await desktopApi.documentPreviewPdf(serializeDocForgeTemplate(document));
			if (request !== pdfRequest) return;
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
			receiptPdfUrl = URL.createObjectURL(
				new Blob([pdfBytes.buffer as ArrayBuffer], { type: 'application/pdf' })
			);
		} catch (cause) {
			if (request !== pdfRequest) return;
			receiptPdfError = cause instanceof Error ? cause.message : String(cause);
			toast.error(receiptPdfError);
		} finally {
			if (request === pdfRequest) receiptPdfLoading = false;
		}
	}

	function revokeReceiptPdf() {
		if (receiptPdfUrl) URL.revokeObjectURL(receiptPdfUrl);
		receiptPdfUrl = null;
	}

	function printReceipt() {
		if (!receiptPdfUrl) return;
		if (!receiptPdfFrame?.contentWindow) {
			toast.error(m.documents_pdf_viewer_unavailable());
			return;
		}
		receiptPdfFrame.contentWindow.print();
	}

	function retryReceiptPdf() {
		lastPdfSignature = '';
		void generateReceiptPdf();
	}
</script>

<Sheet bind:open>
	<SheetContent
		side="end"
		class="!p-0 max-w-none"
		contentClass="!data-[side=end]:end-0 !data-[side=end]:right-0 !data-[side=end]:h-dvh !data-[side=end]:w-screen"
	>
		<SheetHeader class="shrink-0 px-4 py-4">
			<SheetTitle>{m.payments_receipt()} {payment ? `· ${payment.id}` : ''}</SheetTitle>
			{#if organization}
				<p class="text-xs text-muted-foreground">{organization.name}</p>
			{/if}
		</SheetHeader>
		<div class="relative min-h-0 flex-1 overflow-hidden bg-muted/40">
			{#if loading || receiptPdfLoading}
				<div class="flex h-full items-center justify-center text-sm text-muted-foreground">
					{m.common_loading()}
				</div>
			{:else if !payment}
				<p class="p-6 text-center text-sm text-muted-foreground">{m.payments_not_found()}</p>
			{:else if receiptTemplateError}
				<div class="flex h-full flex-col items-center justify-center gap-3 p-6 text-center">
					<p class="text-sm text-destructive" role="alert">{receiptTemplateError}</p>
					<Button
						variant="outline"
						onclick={() => {
							if (!payment) return;
							lastLoadedPayment = payment;
							void loadReceipt(payment);
						}}
					>
						{m.common_retry()}
					</Button>
				</div>
			{:else if !receiptTemplate}
				<p class="p-6 text-center text-sm text-muted-foreground">
					{m.payments_receipt_template_missing()}
				</p>
			{:else if receiptVariablesUnresolved}
				<p class="p-6 text-center text-sm text-destructive" role="alert">
					{m.payments_receipt_template_variables_missing({
						variables: (receiptDocForge
							? renderedDocForge.unresolved
							: renderedTemplate.unresolved
						).join(', ')
					})}
				</p>
			{:else if receiptPdfError}
				<div class="flex h-full flex-col items-center justify-center gap-3 p-6 text-center">
					<p class="text-sm text-destructive" role="alert">{receiptPdfError}</p>
					<Button variant="outline" onclick={retryReceiptPdf}>{m.common_retry()}</Button>
				</div>
			{:else if receiptPdfUrl}
				<iframe
					bind:this={receiptPdfFrame}
					src={receiptPdfUrl}
					title={m.payments_receipt()}
					class="h-full w-full border-0"
				></iframe>
			{:else}
				<div class="flex h-full items-center justify-center text-sm text-muted-foreground">
					{m.common_loading()}
				</div>
			{/if}
		</div>
		<SheetFooter class="shrink-0 flex-row items-center justify-end gap-2 border-t border-border bg-popover px-4 py-3">
			{#if payment}
				<Button disabled={!receiptPdfUrl} onclick={printReceipt}>
					<Printer size={16} class="me-2" />
					{m.payments_print_receipt()}
				</Button>
			{/if}
		</SheetFooter>
	</SheetContent>
</Sheet>
