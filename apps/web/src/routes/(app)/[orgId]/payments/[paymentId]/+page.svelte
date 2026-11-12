<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import type { PageProps } from './$types';
	import { ArrowLeft, Printer } from '@lucide/svelte';
	import { Button, toast } from '@aqardesk/ui';
	import type { TemplateDocument } from '@docforge/core';
	import type {
		ContractTemplate,
		CurrencySettings,
		OrganizationSettings,
		Payment
	} from '@aqardesk/sdk';
	import { desktopApi } from '#lib/api/desktop';
	import { formatCurrency, formatDate } from '#lib/core/i18n';
	import { getLocale } from '#lib/paraglide/runtime';
	import * as m from '#lib/paraglide/messages';
	import {
		docForgeDocumentFromPlaceholders,
		docForgeDocumentJsonFromPlaceholders,
		resolveDocForgeDocument
	} from '#lib/features/contracts/docforge-template';
	import DocForgePreview from '#lib/features/templates/docforge-preview.svelte';

	type ReceiptSection = { title: string; items: string[]; paragraphs: string[] };
	type ReceiptDocument = { title: string; sections: ReceiptSection[]; footer: string[] };

	let { data }: PageProps = $props();
	let payment = $state<Payment | null>(null);
	let organization = $state<OrganizationSettings | null>(null);
	let currencies = $state<CurrencySettings[]>([]);
	let receiptTemplate = $state<ContractTemplate | null>(null);
	let receiptDocForge = $state<TemplateDocument | null>(null);
	let receiptTemplateError = $state<string | null>(null);
	let loading = $state(true);
	const locale = $derived(getLocale());
	const paymentCurrencyCode = $derived(payment?.currency_code ?? '');
	const docForgeValues = $derived.by(() => {
		if (!payment) return {};
		const currentPayment = payment;
		const digits =
			currencies.find((currency) => currency.code === currentPayment.currency_code)?.decimal_digits ?? 2;
		const allocationRows = currentPayment.allocations.map((allocation) => {
			const amount = formatCurrency(
				allocation.amount_minor,
				currentPayment.currency_code,
				digits
			);
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
			return { content: '', unresolved: [] as string[], hasAllocationPlaceholder: false };
		}
		const currentPayment = payment;
		const templateContent = receiptTemplate.versions[0]?.content_body ?? '';
		const hasAllocationPlaceholder = templateContent.includes('{{payment.allocations}}');
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
		return { content, unresolved: [...unresolved], hasAllocationPlaceholder };
	});
	const receiptVariablesUnresolved = $derived(
		(receiptDocForge ? renderedDocForge.unresolved : renderedTemplate.unresolved).length > 0
	);

	onMount(() => {
		void load();
	});

	async function load() {
		try {
			const [payments, currentOrganization, availableCurrencies, availableTemplates] =
				await Promise.all([
				desktopApi.listPayments(),
				desktopApi.getOrganizationSettings(),
				desktopApi.listCurrencies(),
				desktopApi.listContractTemplates()
			]);
			organization = currentOrganization;
			currencies = availableCurrencies;
			payment = payments.find((item) => item.id === page.params.paymentId) ?? null;
			if (!payment) toast.error(m.payments_not_found());
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
			const message = cause instanceof Error ? cause.message : String(cause);
			receiptTemplateError = message;
			toast.error(message);
		} finally {
			loading = false;
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

	function receiptDocument(content: string): ReceiptDocument {
		const result: ReceiptDocument = { title: '', sections: [], footer: [] };
		let section: ReceiptSection | undefined;
		for (const rawLine of content.split(/\r?\n/)) {
			const line = rawLine.trim();
			if (!line) continue;

			const heading = /^(#{1,3})\s+(.+)$/.exec(line);
			if (heading) {
				if (heading[1]!.length === 1 && !result.title) {
					result.title = heading[2]!;
					section = undefined;
				} else {
					section = { title: heading[2]!, items: [], paragraphs: [] };
					result.sections.push(section);
				}
				continue;
			}

			const item = /^(?:[-*+]\s+|\d+[.)]\s+)(.+)$/.exec(line);
			if (item && section) {
				section.items.push(item[1]!);
			} else if (section) {
				section.paragraphs.push(line);
			} else {
				result.footer.push(line);
			}
		}
		return result;
	}

	function receiptField(item: string) {
		const separator = item.search(/[:：]/);
		if (separator < 0) return { label: '', value: item };
		return {
			label: item.slice(0, separator).trim(),
			value: item.slice(separator + 1).trim()
		};
	}

	function printReceipt() {
		const unresolved = receiptDocForge
			? renderedDocForge.unresolved
			: renderedTemplate.unresolved;
		if (!receiptTemplate || receiptTemplateError || unresolved.length > 0) return;
		window.print();
	}
</script>

<svelte:head>
	<title>{m.payments_receipt()} · {organization?.name ?? m.app_name()}</title>
</svelte:head>

<div class="flex items-center justify-between gap-3 print:hidden">
	<Button variant="ghost" class="rounded-md" onclick={() => void goto(`/${page.params.orgId}/payments`)}>
		<ArrowLeft size={16} />
		{m.payments_back_to_list()}
	</Button>
	{#if payment}
		<Button
			variant="outline"
			class="rounded-md"
			disabled={!receiptTemplate || Boolean(receiptTemplateError) || receiptVariablesUnresolved}
			onclick={printReceipt}
		>
			<Printer size={16} />
			{m.payments_print_receipt()}
		</Button>
	{/if}
</div>

{#if loading}
	<p class="py-12 text-center text-sm text-muted-foreground">{m.common_loading()}</p>
{:else if payment}
	{#if receiptTemplateError}
		<p class="mx-auto w-full max-w-2xl rounded-lg border border-destructive/30 p-6 text-center text-sm text-destructive" role="alert">
			{receiptTemplateError}
		</p>
	{:else if !receiptTemplate}
		<p class="mx-auto w-full max-w-2xl rounded-lg border border-dashed border-border p-6 text-center text-sm text-muted-foreground">
			{m.payments_receipt_template_missing()}
		</p>
	{:else if receiptVariablesUnresolved}
		<p class="mx-auto w-full max-w-2xl rounded-lg border border-destructive/30 p-6 text-center text-sm text-destructive" role="alert">
			{m.payments_receipt_template_variables_missing({
				variables: (receiptDocForge
					? renderedDocForge.unresolved
					: renderedTemplate.unresolved
				).join(', ')
			})}
		</p>
	{:else if receiptDocForge}
		<article
			id="payment-receipt"
			dir={receiptTemplate?.locale.toLowerCase().startsWith('ar') ? 'rtl' : 'ltr'}
			class:docforge-receipt={Boolean(receiptDocForge)}
			class="receipt-paper mx-auto w-full max-w-5xl overflow-hidden rounded-xl border border-slate-300 bg-white text-slate-800 shadow-lg print:max-w-none print:rounded-none print:border-0 print:shadow-none"
		>
			<DocForgePreview
				document={receiptDocForge}
				values={docForgeValues}
				id="payment-docforge-preview"
			/>
		</article>
	{:else}
		{@const document = receiptDocument(renderedTemplate.content)}
		<article
			id="payment-receipt"
			dir={receiptTemplate.locale.toLowerCase().startsWith('ar') ? 'rtl' : 'ltr'}
			class="receipt-paper mx-auto w-full max-w-5xl overflow-hidden rounded-xl border border-slate-300 bg-white text-slate-800 shadow-lg print:max-w-none print:rounded-none print:border-0 print:shadow-none"
		>
			<div class="receipt-accent h-3 bg-amber-400"></div>
			<header class="receipt-header grid gap-5 bg-indigo-950 px-6 py-6 text-white sm:grid-cols-[1fr_auto] sm:items-center sm:px-10">
				<div class="flex items-center gap-4">
					{#if organization?.logo_data_url}
						<img src={organization.logo_data_url} alt="" class="max-h-14 max-w-28 rounded bg-white p-1 object-contain print:max-h-12" />
					{/if}
					<div class="grid gap-1">
					<p class="text-xl font-bold tracking-wide">{organization?.name ?? m.app_name()}</p>
					<p class="text-sm text-indigo-100">{m.payments_receipts()}</p>
					</div>
				</div>
				<div class="justify-self-start rounded-full border-2 border-amber-400 bg-amber-300 px-8 py-2 text-center text-xl font-bold text-indigo-950 sm:justify-self-end">
					{document.title || m.payments_receipt()}
				</div>
			</header>

			<div class="receipt-body grid gap-6 p-5 sm:p-9">
				<div class="grid gap-3 rounded-lg border-2 border-indigo-900/70 p-4 sm:grid-cols-[minmax(0,1fr)_auto_auto] sm:items-center">
					<div class="grid gap-1">
						<span class="text-xs font-medium text-slate-500">{m.payments_receipt_number()}</span>
						<strong class="font-mono text-sm" dir="ltr">{payment.id}</strong>
					</div>
					<div class="grid gap-1 sm:px-5 sm:text-center">
						<span class="text-xs font-medium text-slate-500">{m.payments_received_at()}</span>
						<strong class="text-sm">{formatDate(payment.received_at, 'datetime', undefined, organization?.timezone)}</strong>
					</div>
					<div class="grid gap-1 sm:border-s border-indigo-200 sm:ps-5 sm:text-end">
						<span class="text-xs font-medium text-slate-500">{m.payments_status()}</span>
						<strong class={`text-sm ${payment.status === 'void' ? 'text-rose-700' : 'text-emerald-700'}`}>
							{payment.status === 'posted' ? m.payments_status_posted() : m.payments_status_void()}
						</strong>
					</div>
				</div>

				{#each document.sections as section, sectionIndex (section.title)}
					<section class="grid gap-3" class:receipt-payment-section={sectionIndex > 0}>
						<h2 class="border-b border-slate-200 pb-2 text-base font-bold text-indigo-950">
							{section.title}
						</h2>
						{#if section.items.length}
							<div class="grid gap-x-8 gap-y-3 sm:grid-cols-2">
								{#each section.items as item, itemIndex (`${section.title}-${itemIndex}`)}
									{@const field = receiptField(item)}
									<div
										class={`receipt-field grid min-w-0 gap-1 border-b border-slate-200 pb-2 ${field.label === m.payments_amount() ? 'rounded-md border-2 border-indigo-900 bg-amber-100 p-3 text-indigo-950' : ''}`}
										class:receipt-amount={field.label === m.payments_amount()}
									>
										{#if field.label}
											<span class="text-xs text-slate-500">{field.label}</span>
										{/if}
										<strong class="break-words text-sm font-semibold">{field.value}</strong>
									</div>
								{/each}

							</div>
						{/if}
						{#each section.paragraphs as paragraph, paragraphIndex (`${section.title}-paragraph-${paragraphIndex}`)}
							<p class="whitespace-pre-wrap text-sm leading-7 text-slate-700">{paragraph}</p>
						{/each}
					</section>
				{/each}

				{#if payment.allocations.length > 0 && !renderedTemplate.hasAllocationPlaceholder}
					<section class="grid gap-3">
						<h2 class="border-b border-slate-200 pb-2 text-base font-bold text-indigo-950">
							{m.payments_allocation_column()}
						</h2>
						<div class="grid gap-3 sm:grid-cols-2">
							{#each payment.allocations as allocation (allocation.contract_document_id)}
								<div class="grid gap-1 border-b border-slate-200 pb-2">
									<span class="text-xs text-slate-500">
										{allocation.contract_number} · {allocation.property_name} ({allocation.unit_number})
									</span>
									<strong class="text-sm font-semibold">
										{formatCurrency(
											allocation.amount_minor,
											paymentCurrencyCode,
											currencies.find((currency) => currency.code === paymentCurrencyCode)?.decimal_digits ?? 2
										)}
									</strong>
								</div>
							{/each}
						</div>
					</section>
				{/if}

				{#if document.footer.length}
					<div class="grid gap-2 border-t border-slate-200 pt-4 text-sm leading-7 text-slate-600">
						{#each document.footer as line, index (`footer-${index}`)}
							<p>{line}</p>
						{/each}
					</div>
				{/if}

				<div class="receipt-signatures grid gap-6 border-t border-slate-200 pt-6 sm:grid-cols-3">
					{#each [m.payments_signature_cashier(), m.payments_signature_accounts(), m.payments_signature_manager()] as label (label)}
						<div class="grid gap-8 text-center">
							<strong class="text-sm text-indigo-950">{label}</strong>
							<span class="border-b border-dotted border-slate-400"></span>
						</div>
					{/each}
				</div>
			</div>
			<footer class="receipt-footer bg-indigo-950 px-6 py-3 text-center text-xs text-white sm:px-10">
				{organization?.name ?? m.app_name()} · {m.payments_receipt_number()}: <span dir="ltr">{payment.id}</span>
			</footer>
		</article>
	{/if}
{/if}

<style>
	@media print {
		@page {
			size: A5 portrait;
			margin: 0;
		}

		:global(body) {
			background: white !important;
		}

		:global(body *) {
			visibility: hidden !important;
		}

		:global(#payment-receipt),
		:global(#payment-receipt *) {
			visibility: visible !important;
		}

		:global(#payment-receipt) {
			position: absolute !important;
			inset: 0 !important;
			width: 100% !important;
			max-width: none !important;
			min-height: 0 !important;
			margin: 0 !important;
			border: 0 !important;
			background: white !important;
			box-shadow: none !important;
		}

		:global(#payment-receipt.docforge-receipt) {
			overflow: visible !important;
		}

		:global(#payment-receipt .receipt-accent),
		:global(#payment-receipt .receipt-header),
		:global(#payment-receipt .receipt-footer) {
			-webkit-print-color-adjust: exact;
			print-color-adjust: exact;
		}

		:global(#payment-receipt .receipt-body) {
			padding: 10mm !important;
		}

		:global(#payment-receipt .receipt-amount) {
			border: 2px solid #312e81 !important;
			border-radius: 0.5rem;
			background: #fef3c7 !important;
			padding: 0.75rem !important;
			color: #1e1b4b !important;
			-webkit-print-color-adjust: exact;
			print-color-adjust: exact;
		}

		:global(#payment-receipt .receipt-payment-section) {
			break-inside: avoid;
		}

		:global(#payment-receipt .receipt-signatures) {
			break-inside: avoid;
			margin-top: 6mm;
		}
	}
</style>
