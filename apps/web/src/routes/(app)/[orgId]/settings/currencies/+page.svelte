<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from '@aqardesk/ui';
	import { AqarDeskGrid, type GridActionEvent, type AqarDeskColumnDef } from '@aqardesk/svelte-grid';
	import type { CurrencySettings, OrganizationSettings } from '@aqardesk/sdk';
	import { desktopApi, type CurrencyEditorInput } from '#lib/api/desktop';
	import CurrencySheet from '#lib/features/settings/currency-sheet.svelte';
	import { getLocale } from '#lib/paraglide/runtime';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { formatGridCell } from '#lib/core/i18n';
	import * as m from '#lib/paraglide/messages';

	const locale = $derived(getLocale());
	const app = getAppContext();
	const canManageCurrencies = $derived(app.currentUser?.role === 'admin');
	const rtl = $derived(locale === 'ar');
	let currencies = $state<CurrencySettings[]>([]);
	let organization = $state<OrganizationSettings | null>(null);
	let loading = $state(true);
	let busyCode = $state<string | null>(null);
	let savingCurrency = $state(false);
	let selectedCurrency = $state<CurrencySettings | null>(null);
	let editSheetOpen = $state(false);

	const sortedCurrencies = $derived(
		currencies
			.filter((currency) => currency.is_active)
			.toSorted((left, right) => left.code.localeCompare(right.code))
	);
	const columns: AqarDeskColumnDef[] = $derived([
		{ id: 'code', header: m.settings_currency_code(), width: 110 },
		{
			id: 'name',
			header: m.settings_currency_name(),
			width: 200,
			accessor: (row) => locale === 'ar' ? row.name_ar || row.name : row.name
		},
		{ id: 'symbol', header: m.settings_currency_symbol(), width: 100 },
		{
			id: 'decimal_digits',
			header: m.settings_currency_decimal_digits(),
			type: 'integer',
			width: 130
		},
		{
			id: 'enabled_status',
			header: m.settings_currency_enabled(),
			type: 'status',
			width: 140,
			accessor: (row) => row.is_enabled_for_organization ? 'enabled' : 'disabled',
			options: [
				{ value: 'enabled', label: m.settings_currency_enabled() },
				{ value: 'disabled', label: m.settings_currency_disabled() }
			]
		},
		{
			id: 'default_status',
			header: m.settings_currency_default(),
			type: 'status',
			width: 160,
			accessor: (row) =>
				row.code === organization?.default_currency ? 'default' : 'not_default',
			options: [
				{ value: 'default', label: m.settings_currency_default() },
				{ value: 'not_default', label: m.settings_currency_not_default() }
			]
		},
		{
			id: 'actions',
			header: m.users_action(),
			type: 'action',
			meta: {
				items: [
					{
						id: 'edit',
						icon: 'Pencil',
						label: m.settings_currency_edit(),
						variant: 'outline',
						visible: () => canManageCurrencies,
						disabled: () => busyCode !== null || savingCurrency
					},
					{
						id: 'enable',
						icon: 'Check',
						label: m.settings_currency_enabled(),
						variant: 'outline',
						visible: (row: Record<string, unknown>) =>
							canManageCurrencies && row.is_enabled_for_organization !== true,
						disabled: () => busyCode !== null || savingCurrency
					},
					{
						id: 'disable',
						icon: 'Ban',
						label: m.settings_currency_disabled(),
						variant: 'outline',
						visible: (row: Record<string, unknown>) =>
							canManageCurrencies && row.is_enabled_for_organization === true,
						disabled: (row: Record<string, unknown>) =>
							busyCode !== null ||
							savingCurrency ||
							row.code === organization?.default_currency
					}
				]
			}
		}
	]);

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
		if (busyCode || savingCurrency) return;
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

	async function saveCurrency(input: CurrencyEditorInput, initial?: CurrencySettings) {
		savingCurrency = true;
		try {
			if (initial) {
				await desktopApi.updateCurrency(initial.code, {
					name: input.name,
					name_ar: input.name_ar,
					symbol: input.symbol,
					symbol_position: input.symbol_position,
					decimal_digits: input.decimal_digits
				});
				toast.success(m.settings_currency_update_success());
			} else {
				await desktopApi.createCurrency(input);
				toast.success(m.settings_currency_create_success());
			}
			await load();
			if (initial) {
				selectedCurrency =
					currencies.find((currency) => currency.code === initial.code) ?? initial;
			}
			return true;
		} catch (cause) {
			toast.error(cause instanceof Error ? cause.message : String(cause));
			return false;
		} finally {
			savingCurrency = false;
		}
	}

	function handleGridAction(event: GridActionEvent) {
		const currency = currencies.find((item) => item.code === String(event.row.code ?? ''));
		if (!currency) return;
		if (event.actionId === 'dblclick') {
			selectedCurrency = currency;
			editSheetOpen = true;
		} else if (event.actionId === 'edit') {
			selectedCurrency = currency;
			editSheetOpen = true;
		} else if (event.actionId === 'enable' || event.actionId === 'disable') {
			void toggle(currency);
		}
	}
</script>

<div class="min-h-0 min-w-0 flex-1">
	<AqarDeskGrid
		{columns}
		source={sortedCurrencies}
		{loading}
		{rtl}
		formatCellValue={(row, column) => formatGridCell(row, column, app.organizationTimezone)}
		readonly
		rowIdKey="code"
		height="100%"
		emptyTitle={m.settings_currencies_description()}
		onaction={handleGridAction}
	>
		{#snippet toolbarActions()}
			{#if canManageCurrencies}
				<CurrencySheet busy={savingCurrency} onSubmit={(input) => saveCurrency(input)} />
			{/if}
		{/snippet}
	</AqarDeskGrid>
</div>

{#if selectedCurrency}
	{#key selectedCurrency.code}
		<CurrencySheet
			initial={selectedCurrency}
			bind:open={editSheetOpen}
			showTrigger={false}
			busy={savingCurrency}
			onSubmit={(input) => saveCurrency(input, selectedCurrency ?? undefined)}
		/>
	{/key}
{/if}
