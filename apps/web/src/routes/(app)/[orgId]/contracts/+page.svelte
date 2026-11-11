<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from '@aqardesk/ui';
	import { Printer } from '@lucide/svelte';
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
	import * as m from '#lib/paraglide/messages';

	type DocumentBlock = { kind: 'heading' | 'paragraph' | 'list-item'; text: string };

	const app = getAppContext();
	const locale = $derived(getLocale());
	let templates = $state<ContractTemplate[]>([]);
	let properties = $state<Property[]>([]);
	let units = $state<PropertyUnit[]>([]);
	let tenants = $state<Tenant[]>([]);
	let currencies = $state<CurrencySettings[]>([]);
	let loading = $state(true);
	let selectedTemplateId = $state('');
	let selectedTenantId = $state('');
	let selectedPropertyId = $state('');
	let selectedUnitId = $state('');
	let startDate = $state(new Date().toISOString().slice(0, 10));
	let endDate = $state('');
	let rentAmount = $state('');
	let depositAmount = $state('');
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
	const selectedProperty = $derived(properties.find((item) => item.id === selectedPropertyId));
	const propertyUnits = $derived(
		units.filter((unit) => unit.property_id === selectedPropertyId && unit.status !== 'inactive')
	);
	const selectedTemplate = $derived(activeTemplates.find((item) => item.id === selectedTemplateId));
	const selectedUnit = $derived(propertyUnits.find((unit) => unit.id === selectedUnitId));
	const isReadonly = $derived(app.currentUser?.role === 'readonly');

	$effect(() => {
		selectedTemplateId;
		selectedTenantId;
		selectedPropertyId;
		selectedUnitId;
		startDate;
		endDate;
		rentAmount;
		depositAmount;
		generatedDocument = '';
	});

	onMount(() => {
		void load();
	});

	async function load() {
		loading = true;
		try {
			[templates, properties, units, tenants, currencies] = await Promise.all([
				desktopApi.listContractTemplates(),
				desktopApi.listProperties(),
				desktopApi.listUnits(),
				desktopApi.listTenants(),
				desktopApi.listCurrencies()
			]);
			selectedTemplateId = templates.find(
				(template) => template.is_active && template.versions[0]?.content_body?.trim()
			)?.id ?? '';
			selectedTenantId = tenants[0]?.id ?? '';
			selectedPropertyId = properties.find((property) => property.status === 'active')?.id ?? '';
			selectedUnitId =
				units.find((unit) => unit.property_id === selectedPropertyId && unit.status !== 'inactive')?.id ?? '';
			updateAmountsForUnit();
		} catch (cause) {
			toast.error(cause instanceof Error ? cause.message : String(cause));
		} finally {
			loading = false;
		}
	}

	function updateAmountsForUnit() {
		const unit = units.find((item) => item.id === selectedUnitId);
		const digits = currencies.find((currency) => currency.code === unit?.currency_code)?.decimal_digits ?? 2;
		rentAmount = unit?.market_rent_minor == null
			? ''
			: (unit.market_rent_minor / 10 ** digits).toFixed(digits);
	}

	function currencyDigits(currencyCode: string) {
		return currencies.find((currency) => currency.code === currencyCode)?.decimal_digits ?? 2;
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

	function selectedCurrency() {
		return selectedUnit?.currency_code ?? currencies.find((currency) => currency.is_enabled_for_organization)?.code ?? 'SAR';
	}

	function prepareDocument() {
		const template = selectedTemplate;
		const tenant = tenants.find((item) => item.id === selectedTenantId);
		const property = selectedProperty;
		const unit = selectedUnit;
		if (!template || !tenant || !property || !unit) {
			toast.error(m.contracts_no_units());
			return;
		}
		if (!startDate || !endDate || startDate > endDate) {
			toast.error(m.contracts_invalid_dates());
			return;
		}
		const currencyCode = selectedCurrency();
		if (!isValidAmount(rentAmount, currencyCode, true) || Number(rentAmount) < 0) {
			toast.error(m.contracts_amount_invalid());
			return;
		}
		if (!isValidAmount(depositAmount, currencyCode, false) || Number(depositAmount || 0) < 0) {
			toast.error(m.contracts_deposit());
			return;
		}

		const address = [property.address_line_1, property.address_line_2, property.city, property.region]
			.filter(Boolean)
			.join('، ');
		const today = new Date().toISOString().slice(0, 10);
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
			'unit.rent': money(rentAmount, currencyCode),
			'unit.currency': currencyCode,
			'contract.number': number,
			'contract.date': today,
			'contract.start_date': startDate,
			'contract.end_date': endDate,
			'contract.deposit': depositAmount ? money(depositAmount, currencyCode) : '—',
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

<div class="grid min-h-0 min-w-0 flex-1 content-start gap-5 overflow-y-auto p-4 lg:p-6">
	<header class="grid gap-1">
		<h1 class="text-xl font-semibold">{m.contracts_builder_title()}</h1>
		<p class="max-w-3xl text-sm text-muted-foreground">{m.contracts_builder_description()}</p>
	</header>

	{#if loading}
		<div class="rounded-xl border border-border/70 p-6 text-sm text-muted-foreground">{m.common_loading()}</div>
	{:else if activeTemplates.length === 0}
		<div class="rounded-xl border border-dashed border-border p-8 text-center text-sm text-muted-foreground">
			{m.contracts_no_templates()}
		</div>
	{:else if tenants.length === 0}
		<div class="rounded-xl border border-dashed border-border p-8 text-center text-sm text-muted-foreground">
			{m.contracts_no_tenants()}
		</div>
	{:else if properties.filter((property) => property.status === 'active').length === 0 || units.length === 0}
		<div class="rounded-xl border border-dashed border-border p-8 text-center text-sm text-muted-foreground">
			{m.contracts_no_units()}
		</div>
	{:else}
		<section class="grid gap-4 rounded-xl border border-border/70 bg-card p-4 sm:grid-cols-2 xl:grid-cols-4">
			<label class="grid gap-2 text-sm font-medium">
				{m.contracts_template()}
				<select bind:value={selectedTemplateId} class="h-10 rounded-md border border-input bg-background px-3 text-sm">
					{#each activeTemplates as template (template.id)}
						<option value={template.id}>{template.name}</option>
					{/each}
				</select>
			</label>
			<label class="grid gap-2 text-sm font-medium">
				{m.payments_tenant()}
				<select bind:value={selectedTenantId} class="h-10 rounded-md border border-input bg-background px-3 text-sm">
					{#each tenants as tenant (tenant.id)}
						<option value={tenant.id}>{tenant.full_name}</option>
					{/each}
				</select>
			</label>
			<label class="grid gap-2 text-sm font-medium">
				{m.contracts_select_property()}
				<select
					bind:value={selectedPropertyId}
					onchange={() => {
						selectedUnitId = propertyUnits[0]?.id ?? '';
						updateAmountsForUnit();
					}}
					class="h-10 rounded-md border border-input bg-background px-3 text-sm"
				>
					{#each properties.filter((property) => property.status === 'active') as property (property.id)}
						<option value={property.id}>{property.name}</option>
					{/each}
				</select>
			</label>
			<label class="grid gap-2 text-sm font-medium">
				{m.contracts_select_unit()}
				<select
					bind:value={selectedUnitId}
					onchange={updateAmountsForUnit}
					class="h-10 rounded-md border border-input bg-background px-3 text-sm"
				>
					{#each propertyUnits as unit (unit.id)}
						<option value={unit.id}>{unit.unit_number} · {unit.unit_type}</option>
					{/each}
				</select>
			</label>
			<label class="grid gap-2 text-sm font-medium">
				{m.contracts_start_date()}
				<input bind:value={startDate} type="date" class="h-10 rounded-md border border-input bg-background px-3 text-sm" />
			</label>
			<label class="grid gap-2 text-sm font-medium">
				{m.contracts_end_date()}
				<input bind:value={endDate} type="date" class="h-10 rounded-md border border-input bg-background px-3 text-sm" />
			</label>
			<label class="grid gap-2 text-sm font-medium">
				{m.contracts_rent()} ({selectedCurrency()})
				<input
					type="number"
					min="0"
					step="any"
					value={rentAmount}
					oninput={(event) => (rentAmount = event.currentTarget.value)}
					class="h-10 rounded-md border border-input bg-background px-3 text-sm"
				/>
			</label>
			<label class="grid gap-2 text-sm font-medium">
				{m.contracts_deposit()} ({selectedCurrency()})
				<input
					type="number"
					min="0"
					step="any"
					value={depositAmount}
					oninput={(event) => (depositAmount = event.currentTarget.value)}
					class="h-10 rounded-md border border-input bg-background px-3 text-sm"
				/>
			</label>
			<div class="flex items-end sm:col-span-2 xl:col-span-4">
				<button
					type="button"
					disabled={isReadonly || !propertyUnits.length}
					onclick={prepareDocument}
					class="inline-flex h-10 items-center justify-center rounded-md bg-primary px-4 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:pointer-events-none disabled:opacity-50"
				>
					{m.contracts_generate()}
				</button>
			</div>
		</section>

		{#if generatedDocument}
			<section class="grid gap-3">
				<div class="flex flex-wrap items-center justify-between gap-3">
					<div>
						<h2 class="text-base font-semibold">{m.contracts_preview()}</h2>
						<p class="text-xs text-muted-foreground">{m.contracts_disclaimer()}</p>
					</div>
					<button
						type="button"
						onclick={printDocument}
						class="inline-flex h-9 items-center rounded-md border border-input bg-background px-3 text-sm font-medium hover:bg-accent"
					>
						<Printer size={15} class="me-2" />
						{m.contracts_print()}
					</button>
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
