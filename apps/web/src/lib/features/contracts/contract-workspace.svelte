<script lang="ts">
	import { createForm } from '@tanstack/svelte-form';
	import { createInfiniteQuery } from '@tanstack/svelte-query';
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import {
		Button,
		Card,
		CardContent,
		Input,
		Sheet,
		SheetContent,
		SheetFooter,
		SheetHeader,
		SheetTitle,
		Select,
		SelectContent,
		SelectItem,
		SelectTrigger,
		toast
	} from '@aqardesk/ui';
	import { z } from 'zod';
	import type {
		ContractTemplate,
		CurrencySettings,
		Property,
		PropertyUnit,
		Tenant
	} from '@aqardesk/sdk';
	import { desktopApi } from '#lib/api/desktop';
	import type { ContractDocument } from '#lib/api/desktop';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { queryClient } from '#lib/core/query-client';
	import { fetchNextPageOrThrow } from '#lib/core/infinite-query';
	import { contractDocumentQueryKeys } from '#lib/features/contracts/queries';
	import { hasPermission } from '#lib/features/auth/permissions';
	import ContractDocumentsDirectory from '#lib/features/contracts/contract-documents-directory.svelte';
	import FormField from '#lib/shared/components/form-field.svelte';
	import { getFieldError } from '#lib/shared/form-errors';
	import * as m from '#lib/paraglide/messages';
	import CurrencySelector from '#lib/features/settings/currency-selector.svelte';

	type ContractFormValues = {
		templateId: string;
		tenantId: string;
		propertyId: string;
		unitId: string;
		startDate: string;
		endDate: string;
		currencyCode: string;
		rentAmount: string;
		depositAmount: string;
	};

	const app = getAppContext();
	type Props = { propertyId?: string };
	let { propertyId }: Props = $props();
	let templates = $state<ContractTemplate[]>([]);
	let properties = $state<Property[]>([]);
	let units = $state<PropertyUnit[]>([]);
	let tenants = $state<Tenant[]>([]);
	let currencies = $state<CurrencySettings[]>([]);
	let organizationDefaultCurrency = $state('');
	let selectedContractForPreview = $state<ContractDocument | null>(null);
	let loading = $state(true);
	let loadError = $state('');
	let preparing = $state(false);
	let builderOpen = $state(false);
	let currencyManuallySelected = $state(false);
	const canViewContracts = $derived(
		hasPermission(app.currentUser?.role, 'view_contracts', app.currentUser?.permissions)
	);
	const canCreateContracts = $derived(
		hasPermission(app.currentUser?.role, 'create_contracts', app.currentUser?.permissions)
	);
	const pageSize = 50;
	const contractDocumentsQuery = createInfiniteQuery(() => ({
		queryKey: contractDocumentQueryKeys.pages(
			app.currentUser?.organization_id ?? '',
			propertyId
		),
		queryFn: ({ pageParam }) =>
			desktopApi.listContractDocumentsPage(pageSize, pageParam, propertyId),
		initialPageParam: 0,
		getNextPageParam: (lastPage, pages) =>
			lastPage.has_more ? pages.length * pageSize : undefined,
		enabled: !!app.currentUser && canViewContracts
	}));
	const contractDocuments = $derived(
		contractDocumentsQuery.data?.pages.flatMap((page) => page.items) ?? []
	);

	const activeTemplates = $derived(
		templates.filter(
			(template) =>
				template.lease_type !== 'receipt' &&
				template.is_active &&
				template.versions[0]?.content_body?.trim()
		)
	);
	const activeProperties = $derived(
		properties.filter(
			(property) =>
				(!propertyId || property.id === propertyId) &&
				property.status === 'active' &&
				units.some((unit) => unit.property_id === property.id && unit.status !== 'inactive')
		)
	);
	const enabledCurrencies = $derived(
		currencies.filter((currency) => currency.is_active && currency.is_enabled_for_organization)
	);

	function currencyDigits(currencyCode: string) {
		return currencies.find((currency) => currency.code === currencyCode)?.decimal_digits ?? 2;
	}

	function currencyForUnit(unit: PropertyUnit | undefined, sourceCurrencies: CurrencySettings[]) {
		const enabled = sourceCurrencies.filter(
			(currency) => currency.is_active && currency.is_enabled_for_organization
		);
		if (enabled.some((currency) => currency.code === organizationDefaultCurrency)) {
			return organizationDefaultCurrency;
		}
		if (unit?.currency_code && enabled.some((currency) => currency.code === unit.currency_code)) {
			return unit.currency_code;
		}
		return enabled[0]?.code ?? '';
	}

	function parseContractAmount(value: string, currencyCode: string): number | null {
		if (!/^\d+(?:\.\d+)?$/.test(value)) return null;
		const digits = currencyDigits(currencyCode);
		const [whole, fraction = ''] = value.split('.');
		if (fraction.length > digits) return null;
		const amountMinor = Number(`${whole}${fraction.padEnd(digits, '0')}`);
		return Number.isSafeInteger(amountMinor) && amountMinor >= 0 ? amountMinor : null;
	}

	function localDateValue() {
		const today = new Date();
		const year = today.getFullYear();
		const month = String(today.getMonth() + 1).padStart(2, '0');
		const day = String(today.getDate()).padStart(2, '0');
		return `${year}-${month}-${day}`;
	}

	function isValidDate(value: string) {
		const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value);
		if (!match) return false;
		const year = Number(match[1]);
		const month = Number(match[2]);
		const day = Number(match[3]);
		const date = new Date(year, month - 1, day);
		return date.getFullYear() === year && date.getMonth() === month - 1 && date.getDate() === day;
	}

	function isValidAmount(value: string, currencyCode: string, required: boolean) {
		if (!value) return !required;
		const amount = parseContractAmount(value, currencyCode);
		return amount !== null && (!required || amount > 0);
	}

	const contractSchema = z.object({
		templateId: z.string().min(1, m.contracts_no_templates()),
		tenantId: z.string().min(1, m.contracts_no_tenants()),
		propertyId: z.string().min(1, m.contracts_no_units()),
		unitId: z.string().min(1, m.contracts_no_units()),
		startDate: z.string().min(1, m.contracts_invalid_dates()).refine((value) => !value || isValidDate(value), m.contracts_invalid_dates()),
		endDate: z.string().min(1, m.contracts_invalid_dates()).refine((value) => !value || isValidDate(value), m.contracts_invalid_dates()),
		currencyCode: z.string().min(1, m.contracts_currency_required()),
		rentAmount: z.string().min(1, m.contracts_amount_invalid()),
		depositAmount: z.string()
	}).superRefine((values, context) => {
		if (values.startDate && values.endDate && values.startDate > values.endDate) {
			context.addIssue({
				code: 'custom',
				path: ['endDate'],
				message: m.contracts_invalid_dates()
			});
		}
		if (!enabledCurrencies.some((currency) => currency.code === values.currencyCode)) {
			context.addIssue({
				code: 'custom',
				path: ['currencyCode'],
				message: m.contracts_currency_required()
			});
		}
		if (!isValidAmount(values.rentAmount, values.currencyCode, true)) {
			context.addIssue({
				code: 'custom',
				path: ['rentAmount'],
				message: m.contracts_amount_invalid()
			});
		}
		if (!isValidAmount(values.depositAmount, values.currencyCode, false)) {
			context.addIssue({
				code: 'custom',
				path: ['depositAmount'],
				message: m.contracts_amount_invalid()
			});
		}
	});

	const form = createForm(() => ({
		defaultValues: {
			templateId: '',
			tenantId: '',
			propertyId: '',
			unitId: '',
			startDate: localDateValue(),
			endDate: '',
			currencyCode: '',
			rentAmount: '',
			depositAmount: ''
		} satisfies ContractFormValues,
		validators: { onSubmit: contractSchema },
		onSubmit: async ({ value }) => {
			try {
				await saveContract(value);
			} catch (cause) {
				app.setError(cause);
			}
		}
	}));
	const propertyUnits = $derived(
		units.filter(
			(unit) =>
				unit.property_id === form.state.values.propertyId && unit.status !== 'inactive'
		)
	);
	const selectedTemplateName = $derived(
		activeTemplates.find((template) => template.id === form.state.values.templateId)?.name
	);
	const selectedTenantName = $derived(
		tenants.find((tenant) => tenant.id === form.state.values.tenantId)?.full_name
	);
	const selectedPropertyName = $derived(
		activeProperties.find((property) => property.id === form.state.values.propertyId)?.name
	);
	const selectedUnit = $derived(
		propertyUnits.find((unit) => unit.id === form.state.values.unitId)
	);
	function unitTypeLabel(unitType: string) {
		switch (unitType) {
			case 'apartment': return m.units_type_apartment();
			case 'office': return m.units_type_office();
			case 'retail': return m.units_type_retail();
			case 'warehouse': return m.units_type_warehouse();
			case 'parking': return m.units_type_parking();
			default: return m.common_other();
		}
	}
	const selectedUnitName = $derived(
		selectedUnit ? `${selectedUnit.unit_number} · ${unitTypeLabel(selectedUnit.unit_type)}` : undefined
	);
	const selectedCurrency = $derived(
		form.state.values.currencyCode
	);
	const isReadonly = $derived(!canCreateContracts);
	const canStartContract = $derived(
		!loading &&
			!loadError &&
			activeTemplates.length > 0 &&
			tenants.length > 0 &&
			activeProperties.length > 0 &&
			enabledCurrencies.length > 0
	);

	onMount(() => {
		if (canViewContracts) void load();
		else loading = false;
	});

	$effect(() => {
		if (!contractDocumentsQuery.error) return;
		const error = String(contractDocumentsQuery.error);
		if (error !== loadError) toast.error(error);
	});

	async function load() {
		loading = true;
		loadError = '';
		currencyManuallySelected = false;
		try {
			if (!canCreateContracts) {
				templates = [];
				properties = [];
				units = [];
				tenants = [];
				currencies = [];
				return;
			}
			const [
				nextTemplates,
				nextProperties,
				nextUnits,
				nextTenants,
				nextCurrencies,
				organizationSettings
			] =
				await Promise.all([
					desktopApi.listAvailableContractTemplates(),
					desktopApi.listProperties(),
					desktopApi.listUnits(),
					desktopApi.listTenants(),
					desktopApi.listCurrencies(),
					desktopApi.getOrganizationSettings()
				]);
			templates = nextTemplates;
			properties = nextProperties;
			units = nextUnits;
			tenants = nextTenants;
			currencies = nextCurrencies;
			organizationDefaultCurrency = organizationSettings.default_currency;

			const availableProperties = nextProperties.filter(
				(property) =>
					property.status === 'active' &&
					nextUnits.some((unit) => unit.property_id === property.id && unit.status !== 'inactive')
			);
			const requestedPropertyId = propertyId ?? page.url.searchParams.get('propertyId');
			const firstProperty = requestedPropertyId
				? availableProperties.find((property) => property.id === requestedPropertyId)
				: availableProperties[0];
			const requestedTenantId = page.url.searchParams.get('tenantId');
			const firstTenant = requestedTenantId
				? nextTenants.find((tenant) => tenant.id === requestedTenantId)
				: nextTenants[0];
			const firstUnit = nextUnits.find(
				(unit) => unit.property_id === firstProperty?.id && unit.status !== 'inactive'
			);
			form.setFieldValue(
				'templateId',
				nextTemplates.find(
					(template) =>
						template.lease_type !== 'receipt' &&
						template.is_active &&
						template.versions[0]?.content_body?.trim()
				)?.id ?? ''
			);
			form.setFieldValue('tenantId', firstTenant?.id ?? '');
			form.setFieldValue('propertyId', firstProperty?.id ?? '');
			form.setFieldValue('unitId', firstUnit?.id ?? '');
			form.setFieldValue('startDate', localDateValue());
			form.setFieldValue('rentAmount', '');
			form.setFieldValue('depositAmount', '');
			form.setFieldValue('endDate', '');
			updateAmountsForUnit(firstUnit?.id ?? '', nextUnits, nextCurrencies);
			if (page.url.searchParams.get('create') === '1') builderOpen = true;
		} catch (cause) {
			loadError = cause instanceof Error ? cause.message : String(cause);
			toast.error(loadError);
		} finally {
			loading = false;
		}
	}

	function updateAmountsForUnit(
		unitId: string,
		sourceUnits: PropertyUnit[] = units,
		sourceCurrencies: CurrencySettings[] = currencies
	) {
		const unit = sourceUnits.find((item) => item.id === unitId);
		const enabled = sourceCurrencies.filter(
			(currency) => currency.is_active && currency.is_enabled_for_organization
		);
		const currentCurrencyCode = form.state.values.currencyCode;
		const nextCurrencyCode =
			currencyManuallySelected && enabled.some((currency) => currency.code === currentCurrencyCode)
				? currentCurrencyCode
				: currencyForUnit(unit, sourceCurrencies);
		const currencyChanged = nextCurrencyCode !== form.state.values.currencyCode;
		form.setFieldValue('currencyCode', nextCurrencyCode);
		if (currencyChanged) form.setFieldValue('depositAmount', '');
		const digits =
			sourceCurrencies.find((currency) => currency.code === nextCurrencyCode)?.decimal_digits ?? 2;
		form.setFieldValue(
			'rentAmount',
			unit?.market_rent_minor == null || unit.currency_code !== nextCurrencyCode
				? ''
				: (unit.market_rent_minor / 10 ** digits).toFixed(digits)
		);
	}

	function changeContractCurrency(currencyCode: string) {
		if (!enabledCurrencies.some((currency) => currency.code === currencyCode)) return;
		if (currencyCode === form.state.values.currencyCode) return;
		currencyManuallySelected = true;
		form.setFieldValue('currencyCode', currencyCode);
		form.setFieldValue('depositAmount', '');
		form.setFieldValue(
			'rentAmount',
			selectedUnit?.currency_code === currencyCode && selectedUnit.market_rent_minor != null
				? (selectedUnit.market_rent_minor / 10 ** currencyDigits(currencyCode)).toFixed(
						currencyDigits(currencyCode)
					)
				: ''
		);
	}

	function amountPlaceholder(currencyCode: string) {
		const digits = currencyDigits(currencyCode);
		return digits === 0 ? '0' : `0.${'0'.repeat(digits)}`;
	}

	function selectProperty(propertyId: string) {
		form.setFieldValue('propertyId', propertyId);
		const firstUnit = units.find(
			(unit) => unit.property_id === propertyId && unit.status !== 'inactive'
		);
		form.setFieldValue('unitId', firstUnit?.id ?? '');
		updateAmountsForUnit(firstUnit?.id ?? '');
	}

	async function saveContract(input: ContractFormValues) {
		preparing = true;
		try {
			const rentAmountMinor = parseContractAmount(input.rentAmount, input.currencyCode);
			const depositAmountMinor = input.depositAmount
				? parseContractAmount(input.depositAmount, input.currencyCode)
				: 0;
			if (rentAmountMinor === null || depositAmountMinor === null) {
				toast.error(m.contracts_amount_invalid());
				return;
			}
			const organizationId = app.currentUser?.organization_id;
			if (!organizationId) throw new Error('The active organization is unavailable.');
			const saved = await desktopApi.createContractDocument({
				template_id: input.templateId,
				tenant_id: input.tenantId,
				property_id: input.propertyId,
				unit_id: input.unitId,
				start_date: input.startDate,
				end_date: input.endDate,
				currency_code: input.currencyCode,
				rent_amount_minor: rentAmountMinor,
				deposit_amount_minor: depositAmountMinor
			});
			await queryClient.invalidateQueries({
				queryKey: contractDocumentQueryKeys.pages(organizationId, propertyId)
			});
			await queryClient.invalidateQueries({
				queryKey: contractDocumentQueryKeys.property(organizationId, input.propertyId)
			});
			selectedContractForPreview = saved;
			builderOpen = false;
			toast.success(m.contracts_saved());
		} finally {
			preparing = false;
		}
	}
</script>

<svelte:head>
	<title>{m.contracts_builder_title()} | AqarDesk</title>
</svelte:head>

<div class="flex min-h-0 min-w-0 flex-1 flex-col">
	{#if !canViewContracts}
		<Card class="border-destructive/40">
			<CardContent class="p-5 text-sm text-muted-foreground">{m.contracts_permission_missing()}</CardContent>
		</Card>
	{:else if loading}
		<Card role="status" aria-live="polite">
			<CardContent class="p-6 text-sm text-muted-foreground">{m.common_loading()}</CardContent>
		</Card>
	{:else if loadError}
		<Card class="border-destructive/40">
			<CardContent class="flex flex-wrap items-center justify-between gap-4 p-5">
				<p class="min-w-0 text-sm text-destructive" role="alert">{loadError}</p>
				<Button variant="outline" onclick={() => void load()}>{m.common_retry()}</Button>
			</CardContent>
		</Card>
	{:else}
		<ContractDocumentsDirectory
			contracts={contractDocuments}
			loading={contractDocumentsQuery.isLoading}
			hasMore={contractDocumentsQuery.hasNextPage}
			onloadmore={() => fetchNextPageOrThrow(() => contractDocumentsQuery.fetchNextPage())}
			selectedContract={selectedContractForPreview}
			canCreate={canCreateContracts}
			createDisabled={!canStartContract}
			onCreate={() => (builderOpen = true)}
		/>
		{#if canCreateContracts && activeTemplates.length === 0}
			<div class="rounded-xl border border-dashed border-border p-5 text-center text-sm text-muted-foreground">
				{m.contracts_no_templates()}
			</div>
		{:else if canCreateContracts && tenants.length === 0}
			<div class="rounded-xl border border-dashed border-border p-5 text-center text-sm text-muted-foreground">
				{m.contracts_no_tenants()}
			</div>
		{:else if canCreateContracts && activeProperties.length === 0}
			<div class="rounded-xl border border-dashed border-border p-5 text-center text-sm text-muted-foreground">
				{m.contracts_no_units()}
			</div>
		{:else if canCreateContracts && enabledCurrencies.length === 0}
			<div class="rounded-xl border border-dashed border-border p-5 text-center text-sm text-muted-foreground" role="status">
				{m.contracts_no_currencies()}
			</div>
		{/if}
		{#if canCreateContracts && activeTemplates.length > 0 && tenants.length > 0 && activeProperties.length > 0 && enabledCurrencies.length > 0}
		<Sheet bind:open={builderOpen}>
			<SheetContent side="end" class="max-w-2xl">
				<SheetHeader class="shrink-0 px-4 py-4">
					<SheetTitle>{m.contracts_new_contract()}</SheetTitle>
				</SheetHeader>
				<form
					class="flex min-h-0 flex-1 flex-col"
					onsubmit={(event) => {
						event.preventDefault();
						event.stopPropagation();
						void form.handleSubmit();
					}}
				>
					<div class="grid min-h-0 flex-1 content-start gap-3 overflow-y-auto p-4 scrollbar-none sm:p-5">
						<div class="grid gap-3 sm:grid-cols-2">
							<form.Field name="templateId">
								{#snippet children(field)}
									<FormField
										label={m.contracts_template()}
										id="contract-template"
										error={getFieldError(field.state.meta.errors)}
									>
										<Select
											type="single"
											value={field.state.value}
											items={activeTemplates.map((template) => ({
												value: template.id,
												label: template.name
											}))}
											onValueChange={(value) => {
												if (typeof value === 'string') field.handleChange(value);
											}}
										>
											<SelectTrigger
												id="contract-template"
												aria-invalid={field.state.meta.errors.length > 0}
												disabled={isReadonly}
											>
												<span class="min-w-0 flex-1 truncate">
													{selectedTemplateName ?? m.contracts_template()}
												</span>
											</SelectTrigger>
											<SelectContent>
												{#each activeTemplates as template (template.id)}
													<SelectItem value={template.id} label={template.name}>
														{template.name}
													</SelectItem>
												{/each}
											</SelectContent>
										</Select>
									</FormField>
								{/snippet}
							</form.Field>
							<form.Field name="tenantId">
								{#snippet children(field)}
									<FormField
										label={m.payments_tenant()}
										id="contract-tenant"
										error={getFieldError(field.state.meta.errors)}
									>
										<Select
											type="single"
											value={field.state.value}
											items={tenants.map((tenant) => ({
												value: tenant.id,
												label: tenant.full_name
											}))}
											onValueChange={(value) => {
												if (typeof value === 'string') field.handleChange(value);
											}}
										>
											<SelectTrigger
												id="contract-tenant"
												aria-invalid={field.state.meta.errors.length > 0}
												disabled={isReadonly}
											>
												<span class="min-w-0 flex-1 truncate">
													{selectedTenantName ?? m.payments_tenant()}
												</span>
											</SelectTrigger>
											<SelectContent>
												{#each tenants as tenant (tenant.id)}
													<SelectItem value={tenant.id} label={tenant.full_name}>
														{tenant.full_name}
													</SelectItem>
												{/each}
											</SelectContent>
										</Select>
									</FormField>
								{/snippet}
							</form.Field>
						</div>
						<div class="grid gap-3 sm:grid-cols-2">
							<form.Field name="propertyId">
								{#snippet children(field)}
									<FormField
										label={m.contracts_select_property()}
										id="contract-property"
										error={getFieldError(field.state.meta.errors)}
									>
										<Select
											type="single"
											value={field.state.value}
											items={activeProperties.map((property) => ({
												value: property.id,
												label: property.name
											}))}
											onValueChange={(value) => {
												if (typeof value === 'string') {
													field.handleChange(value);
													selectProperty(value);
												}
											}}
										>
											<SelectTrigger
												id="contract-property"
												aria-invalid={field.state.meta.errors.length > 0}
												disabled={isReadonly || !!propertyId}
											>
												<span class="min-w-0 flex-1 truncate">
													{selectedPropertyName ?? m.contracts_select_property()}
												</span>
											</SelectTrigger>
											<SelectContent>
												{#each activeProperties as property (property.id)}
													<SelectItem value={property.id} label={property.name}>
														{property.name}
													</SelectItem>
												{/each}
											</SelectContent>
										</Select>
									</FormField>
								{/snippet}
							</form.Field>
							<form.Field name="unitId">
								{#snippet children(field)}
									<FormField
										label={m.contracts_select_unit()}
										id="contract-unit"
										error={getFieldError(field.state.meta.errors)}
									>
										<Select
											type="single"
											value={field.state.value}
											items={propertyUnits.map((unit) => ({
												value: unit.id,
												label: `${unit.unit_number} · ${unitTypeLabel(unit.unit_type)}`
											}))}
											onValueChange={(value) => {
												if (typeof value === 'string') {
													field.handleChange(value);
													updateAmountsForUnit(value);
												}
											}}
										>
											<SelectTrigger
												id="contract-unit"
												aria-invalid={field.state.meta.errors.length > 0}
												disabled={isReadonly || propertyUnits.length === 0}
											>
												<span class="min-w-0 flex-1 truncate">
													{selectedUnitName ?? m.contracts_select_unit()}
												</span>
											</SelectTrigger>
											<SelectContent>
												{#each propertyUnits as unit (unit.id)}
													<SelectItem
														value={unit.id}
														label={`${unit.unit_number} · ${unitTypeLabel(unit.unit_type)}`}
													>
														{unit.unit_number} · {unitTypeLabel(unit.unit_type)}
													</SelectItem>
												{/each}
											</SelectContent>
										</Select>
									</FormField>
								{/snippet}
							</form.Field>
						</div>
						<div class="grid gap-3 sm:grid-cols-2">
								<form.Field name="startDate">
									{#snippet children(field)}
										<FormField
											label={m.contracts_start_date()}
											id="contract-start-date"
											error={getFieldError(field.state.meta.errors)}
										>
											<Input
												id="contract-start-date"
												type="date"
												value={field.state.value}
												max={form.state.values.endDate || undefined}
												oninput={(event) => field.handleChange(event.currentTarget.value)}
												onblur={field.handleBlur}
												aria-invalid={field.state.meta.errors.length > 0}
												disabled={isReadonly}
											/>
										</FormField>
									{/snippet}
								</form.Field>
								<form.Field name="endDate">
									{#snippet children(field)}
										<FormField
											label={m.contracts_end_date()}
											id="contract-end-date"
											error={getFieldError(field.state.meta.errors)}
										>
											<Input
												id="contract-end-date"
												type="date"
												value={field.state.value}
												min={form.state.values.startDate || undefined}
												oninput={(event) => field.handleChange(event.currentTarget.value)}
												onblur={field.handleBlur}
												aria-invalid={field.state.meta.errors.length > 0}
												disabled={isReadonly}
											/>
										</FormField>
									{/snippet}
								</form.Field>
							</div>
							<FormField
								label={m.payments_currency()}
								id="contract-currency"
								description={m.contracts_currency_hint()}
							>
								<CurrencySelector
									id="contract-currency"
									currencies={enabledCurrencies}
									value={selectedCurrency}
									onChange={changeContractCurrency}
									disabled={isReadonly || enabledCurrencies.length < 2}
								/>
							</FormField>
							<div class="grid gap-3 sm:grid-cols-2">
								<form.Field name="rentAmount">
									{#snippet children(field)}
										<FormField
											label={m.contracts_rent()}
											id="contract-rent"
											error={getFieldError(field.state.meta.errors)}
										>
											<Input
												id="contract-rent"
												type="text"
												inputmode="decimal"
												dir="ltr"
												placeholder={amountPlaceholder(selectedCurrency)}
												value={field.state.value}
												oninput={(event) => field.handleChange(event.currentTarget.value)}
												onblur={field.handleBlur}
												aria-invalid={field.state.meta.errors.length > 0}
												disabled={isReadonly}
											/>
										</FormField>
									{/snippet}
								</form.Field>
								<form.Field name="depositAmount">
									{#snippet children(field)}
										<FormField
											label={m.contracts_deposit()}
											id="contract-deposit"
											error={getFieldError(field.state.meta.errors)}
										>
											<Input
												id="contract-deposit"
												type="text"
												inputmode="decimal"
												dir="ltr"
												placeholder={amountPlaceholder(selectedCurrency)}
												value={field.state.value}
												oninput={(event) => field.handleChange(event.currentTarget.value)}
												onblur={field.handleBlur}
												aria-invalid={field.state.meta.errors.length > 0}
												disabled={isReadonly}
											/>
										</FormField>
									{/snippet}
								</form.Field>
							</div>
					</div>
					<SheetFooter class="shrink-0 border-t border-border bg-popover px-4 py-3">
						<Button
							type="button"
							variant="outline"
							onclick={() => (builderOpen = false)}
							disabled={preparing}
						>
							{m.common_cancel()}
						</Button>
						<Button type="submit" disabled={isReadonly || preparing || !selectedCurrency}>
							{preparing ? m.common_processing() : m.contracts_generate()}
						</Button>
					</SheetFooter>
				</form>
			</SheetContent>
		</Sheet>
		{/if}
	{/if}
</div>
