<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import type { PageProps } from './$types';
	import { ArrowLeft, Printer } from '@lucide/svelte';
	import { Button, Card, CardContent, toast } from '@aqardesk/ui';
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

	type ReceiptBlock = { kind: 'heading' | 'paragraph' | 'list-item'; text: string };

	let { data }: PageProps = $props();
	let payment = $state<Payment | null>(null);
	let organization = $state<OrganizationSettings | null>(null);
	let currencies = $state<CurrencySettings[]>([]);
	let templates = $state<ContractTemplate[]>([]);
	let loading = $state(true);
	const locale = $derived(getLocale());
	const receiptTemplate = $derived(
		templates.find(
			(template) =>
				template.is_active &&
				template.lease_type === 'receipt' &&
				template.locale.toLowerCase().startsWith(locale) &&
				template.versions[0]?.content_body?.trim()
		)
	);
	const renderedTemplate = $derived.by(() => {
		if (!receiptTemplate || !payment) return { content: '', unresolved: [] as string[] };
		const values: Record<string, string> = {
			'payment.receipt_number': payment.id,
			'payment.date': formatDate(payment.received_at, 'datetime', undefined, organization?.timezone),
			'payment.status':
				payment.status === 'posted' ? m.payments_status_posted() : m.payments_status_void(),
			'payment.tenant': payment.tenant_name,
			'payment.amount': formatMoney(payment),
			'payment.currency': payment.currency_code,
			'payment.method': methodLabel(payment.payment_method),
			'payment.reference': payment.external_reference || '—',
			'payment.notes': payment.notes || '—',
			'organization.name': organization?.name ?? m.app_name()
		};
		const unresolved = new Set<string>();
		const content = (receiptTemplate.versions[0]?.content_body ?? '').replace(
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
			templates = availableTemplates;
			payment = payments.find((item) => item.id === page.params.paymentId) ?? null;
			if (!payment) toast.error(m.payments_not_found());
		} catch (cause) {
			toast.error(cause instanceof Error ? cause.message : String(cause));
		} finally {
			loading = false;
		}
	}

	function formatMoney(item: Payment) {
		const digits =
			currencies.find((currency) => currency.code === item.currency_code)?.decimal_digits ?? 2;
		return formatCurrency(item.amount_minor, item.currency_code, digits);
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

	function receiptBlocks(content: string): ReceiptBlock[] {
		return content
			.split(/\r?\n/)
			.map((line) => line.trim())
			.filter(Boolean)
			.map((line) => {
				const heading = /^(#{1,3})\s+(.+)$/.exec(line);
				if (heading) return { kind: 'heading', text: heading[2]! };
				const item = /^(?:[-*+]\s+|\d+[.)]\s+)(.+)$/.exec(line);
				if (item) return { kind: 'list-item', text: item[1]! };
				return { kind: 'paragraph', text: line };
			});
	}

	function printReceipt() {
		if (!receiptTemplate || renderedTemplate.unresolved.length > 0) return;
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
			disabled={!receiptTemplate || renderedTemplate.unresolved.length > 0}
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
	{#if !receiptTemplate}
		<p class="mx-auto w-full max-w-2xl rounded-lg border border-dashed border-border p-6 text-center text-sm text-muted-foreground">
			{m.payments_receipt_template_missing()}
		</p>
	{:else if renderedTemplate.unresolved.length > 0}
		<p class="mx-auto w-full max-w-2xl rounded-lg border border-destructive/30 p-6 text-center text-sm text-destructive" role="alert">
			{m.payments_receipt_template_variables_missing({
				variables: renderedTemplate.unresolved.join(', ')
			})}
		</p>
	{:else}
		<Card
			id="payment-receipt"
			dir={receiptTemplate.locale.toLowerCase().startsWith('ar') ? 'rtl' : 'ltr'}
			class="mx-auto w-full max-w-2xl rounded-lg bg-card p-5 shadow-card ring-border/50 sm:p-8 print:max-w-none print:border-0 print:bg-white print:text-slate-950 print:shadow-none"
		>
			<CardContent class="grid gap-4 p-0 text-sm leading-8">
				{#each receiptBlocks(renderedTemplate.content) as block, index (`${block.kind}-${index}`)}
					{#if block.kind === 'heading'}
						{#if index === 0}
							<h2 class="border-b border-border pb-4 text-xl font-semibold print:border-slate-300">
								{block.text}
							</h2>
						{:else}
							<h3 class="pt-2 text-base font-semibold">{block.text}</h3>
						{/if}
					{:else if block.kind === 'list-item'}
						<p class="before:me-2 before:text-primary before:content-['•'] print:before:text-slate-700">
							{block.text}
						</p>
					{:else}
						<p class="whitespace-pre-wrap">{block.text}</p>
					{/if}
				{/each}
			</CardContent>
		</Card>
	{/if}
{/if}

<style>
	@media print {
		@page {
			size: A4;
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
			margin: 0 !important;
			padding: 18mm !important;
			border: 0 !important;
			background: white !important;
			box-shadow: none !important;
		}
	}
</style>
