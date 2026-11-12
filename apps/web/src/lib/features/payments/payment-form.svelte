<script lang="ts">
	import type { CurrencySettings, PaymentInput, Tenant } from '@aqardesk/sdk';
	import {
		Button,
		Input,
		Select,
		SelectContent,
		SelectItem,
		SelectTrigger,
		SelectValue,
		SheetFooter,
		Textarea,
		toast
	} from '@aqardesk/ui';
	import { desktopApi, type PaymentContract } from '#lib/api/desktop';
	import { getLocale } from '#lib/paraglide/runtime';
	import CurrencySelector from '#lib/features/settings/currency-selector.svelte';
	import FormField from '#lib/shared/components/form-field.svelte';
	import * as m from '#lib/paraglide/messages';

	type Props = {
		tenants: Tenant[];
		currencies: CurrencySettings[];
		contractIds?: string[];
		busy: boolean;
		onSubmit: (input: PaymentInput) => Promise<boolean>;
		onCancel: () => void;
	};

	let { tenants, currencies, contractIds, busy, onSubmit, onCancel }: Props = $props();
	const locale = $derived(getLocale());
	const availableCurrencies = $derived(
		currencies.filter((currency) => currency.is_active && currency.is_enabled_for_organization)
	);
	let tenantId = $state('');
	let tenantContracts = $state<PaymentContract[]>([]);
	let contractsLoading = $state(false);
	let contractLoadError = $state('');
	let allocations = $state<Array<{ contractId: string; amount: string }>>([
		{ contractId: '', amount: '' }
	]);
	let contractRequest = 0;
	let amount = $state('');
	let currencyCode = $state('');
	let paymentMethod = $state('bank_transfer');
	const paymentMethods = ['cash', 'bank_transfer', 'card', 'cheque', 'online', 'other'] as const;
	let receivedAt = $state(localDateTime());
	let externalReference = $state('');
	let notes = $state('');
	let formError = $state('');
	const contractsForCurrency = $derived(
		tenantContracts.filter(
			(contract) =>
				(!contractIds || contractIds.includes(contract.id)) &&
				contract.currency_code === currencyCode
		)
	);
	const eligibleContracts = $derived(
		tenantContracts.filter((contract) => !contractIds || contractIds.includes(contract.id))
	);

	$effect(() => {
		if (!availableCurrencies.some((currency) => currency.code === currencyCode)) {
			currencyCode = availableCurrencies[0]?.code ?? '';
		}
	});

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
		const parsedAllocations = allocations.map((allocation) => ({
			contract: tenantContracts.find(
				(item) =>
					item.id === allocation.contractId &&
					(!contractIds || contractIds.includes(item.id))
			),
			amountMinor: parseAmount(allocation.amount)
		}));
		if (!tenantId) {
			formError = m.payments_select_tenant();
			return;
		}
		if (contractsLoading || contractLoadError || eligibleContracts.length === 0) {
			formError = contractLoadError || m.payments_no_contracts();
			return;
		}
		if (amountMinor === null) {
			formError = m.payments_amount_invalid();
			return;
		}
		if (
			parsedAllocations.some((allocation) => !allocation.contract || allocation.amountMinor === null) ||
			new Set(parsedAllocations.map((allocation) => allocation.contract?.id)).size !==
				parsedAllocations.length
		) {
			formError = m.payments_allocation_invalid();
			return;
		}
		const allocationTotal = parsedAllocations.reduce(
			(total, allocation) => total + (allocation.amountMinor ?? 0),
			0
		);
		if (!Number.isSafeInteger(allocationTotal) || allocationTotal !== amountMinor) {
			formError = m.payments_allocation_total_invalid();
			return;
		}
		if (parsedAllocations.some((allocation) => allocation.contract?.currency_code !== currencyCode)) {
			formError = m.payments_allocation_currency();
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
			allocations: parsedAllocations.map((allocation) => ({
				contract_document_id: allocation.contract!.id,
				amount_minor: allocation.amountMinor!
			})),
			external_reference: externalReference.trim() || null,
			notes: notes.trim() || null
		});
	}

	async function selectTenant(value: string) {
		const request = ++contractRequest;
		tenantId = value;
		tenantContracts = [];
		allocations = [{ contractId: '', amount: '' }];
		amount = '';
		formError = '';
		contractLoadError = '';
		if (!value) {
			contractsLoading = false;
			return;
		}
		contractsLoading = true;
		try {
			const contracts = await desktopApi.listPaymentContracts(value);
			if (request !== contractRequest) return;
			tenantContracts = contracts;
			const firstEnabledContract = contracts.find(
				(contract) =>
					(!contractIds || contractIds.includes(contract.id)) &&
					availableCurrencies.some((currency) => currency.code === contract.currency_code)
			);
			if (firstEnabledContract) currencyCode = firstEnabledContract.currency_code;
		} catch (cause) {
			if (request !== contractRequest) return;
			contractLoadError = cause instanceof Error ? cause.message : String(cause);
			toast.error(contractLoadError);
		} finally {
			if (request === contractRequest) contractsLoading = false;
		}
	}

	function addAllocation() {
		allocations = [...allocations, { contractId: '', amount: '' }];
	}

	function availableContractsFor(index: number) {
		const selectedElsewhere = new Set(
			allocations.filter((_, itemIndex) => itemIndex !== index).map((item) => item.contractId)
		);
		return contractsForCurrency.filter((contract) => !selectedElsewhere.has(contract.id));
	}

	function contractLabel(contract: PaymentContract) {
		return `${contract.contract_number} · ${contract.property_name} (${contract.unit_number}) · ${contract.start_date} – ${contract.end_date}`;
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
			<Select
				type="single"
				value={tenantId}
				items={tenants.map((tenant) => ({ value: tenant.id, label: tenant.full_name }))}
				onValueChange={(selected) => {
					void selectTenant(selected ?? '');
				}}
			>
				<SelectTrigger id="payment-tenant" class="h-9 w-full rounded-md text-sm">
					<SelectValue placeholder={m.payments_select_tenant()} />
				</SelectTrigger>
				<SelectContent>
					{#each tenants as tenant (tenant.id)}
						<SelectItem value={tenant.id} label={tenant.full_name}>{tenant.full_name}</SelectItem>
					{/each}
				</SelectContent>
			</Select>
		</FormField>
		{#if contractsLoading}
			<p class="text-sm text-muted-foreground" role="status">{m.payments_contracts_loading()}</p>
		{:else if tenantId && contractLoadError}
			<p class="text-sm text-destructive" role="alert">{contractLoadError}</p>
		{:else if tenantId && !contractLoadError && eligibleContracts.length === 0}
			<p class="text-sm text-destructive" role="alert">{m.payments_no_contracts()}</p>
		{:else if tenantId && tenantContracts.length > 0}
			<FormField label={m.payments_allocation_column()} id="payment-allocations" description={m.payments_allocation_help()}>
				<div id="payment-allocations" class="grid gap-3">
					{#if contractsForCurrency.length === 0}
						<p class="text-sm text-destructive" role="alert">{m.payments_allocation_currency()}</p>
					{/if}
					{#each allocations as allocation, index (index)}
						<div class="grid gap-2 rounded-md border border-border p-3">
							<div class="flex items-start gap-2">
								<div class="min-w-0 flex-1">
									<Select
										type="single"
										value={allocation.contractId}
										items={availableContractsFor(index).map((contract) => ({
											value: contract.id,
											label: contractLabel(contract)
										}))}
										onValueChange={(selected) => {
											allocations[index] = { ...allocation, contractId: selected ?? '' };
											allocations = [...allocations];
										}}
									>
										<SelectTrigger
											aria-label={`${m.payments_contract()} ${index + 1}`}
											class="h-9 w-full rounded-md text-sm"
										>
											<SelectValue placeholder={m.payments_select_contract()} />
										</SelectTrigger>
										<SelectContent>
											{#each availableContractsFor(index) as contract (contract.id)}
												<SelectItem value={contract.id} label={contractLabel(contract)}>
													{contractLabel(contract)}
												</SelectItem>
											{/each}
										</SelectContent>
									</Select>
								</div>
								{#if allocations.length > 1}
									<Button
										type="button"
										variant="ghost"
										size="sm"
										aria-label={m.payments_remove_allocation()}
										onclick={() => {
											allocations = allocations.filter((_, itemIndex) => itemIndex !== index);
										}}
									>
										{m.payments_remove_allocation()}
									</Button>
								{/if}
							</div>
							<FormField label={m.payments_allocation_amount()} id={`payment-allocation-${index}`}>
								<Input
									id={`payment-allocation-${index}`}
									type="text"
									inputmode="decimal"
									bind:value={allocation.amount}
									placeholder="0.00"
									dir="ltr"
									required
									autocomplete="off"
								/>
							</FormField>
						</div>
					{/each}
					<Button
						type="button"
						variant="outline"
						disabled={allocations.length >= contractsForCurrency.length}
						onclick={addAllocation}
					>
						{m.payments_add_allocation()}
					</Button>
				</div>
			</FormField>
		{/if}
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
					disabled={allocations.some((allocation) => allocation.contractId !== '')}
					onChange={(code) => (currencyCode = code)}
				/>
			</FormField>
		</div>
		<FormField label={m.payments_method()} id="payment-method">
			<Select
				type="single"
				value={paymentMethod}
				items={paymentMethods.map((method) => ({
					value: method,
					label: methodLabel(method)
				}))}
				onValueChange={(selected) => {
					if (selected) paymentMethod = selected;
				}}
			>
				<SelectTrigger id="payment-method" class="h-9 w-full rounded-md text-sm">
					<SelectValue />
				</SelectTrigger>
				<SelectContent>
					{#each paymentMethods as method (method)}
						<SelectItem value={method} label={methodLabel(method)}>{methodLabel(method)}</SelectItem>
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
			<Textarea
				id="payment-notes"
				bind:value={notes}
				rows={3}
				maxlength={2000}
			/>
		</FormField>
	</div>
</form>
<SheetFooter class="shrink-0 border-t border-border bg-popover px-4 py-3">
	<Button type="button" variant="outline" disabled={busy} onclick={onCancel}>{m.common_cancel()}</Button>
	<Button
		type="submit"
		form="payment-form"
		disabled={busy || tenants.length === 0 || availableCurrencies.length === 0 || contractsLoading || contractsForCurrency.length === 0}
	>
		{busy ? m.common_processing() : m.payments_record()}
	</Button>
</SheetFooter>
