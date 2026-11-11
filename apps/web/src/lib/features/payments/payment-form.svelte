<script lang="ts">
	import type { CurrencySettings, PaymentInput, Tenant } from '@aqardesk/sdk';
	import { Button, Input, Select, SelectContent, SelectItem, SelectTrigger, SelectValue, SheetFooter } from '@aqardesk/ui';
	import { getLocale } from '#lib/paraglide/runtime';
	import CurrencySelector from '#lib/features/settings/currency-selector.svelte';
	import FormField from '#lib/shared/components/form-field.svelte';
	import * as m from '#lib/paraglide/messages';

	type Props = {
		tenants: Tenant[];
		currencies: CurrencySettings[];
		busy: boolean;
		onSubmit: (input: PaymentInput) => Promise<boolean>;
		onCancel: () => void;
	};

	let { tenants, currencies, busy, onSubmit, onCancel }: Props = $props();
	const locale = $derived(getLocale());
	const availableCurrencies = $derived(
		currencies.filter((currency) => currency.is_active && currency.is_enabled_for_organization)
	);
	let tenantId = $state('');
	let amount = $state('');
	let currencyCode = $state(availableCurrencies[0]?.code ?? 'SAR');
	let paymentMethod = $state('bank_transfer');
	let receivedAt = $state(localDateTime());
	let externalReference = $state('');
	let notes = $state('');
	let formError = $state('');

	function localDateTime() {
		const now = new Date();
		now.setMinutes(now.getMinutes() - now.getTimezoneOffset());
		return now.toISOString().slice(0, 16);
	}

	function parseAmount(value: string) {
		const currency = availableCurrencies.find((item) => item.code === currencyCode);
		if (!currency) return null;
		const normalized = value.trim().replace(',', '.');
		if (!/^\d+(?:\.\d+)?$/.test(normalized)) return null;
		const [whole, fraction = ''] = normalized.split('.');
		if (fraction.length > currency.decimal_digits) return null;
		const scale = 10 ** currency.decimal_digits;
		const minor =
			Number(whole) * scale +
			Number(fraction.padEnd(currency.decimal_digits, '0') || '0');
		return Number.isSafeInteger(minor) && minor > 0 ? minor : null;
	}

	function submit(event: SubmitEvent) {
		event.preventDefault();
		formError = '';
		const amountMinor = parseAmount(amount);
		const date = new Date(receivedAt);
		if (!tenantId) {
			formError = m.payments_select_tenant();
			return;
		}
		if (amountMinor === null) {
			formError = m.payments_amount_invalid();
			return;
		}
		if (Number.isNaN(date.getTime())) {
			formError = m.payments_date_invalid();
			return;
		}
		void onSubmit({
			tenant_id: tenantId,
			received_at: date.toISOString(),
			amount_minor: amountMinor,
			currency_code: currencyCode,
			payment_method: paymentMethod,
			external_reference: externalReference.trim() || null,
			notes: notes.trim() || null
		});
	}

	function currencyLabel(currency: CurrencySettings) {
		const name = locale === 'ar' ? currency.name_ar || currency.name : currency.name;
		return `${currency.code} · ${name} (${currency.symbol})`;
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
</script>

<form id="payment-form" class="flex min-h-0 flex-1 flex-col" onsubmit={submit}>
	<div class="grid min-h-0 flex-1 content-start gap-3 overflow-y-auto p-4 scrollbar-none">
		{#if formError}<p class="text-sm text-destructive" role="alert">{formError}</p>{/if}
		<FormField label={m.payments_tenant()} id="payment-tenant">
			<select
				id="payment-tenant"
				bind:value={tenantId}
				required
				class="h-9 w-full rounded-md border border-input bg-card px-3 text-sm"
			>
				<option value="" disabled>{m.payments_select_tenant()}</option>
				{#each tenants as tenant (tenant.id)}
					<option value={tenant.id}>{tenant.full_name}</option>
				{/each}
			</select>
		</FormField>
		<div class="grid gap-3 sm:grid-cols-2">
			<FormField label={m.payments_amount()} id="payment-amount">
				<Input
					id="payment-amount"
					type="text"
					inputmode="decimal"
					bind:value={amount}
					placeholder="0.00"
					dir="ltr"
					required
					autocomplete="off"
				/>
			</FormField>
			<FormField label={m.payments_currency()} id="payment-currency">
				<CurrencySelector
					id="payment-currency"
					currencies={availableCurrencies}
					value={currencyCode}
					onChange={(code) => (currencyCode = code)}
				/>
			</FormField>
		</div>
		<FormField label={m.payments_method()} id="payment-method">
			<Select
				type="single"
				value={paymentMethod}
				onValueChange={(selected) => {
					if (selected) paymentMethod = selected;
				}}
			>
				<SelectTrigger id="payment-method" class="h-9 w-full rounded-md text-sm">
					<SelectValue />
				</SelectTrigger>
				<SelectContent>
					{#each ['cash', 'bank_transfer', 'card', 'cheque', 'online', 'other'] as method (method)}
						<SelectItem value={method}>{methodLabel(method)}</SelectItem>
					{/each}
				</SelectContent>
			</Select>
		</FormField>
		<FormField label={m.payments_received_at()} id="payment-received-at">
			<Input id="payment-received-at" type="datetime-local" bind:value={receivedAt} required />
		</FormField>
		<FormField label={m.payments_reference()} id="payment-reference">
			<Input id="payment-reference" bind:value={externalReference} maxlength={120} autocomplete="off" />
		</FormField>
		<FormField label={m.common_notes()} id="payment-notes">
			<textarea
				id="payment-notes"
				bind:value={notes}
				rows="3"
				maxlength={2000}
				class="min-h-20 w-full rounded-md border border-input bg-card px-3 py-2 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring/40"
			></textarea>
		</FormField>
	</div>
</form>
<SheetFooter class="shrink-0 border-t border-border bg-popover px-4 py-3">
	<Button type="button" variant="outline" disabled={busy} onclick={onCancel}>{m.common_cancel()}</Button>
	<Button type="submit" form="payment-form" disabled={busy || tenants.length === 0 || availableCurrencies.length === 0}>
		{busy ? m.common_processing() : m.payments_record()}
	</Button>
</SheetFooter>
