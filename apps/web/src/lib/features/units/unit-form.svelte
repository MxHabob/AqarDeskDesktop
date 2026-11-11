<script lang="ts">
	import { createForm } from '@tanstack/svelte-form';
	import { z } from 'zod';
	import { untrack } from 'svelte';
	import type { Property, PropertyUnit, PropertyUnitInput, UnitType } from '@aqardesk/sdk';
	import {
		Button,
		Input,
		Select,
		SelectContent,
		SelectItem,
		SelectTrigger,
		SelectValue,
		SheetFooter
	} from '@aqardesk/ui';
	import FormField from '#lib/shared/components/form-field.svelte';
	import { getFieldError } from '#lib/shared/form-errors';
	import { currencyDecimalDigits, formatMinorAmount, parseMajorAmount } from '#lib/shared/currency';
	import * as m from '#lib/paraglide/messages';

	type Props = {
		initial?: PropertyUnit;
		properties: Property[];
		busy: boolean;
		onSubmit: (values: PropertyUnitInput) => Promise<boolean>;
		onCancel: () => void;
	};
	type Values = Omit<PropertyUnitInput, 'market_rent_minor' | 'currency_code'> & {
		currency_code: string;
		market_rent: string;
	};
	let { initial, properties, busy, onSubmit, onCancel }: Props = $props();
	const defaultValues: Values = untrack(() => initial
		? {
				property_id: initial.property_id,
				unit_number: initial.unit_number,
				unit_type: initial.unit_type,
				status: initial.status,
				bedrooms: initial.bedrooms,
				bathrooms: initial.bathrooms,
				area_square_meters: initial.area_square_meters ?? null,
				floor_number: initial.floor_number ?? null,
				currency_code: initial.currency_code ?? 'SAR',
				market_rent: formatMinorAmount(initial.market_rent_minor ?? null, initial.currency_code ?? 'SAR')
			}
		: {
				property_id: properties[0]?.id ?? '',
				unit_number: '',
				unit_type: 'apartment',
				status: 'available',
				bedrooms: 0,
				bathrooms: 0,
				area_square_meters: null,
				floor_number: null,
				currency_code: 'SAR',
				market_rent: ''
			});
	const schema = z.object({
		property_id: z.string().min(1, m.units_property()),
		unit_number: z.string().trim().min(1, m.units_number()),
		unit_type: z.enum(['apartment', 'office', 'retail', 'warehouse', 'parking', 'other']),
		status: z.enum(['available', 'maintenance', 'inactive']),
		bedrooms: z.number().int().min(0),
		bathrooms: z.number().min(0),
		area_square_meters: z.number().positive().nullable(),
		floor_number: z.number().int().nullable(),
		currency_code: z.string().trim().regex(/^[A-Za-z]{3}$/).transform((value) => value.toUpperCase()),
		market_rent: z.string().trim().refine(
			(value) =>
				value === '' ||
				(/^\d+(?:\.\d+)?$/.test(value) && Number.isFinite(Number(value)) && Number(value) >= 0),
			m.units_rent_invalid()
		)
	}).refine(
		(value) =>
			value.market_rent === '' ||
			((value.market_rent.split('.')[1]?.length ?? 0) <= currencyDecimalDigits(value.currency_code) &&
				parseMajorAmount(value.market_rent, value.currency_code) !== null),
		{ message: m.units_rent_invalid(), path: ['market_rent'] }
	);
	const form = createForm(() => ({
		defaultValues,
		validators: { onSubmit: schema },
		onSubmit: async ({ value }) => {
			const { market_rent, ...unit } = value;
			const marketRentMinor = parseMajorAmount(market_rent, value.currency_code);
			const ok = await onSubmit({
				...unit,
				currency_code: marketRentMinor === null ? null : value.currency_code,
				market_rent_minor: marketRentMinor
			});
			if (ok) onCancel();
		}
	}));
	function setType(value: string, change: (value: UnitType) => void) {
		if (['apartment', 'office', 'retail', 'warehouse', 'parking', 'other'].includes(value)) change(value as UnitType);
	}
	function setStatus(value: string, change: (value: Values['status']) => void) {
		if (value === 'available' || value === 'maintenance' || value === 'inactive') change(value);
	}
	function submit(event: SubmitEvent) {
		event.preventDefault();
		event.stopPropagation();
		void form.handleSubmit();
	}
</script>

<form id="unit-form" class="flex min-h-0 flex-1 flex-col" onsubmit={submit}>
	<div class="grid min-h-0 flex-1 content-start gap-3 overflow-y-auto p-4 scrollbar-none">
	<form.Field name="property_id">
		{#snippet children(field)}
			<FormField label={m.units_property()} id="unit-property" error={getFieldError(field.state.meta.errors)}>
				<Select type="single" value={field.state.value} onValueChange={field.handleChange}>
					<SelectTrigger id="unit-property" class="h-9 w-full rounded-md text-sm"><SelectValue /></SelectTrigger>
					<SelectContent>
						{#each properties as property (property.id)}
							<SelectItem value={property.id}>{property.name} ({property.code})</SelectItem>
						{/each}
					</SelectContent>
				</Select>
			</FormField>
		{/snippet}
	</form.Field>
	<div class="grid gap-4 sm:grid-cols-2">
		<form.Field name="unit_number">
			{#snippet children(field)}
				<FormField label={m.units_number()} id="unit-number" error={getFieldError(field.state.meta.errors)}>
					<Input id="unit-number" value={field.state.value} oninput={(event) => field.handleChange(event.currentTarget.value)} onblur={field.handleBlur} />
				</FormField>
			{/snippet}
		</form.Field>
		<form.Field name="unit_type">
			{#snippet children(field)}
				<FormField label={m.units_type()} id="unit-type">
					<Select type="single" value={field.state.value} onValueChange={(value) => setType(value, field.handleChange)}>
						<SelectTrigger id="unit-type" class="h-9 w-full rounded-md text-sm"><SelectValue /></SelectTrigger>
						<SelectContent>
							<SelectItem value="apartment">{m.units_type_apartment()}</SelectItem>
							<SelectItem value="office">{m.units_type_office()}</SelectItem>
							<SelectItem value="retail">{m.units_type_retail()}</SelectItem>
							<SelectItem value="warehouse">{m.units_type_warehouse()}</SelectItem>
							<SelectItem value="parking">{m.units_type_parking()}</SelectItem>
							<SelectItem value="other">{m.common_other()}</SelectItem>
						</SelectContent>
					</Select>
				</FormField>
			{/snippet}
		</form.Field>
	</div>
	<form.Field name="status">
		{#snippet children(field)}
			<FormField label={m.units_status()} id="unit-status">
				<Select type="single" value={field.state.value} onValueChange={(value) => setStatus(value, field.handleChange)}>
					<SelectTrigger id="unit-status" class="h-9 w-full rounded-md text-sm"><SelectValue /></SelectTrigger>
					<SelectContent>
						<SelectItem value="available">{m.units_status_available()}</SelectItem>
						<SelectItem value="maintenance">{m.units_status_maintenance()}</SelectItem>
						<SelectItem value="inactive">{m.common_disabled()}</SelectItem>
					</SelectContent>
				</Select>
			</FormField>
		{/snippet}
	</form.Field>
	<div class="grid gap-4 sm:grid-cols-2">
		<form.Field name="bedrooms">
			{#snippet children(field)}
				<FormField label={m.units_bedrooms()} id="unit-bedrooms">
					<Input id="unit-bedrooms" type="number" min="0" value={field.state.value} oninput={(event) => field.handleChange(Number(event.currentTarget.value))} />
				</FormField>
			{/snippet}
		</form.Field>
		<form.Field name="bathrooms">
			{#snippet children(field)}
				<FormField label={m.units_bathrooms()} id="unit-bathrooms">
					<Input id="unit-bathrooms" type="number" min="0" step="0.5" value={field.state.value} oninput={(event) => field.handleChange(Number(event.currentTarget.value))} />
				</FormField>
			{/snippet}
		</form.Field>
		<form.Field name="area_square_meters">
			{#snippet children(field)}
				<FormField label={m.units_area()} id="unit-area" error={getFieldError(field.state.meta.errors)}>
					<Input id="unit-area" type="number" min="0" step="0.01" value={field.state.value ?? ''} oninput={(event) => field.handleChange(event.currentTarget.value === '' ? null : Number(event.currentTarget.value))} />
				</FormField>
			{/snippet}
		</form.Field>
		<form.Field name="floor_number">
			{#snippet children(field)}
				<FormField label={m.units_floor()} id="unit-floor">
					<Input id="unit-floor" type="number" value={field.state.value ?? ''} oninput={(event) => field.handleChange(event.currentTarget.value === '' ? null : Number(event.currentTarget.value))} />
				</FormField>
			{/snippet}
		</form.Field>
		<form.Field name="currency_code">
			{#snippet children(field)}
				<FormField label={m.units_currency()} id="unit-currency">
					<Input id="unit-currency" value={field.state.value ?? ''} oninput={(event) => field.handleChange(event.currentTarget.value.toUpperCase())} dir="ltr" maxlength={3} />
				</FormField>
			{/snippet}
		</form.Field>
		<form.Field name="market_rent">
			{#snippet children(field)}
				<FormField label={m.units_rent()} id="unit-rent">
					<Input id="unit-rent" type="number" min="0" step={1 / 10 ** currencyDecimalDigits(form.state.values.currency_code)} value={field.state.value} oninput={(event) => field.handleChange(event.currentTarget.value)} dir="ltr" />
				</FormField>
			{/snippet}
		</form.Field>
	</div>
	</div>
</form>
<SheetFooter class="shrink-0 border-t border-border bg-popover px-4 py-3">
	<Button type="button" variant="outline" disabled={busy} onclick={onCancel}>{m.common_cancel()}</Button>
	<Button type="submit" form="unit-form" disabled={busy || properties.length === 0}>{busy ? m.common_processing() : initial ? m.common_save() : m.units_add()}</Button>
</SheetFooter>
