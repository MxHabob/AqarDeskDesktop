<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from '@aqardesk/ui';
	import type { CurrencySettings, OrganizationSettings } from '@aqardesk/sdk';
	import { desktopApi } from '#lib/api/desktop';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { getLocale } from '#lib/paraglide/runtime';
	import * as m from '#lib/paraglide/messages';

	const app = getAppContext();
	const locale = $derived(getLocale());
	let currencies = $state<CurrencySettings[]>([]);
	let organization = $state<OrganizationSettings | null>(null);
	let loading = $state(true);
	let busyCode = $state<string | null>(null);

	const sortedCurrencies = $derived(
		currencies
			.filter((currency) => currency.is_active)
			.toSorted((left, right) => left.code.localeCompare(right.code))
	);

	onMount(() => {
		void load();
	});

	async function load() {
		loading = true;
		try {
			const [availableCurrencies, currentOrganization] = await Promise.all([
				desktopApi.listCurrencies(),
				desktopApi.getOrganizationSettings()
			]);
			currencies = availableCurrencies;
			organization = currentOrganization;
		} catch (cause) {
			toast.error(cause instanceof Error ? cause.message : String(cause));
		} finally {
			loading = false;
		}
	}

	async function toggle(currency: CurrencySettings) {
		if (busyCode) return;
		if (currency.code === organization?.default_currency && currency.is_enabled_for_organization) {
			toast.error(m.settings_currency_disable_default());
			return;
		}

		busyCode = currency.code;
		try {
			await desktopApi.setOrganizationCurrencyEnabled(
				currency.code,
				!currency.is_enabled_for_organization
			);
			currencies = currencies.map((item) =>
				item.code === currency.code
					? { ...item, is_enabled_for_organization: !item.is_enabled_for_organization }
					: item
			);
			toast.success(m.settings_currency_change_success());
		} catch (cause) {
			toast.error(cause instanceof Error ? cause.message : String(cause));
		} finally {
			busyCode = null;
		}
	}

	function displayName(currency: CurrencySettings) {
		return locale === 'ar' ? currency.name_ar || currency.name : currency.name;
	}
</script>

<header class="grid gap-1">
	<h2 class="text-lg font-semibold">{m.settings_currencies()}</h2>
	<p class="text-sm text-muted-foreground">{m.settings_currencies_description()}</p>
</header>

{#if loading}
	<div class="py-8 text-sm text-muted-foreground" role="status">{m.common_processing()}</div>
{:else}
	<div class="divide-y divide-border/70">
		{#each sortedCurrencies as currency (currency.code)}
			{@const isDefault = currency.code === organization?.default_currency}
			<div class="flex items-center justify-between gap-4 py-3">
				<div class="min-w-0">
					<p class="truncate text-sm font-medium">
						{currency.code} · {displayName(currency)} ({currency.symbol})
					</p>
					{#if isDefault}
						<p class="mt-1 text-xs text-muted-foreground">{m.settings_currency_default()}</p>
					{/if}
				</div>
				<button
					type="button"
					role="switch"
					aria-checked={currency.is_enabled_for_organization}
					aria-label={`${displayName(currency)}: ${currency.is_enabled_for_organization ? m.settings_currency_enabled() : m.settings_currency_disabled()}`}
					disabled={busyCode !== null || (isDefault && currency.is_enabled_for_organization)}
					onclick={() => void toggle(currency)}
					class="min-w-24 rounded-md border border-border px-3 py-1.5 text-xs font-medium transition-colors disabled:cursor-not-allowed disabled:opacity-60"
				>
					{currency.is_enabled_for_organization
						? m.settings_currency_enabled()
						: m.settings_currency_disabled()}
				</button>
			</div>
		{/each}
	</div>
{/if}
