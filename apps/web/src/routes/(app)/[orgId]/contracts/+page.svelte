<script lang="ts">
	import { createForm } from '@tanstack/svelte-form';
	import { onMount } from 'svelte';
	import {
		Button,
		Card,
		CardContent,
		Input,
		Select,
		SelectContent,
		SelectItem,
		SelectTrigger,
		toast
	} from '@aqardesk/ui';
	import { Printer } from '@lucide/svelte';
	import { z } from 'zod';
	import type {
		ContractTemplate,
		CurrencySettings,
		Property,
		PropertyUnit,
		Tenant
	} from '@aqardesk/sdk';
	import { desktopApi } from '#lib/api/desktop';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { getLocale } from '#lib/paraglide/runtime';
	import FormField from '#lib/shared/components/form-field.svelte';
	import { getFieldError } from '#lib/shared/form-errors';
	import * as m from '#lib/paraglide/messages';

	type DocumentBlock = { kind: 'heading' | 'paragraph' | 'list-item'; text: string };
	type ContractFormValues = {
		templateId: string;
		tenantId: string;
		propertyId: string;
		unitId: string;
		startDate: string;
		endDate: string;
		rentAmount: string;
		depositAmount: string;
	};

	const app = getAppContext();
	const locale = $derived(getLocale());
	let templates = $state<ContractTemplate[]>([]);
	let properties = $state<Property[]>([]);
	let units = $state<PropertyUnit[]>([]);
	let tenants = $state<Tenant[]>([]);
	let currencies = $state<CurrencySettings[]>([]);
	let loading = $state(true);
	let loadError = $state('');
	let preparing = $state(false);
	let generatedDocument = $state('');
	let generatedTitle = $state('');
	let generatedNumber = $state('');
	let generatedDirection = $state<'rtl' | 'ltr'>('rtl');

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
				property.status === 'active' &&
				units.some((unit) => unit.property_id === property.id && unit.status !== 'inactive')
		)
	);

	function currencyDigits(currencyCode: string) {
		return currencies.find((currency) => currency.code === currencyCode)?.decimal_digits ?? 2;
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

	function money(value: string, currencyCode: string) {
		const amount = Number(value);
		if (!Number.isFinite(amount) || !currencyCode) return '';
		return new Intl.NumberFormat(locale, {
			style: 'currency',
			currency: currencyCode,
			minimumFractionDigits: currencyDigits(currencyCode),
			maximumFractionDigits: currencyDigits(currencyCode)
		}).format(amount);
	}

	function isValidAmount(value: string, currencyCode: string, required: boolean) {
		if (!value) return !required;
		if (!/^\d+(?:\.\d+)?$/.test(value)) return false;
		const fraction = value.split('.')[1] ?? '';
		return fraction.length <= currencyDigits(currencyCode) && Number.isFinite(Number(value));
	}

	const contractSchema = z.object({
		templateId: z.string().min(1, m.contracts_no_templates()),
		tenantId: z.string().min(1, m.contracts_no_tenants()),
		propertyId: z.string().min(1, m.contracts_no_units()),
		unitId: z.string().min(1, m.contracts_no_units()),
		startDate: z.string().min(1, m.contracts_invalid_dates()).refine((value) => !value || isValidDate(value), m.contracts_invalid_dates()),
		endDate: z.string().min(1, m.contracts_invalid_dates()).refine((value) => !value || isValidDate(value), m.contracts_invalid_dates()),
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
		const currencyCode =
			units.find((unit) => unit.id === values.unitId)?.currency_code ??
			currencies.find((currency) => currency.is_enabled_for_organization)?.code ??
			'SAR';
		if (!isValidAmount(values.rentAmount, currencyCode, true) || Number(values.rentAmount) < 0) {
			context.addIssue({
				code: 'custom',
				path: ['rentAmount'],
				message: m.contracts_amount_invalid()
			});
		}
		if (!isValidAmount(values.depositAmount, currencyCode, false) || Number(values.depositAmount || 0) < 0) {
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
			rentAmount: '',
			depositAmount: ''
		} satisfies ContractFormValues,
		validators: { onSubmit: contractSchema },
		onSubmit: async ({ value }) => {
			preparing = true;
			try {
				prepareDocument(value);
			} finally {
				preparing = false;
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
	const selectedUnitName = $derived(
		selectedUnit ? `${selectedUnit.unit_number} · ${selectedUnit.unit_type}` : undefined
	);
	const selectedCurrency = $derived(
		selectedUnit?.currency_code ??
			currencies.find((currency) => currency.is_enabled_for_organization)?.code ??
			'SAR'
	);
	const isReadonly = $derived(app.currentUser?.role === 'readonly');

	$effect(() => {
		form.state.values.templateId;
		form.state.values.tenantId;
		form.state.values.propertyId;
		form.state.values.unitId;
		form.state.values.startDate;
		form.state.values.endDate;
		form.state.values.rentAmount;
		form.state.values.depositAmount;
		generatedDocument = '';
	});

	onMount(() => {
		void load();
	});

	async function load() {
		loading = true;
		loadError = '';
		try {
			const [nextTemplates, nextProperties, nextUnits, nextTenants, nextCurrencies] =
				await Promise.all([
					desktopApi.listContractTemplates(),
					desktopApi.listProperties(),
					desktopApi.listUnits(),
					desktopApi.listTenants(),
					desktopApi.listCurrencies()
				]);
			templates = nextTemplates;
			properties = nextProperties;
			units = nextUnits;
			tenants = nextTenants;
			currencies = nextCurrencies;

			const firstProperty = nextProperties.find(
				(property) =>
					property.status === 'active' &&
					nextUnits.some((unit) => unit.property_id === property.id && unit.status !== 'inactive')
			);
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
			form.setFieldValue('tenantId', nextTenants[0]?.id ?? '');
			form.setFieldValue('propertyId', firstProperty?.id ?? '');
			form.setFieldValue('unitId', firstUnit?.id ?? '');
			form.setFieldValue('startDate', localDateValue());
			form.setFieldValue('rentAmount', '');
			form.setFieldValue('depositAmount', '');
			form.setFieldValue('endDate', '');
			updateAmountsForUnit(firstUnit?.id ?? '', nextUnits, nextCurrencies);
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
		const digits = sourceCurrencies.find((currency) => currency.code === unit?.currency_code)?.decimal_digits ?? 2;
		form.setFieldValue(
			'rentAmount',
			unit?.market_rent_minor == null
				? ''
				: (unit.market_rent_minor / 10 ** digits).toFixed(digits)
		);
	}

	function selectProperty(propertyId: string) {
		form.setFieldValue('propertyId', propertyId);
		const firstUnit = units.find(
			(unit) => unit.property_id === propertyId && unit.status !== 'inactive'
		);
		form.setFieldValue('unitId', firstUnit?.id ?? '');
		updateAmountsForUnit(firstUnit?.id ?? '');
	}

	function prepareDocument(input: ContractFormValues) {
		const template = activeTemplates.find((item) => item.id === input.templateId);
		const tenant = tenants.find((item) => item.id === input.tenantId);
		const property = properties.find((item) => item.id === input.propertyId);
		const unit = units.find(
			(item) =>
				item.id === input.unitId &&
				item.property_id === input.propertyId &&
				item.status !== 'inactive'
		);
		if (!template || !tenant || !property || !unit) {
			toast.error(m.contracts_no_units());
			return;
		}
		const currencyCode = unit.currency_code ?? selectedCurrency;

		const address = [property.address_line_1, property.address_line_2, property.city, property.region]
			.filter(Boolean)
			.join('، ');
		const today = localDateValue();
		const number = `AQD-${today.replaceAll('-', '')}-${crypto.randomUUID().slice(0, 8).toUpperCase()}`;
		const values: Record<string, string> = {
			'tenant.full_name': tenant.full_name,
			'tenant.email': tenant.email ?? '—',
			'tenant.phone': tenant.phone ?? '—',
			'tenant.address': tenant.mailing_address ?? '—',
			'property.name': property.name,
			'property.code': property.code,
			'property.address': address || '—',
			'property.city': property.city || '—',
			'unit.number': unit.unit_number,
			'unit.type': unit.unit_type,
			'unit.floor': unit.floor_number == null ? '—' : String(unit.floor_number),
			'unit.area': unit.area_square_meters == null ? '—' : `${unit.area_square_meters} م²`,
			'unit.bedrooms': String(unit.bedrooms),
			'unit.bathrooms': String(unit.bathrooms),
			'unit.rent': money(input.rentAmount, currencyCode),
			'unit.currency': currencyCode,
			'contract.number': number,
			'contract.date': today,
			'contract.start_date': input.startDate,
			'contract.end_date': input.endDate,
			'contract.deposit': input.depositAmount ? money(input.depositAmount, currencyCode) : '—',
			'organization.name': app.organizationName || '—'
		};
		const content = template.versions[0]?.content_body ?? '';
		const unresolved = new Set<string>();
		const resolved = content.replace(/\{\{\s*([a-zA-Z0-9_.-]+)\s*\}\}/g, (token, key: string) => {
			if (!(key in values)) {
				unresolved.add(key);
				return token;
			}
			return values[key];
		});
		if (unresolved.size) {
			toast.error(m.contracts_missing_variables({ variables: [...unresolved].join(', ') }));
			return;
		}
		generatedDocument = resolved;
		generatedTitle = template.name;
		generatedNumber = number;
		generatedDirection = template.locale.toLowerCase().startsWith('ar') ? 'rtl' : 'ltr';
	}

	function documentBlocks(content: string): DocumentBlock[] {
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
		if (!generatedDocument) return;
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

<svelte:head>
	<title>{m.contracts_builder_title()} | AqarDesk</title>
</svelte:head>

<div class="grid min-h-0 min-w-0 flex-1 content-start gap-5 overflow-y-auto p-4 lg:p-6">
	{#if loading}
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
	{:else if activeTemplates.length === 0}
		<div class="rounded-xl border border-dashed border-border p-8 text-center text-sm text-muted-foreground">
			{m.contracts_no_templates()}
		</div>
	{:else if tenants.length === 0}
		<div class="rounded-xl border border-dashed border-border p-8 text-center text-sm text-muted-foreground">
			{m.contracts_no_tenants()}
		</div>
	{:else if activeProperties.length === 0}
		<div class="rounded-xl border border-dashed border-border p-8 text-center text-sm text-muted-foreground">
			{m.contracts_no_units()}
		</div>
	{:else}
		<form
			class="grid min-w-0 gap-5"
			onsubmit={(event) => {
				event.preventDefault();
				event.stopPropagation();
				void form.handleSubmit();
			}}
		>
			<Card>
				<CardContent class="grid gap-x-4 gap-y-5 sm:grid-cols-2 xl:grid-cols-4">
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
											<SelectItem value={template.id} label={template.name}>{template.name}</SelectItem>
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
											<SelectItem value={tenant.id} label={tenant.full_name}>{tenant.full_name}</SelectItem>
										{/each}
									</SelectContent>
								</Select>
							</FormField>
						{/snippet}
					</form.Field>

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
										disabled={isReadonly}
									>
										<span class="min-w-0 flex-1 truncate">
											{selectedPropertyName ?? m.contracts_select_property()}
										</span>
									</SelectTrigger>
									<SelectContent>
										{#each activeProperties as property (property.id)}
											<SelectItem value={property.id} label={property.name}>{property.name}</SelectItem>
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
												label={`${unit.unit_number} · ${unit.unit_type}`}
											>
												{unit.unit_number} · {unit.unit_type}
											</SelectItem>
										{/each}
									</SelectContent>
								</Select>
							</FormField>
						{/snippet}
					</form.Field>

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

					<form.Field name="rentAmount">
						{#snippet children(field)}
							<FormField
								label={`${m.contracts_rent()} (${selectedCurrency})`}
								id="contract-rent"
								error={getFieldError(field.state.meta.errors)}
							>
								<Input
									id="contract-rent"
									type="number"
									inputmode="decimal"
									dir="ltr"
									min="0"
									step={1 / 10 ** currencyDigits(selectedCurrency)}
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
								label={`${m.contracts_deposit()} (${selectedCurrency})`}
								id="contract-deposit"
								error={getFieldError(field.state.meta.errors)}
							>
								<Input
									id="contract-deposit"
									type="number"
									inputmode="decimal"
									dir="ltr"
									min="0"
									step={1 / 10 ** currencyDigits(selectedCurrency)}
									value={field.state.value}
									oninput={(event) => field.handleChange(event.currentTarget.value)}
									onblur={field.handleBlur}
									aria-invalid={field.state.meta.errors.length > 0}
									disabled={isReadonly}
								/>
							</FormField>
						{/snippet}
					</form.Field>
				</CardContent>
				<div class="flex justify-end border-t border-border px-6 py-4">
					<Button type="submit" disabled={isReadonly || preparing}>
						{preparing ? m.common_processing() : m.contracts_generate()}
					</Button>
				</div>
			</Card>
		</form>

		{#if generatedDocument}
			<section class="grid gap-3">
				<div class="flex flex-wrap items-center justify-between gap-3">
					<div>
						<h2 class="text-base font-semibold">{m.contracts_preview()}</h2>
						<p class="text-xs text-muted-foreground">{m.contracts_disclaimer()}</p>
					</div>
					<Button variant="outline" onclick={printDocument}>
						<Printer size={15} class="me-2" />
						{m.contracts_print()}
					</Button>
				</div>
				<article
					id="issued-contract"
					dir={generatedDirection}
					class="mx-auto min-h-[29.7cm] w-full max-w-[21cm] rounded-md border border-border/70 bg-background px-8 py-10 shadow-sm sm:px-14 sm:py-16"
				>
					<header class="mb-8 border-b-2 border-primary/60 pb-5">
						<p class="text-[10px] font-bold uppercase tracking-[0.16em] text-primary">AqarDesk</p>
						<h3 class="mt-3 text-2xl font-semibold leading-relaxed">{generatedTitle}</h3>
						<p class="mt-2 text-xs text-muted-foreground" dir="ltr">{generatedNumber}</p>
					</header>
					<div class="grid gap-3 text-sm leading-8">
						{#each documentBlocks(generatedDocument) as block, index (`${block.kind}-${index}`)}
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
			</section>
		{/if}
	{/if}
</div>
