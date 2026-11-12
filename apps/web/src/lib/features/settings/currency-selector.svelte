<script lang="ts">
	import {
		Select,
		SelectContent,
		SelectItem,
		SelectTrigger,
		SelectValue
	} from '@aqardesk/ui';
	import type { CurrencySettings } from '@aqardesk/sdk';
	import { getLocale } from '#lib/paraglide/runtime';

	type Props = {
		currencies: CurrencySettings[];
		value: string;
		onChange: (code: string) => void;
		id?: string;
		disabled?: boolean;
	};

	let { currencies, value, onChange, id, disabled = false }: Props = $props();
	const locale = $derived(getLocale());
	const available = $derived(
		currencies.filter((currency) => currency.is_active && currency.is_enabled_for_organization)
	);

	function currencyLabel(currency: CurrencySettings) {
		const name = locale === 'ar' ? currency.name_ar || currency.name : currency.name;
		return `${currency.code} · ${name} (${currency.symbol})`;
	}
</script>

<Select
	type="single"
	{value}
	items={available.map((currency) => ({
		value: currency.code,
		label: currencyLabel(currency)
	}))}
	{disabled}
	onValueChange={(selected) => {
		if (selected) onChange(selected);
	}}
>
	<SelectTrigger {id} class="h-9 w-full min-w-0 rounded-md text-sm">
		<SelectValue class="min-w-0 flex-1 truncate" />
	</SelectTrigger>
	<SelectContent>
		{#each available as currency (currency.code)}
			<SelectItem value={currency.code} label={currencyLabel(currency)}>{currencyLabel(currency)}</SelectItem>
		{/each}
	</SelectContent>
</Select>
