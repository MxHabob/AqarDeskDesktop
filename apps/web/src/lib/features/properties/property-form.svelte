<script lang="ts">
	import { createForm } from '@tanstack/svelte-form';
	import { z } from 'zod';
	import { untrack } from 'svelte';
	import type { Property, PropertyInput, PropertyType } from '@aqardesk/sdk';
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
	import * as m from '#lib/paraglide/messages';

	type Props = {
		initial?: Property;
		busy: boolean;
		onSubmit: (values: PropertyInput) => Promise<boolean>;
		onCancel: () => void;
	};

	let { initial, busy, onSubmit, onCancel }: Props = $props();
	type Values = PropertyInput;
	const defaultValues: Values = untrack(() => initial
		? {
				name: initial.name,
				code: initial.code,
				property_type: initial.property_type,
				status: initial.status === 'archived' ? 'inactive' : initial.status,
				address_line_1: initial.address_line_1,
				address_line_2: initial.address_line_2 ?? '',
				city: initial.city,
				region: initial.region ?? '',
				postal_code: initial.postal_code ?? '',
				country_code: initial.country_code,
				description: initial.description ?? ''
			}
		: {
				name: '',
				code: '',
				property_type: 'residential',
				status: 'active',
				address_line_1: '',
				address_line_2: '',
				city: '',
				region: '',
				postal_code: '',
				country_code: '',
				description: ''
			});
	const schema = z.object({
		name: z.string().trim().min(1, m.properties_name()),
		code: z.string().trim().min(1, m.properties_code()),
		property_type: z.enum(['residential', 'commercial', 'mixed_use', 'land', 'other']),
		status: z.enum(['active', 'inactive']),
		address_line_1: z.string().trim().min(1, m.properties_address()),
		address_line_2: z.string(),
		city: z.string().trim().min(1, m.properties_city()),
		region: z.string(),
		postal_code: z.string(),
		country_code: z.string().trim().min(2, m.properties_country()).max(3),
		description: z.string()
	});
	const form = createForm(() => ({
		defaultValues,
		validators: { onSubmit: schema },
		onSubmit: async ({ value }) => {
			const ok = await onSubmit({
				...value,
				address_line_2: value.address_line_2 || null,
				region: value.region || null,
				postal_code: value.postal_code || null,
				description: value.description || null,
				country_code: value.country_code.toUpperCase()
			});
			if (ok) onCancel();
		}
	}));

	function propertyType(value: string, change: (value: PropertyType) => void) {
		if (['residential', 'commercial', 'mixed_use', 'land', 'other'].includes(value)) {
			change(value as PropertyType);
		}
	}

	function propertyStatus(value: string, change: (value: 'active' | 'inactive') => void) {
		if (value === 'active' || value === 'inactive') change(value);
	}

	function submit(event: SubmitEvent) {
		event.preventDefault();
		event.stopPropagation();
		void form.handleSubmit();
	}
</script>

<form id="property-form" class="flex min-h-0 flex-1 flex-col" onsubmit={submit}>
	<div class="grid min-h-0 flex-1 content-start gap-3 overflow-y-auto p-4 scrollbar-none">
	<form.Field name="name">
		{#snippet children(field)}
			<FormField label={m.properties_name()} id="property-name" error={getFieldError(field.state.meta.errors)}>
				<Input id="property-name" value={field.state.value} oninput={(event) => field.handleChange(event.currentTarget.value)} onblur={field.handleBlur} aria-invalid={field.state.meta.errors.length > 0} />
			</FormField>
		{/snippet}
	</form.Field>
	<div class="grid gap-4 sm:grid-cols-2">
		<form.Field name="code">
			{#snippet children(field)}
				<FormField label={m.properties_code()} id="property-code" error={getFieldError(field.state.meta.errors)}>
					<Input id="property-code" value={field.state.value} oninput={(event) => field.handleChange(event.currentTarget.value)} onblur={field.handleBlur} dir="ltr" />
				</FormField>
			{/snippet}
		</form.Field>
		<form.Field name="property_type">
			{#snippet children(field)}
				<FormField label={m.properties_type()} id="property-type">
					<Select type="single" value={field.state.value} onValueChange={(value) => propertyType(value, field.handleChange)}>
						<SelectTrigger id="property-type" class="h-9 w-full rounded-md text-sm"><SelectValue /></SelectTrigger>
						<SelectContent>
							<SelectItem value="residential">{m.properties_type_residential()}</SelectItem>
							<SelectItem value="commercial">{m.properties_type_commercial()}</SelectItem>
							<SelectItem value="mixed_use">{m.properties_type_mixed()}</SelectItem>
							<SelectItem value="land">{m.properties_type_land()}</SelectItem>
							<SelectItem value="other">{m.common_other()}</SelectItem>
						</SelectContent>
					</Select>
				</FormField>
			{/snippet}
		</form.Field>
	</div>
	<form.Field name="status">
		{#snippet children(field)}
			<FormField label={m.properties_status()} id="property-status">
				<Select
					type="single"
					value={field.state.value}
					onValueChange={(value) => propertyStatus(value, field.handleChange)}
				>
					<SelectTrigger id="property-status" class="h-9 w-full rounded-md text-sm">
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						<SelectItem value="active">{m.common_enabled()}</SelectItem>
						<SelectItem value="inactive">{m.common_disabled()}</SelectItem>
					</SelectContent>
				</Select>
			</FormField>
		{/snippet}
	</form.Field>
	<form.Field name="address_line_1">
		{#snippet children(field)}
			<FormField label={m.properties_address()} id="property-address" error={getFieldError(field.state.meta.errors)}>
				<Input id="property-address" value={field.state.value} oninput={(event) => field.handleChange(event.currentTarget.value)} onblur={field.handleBlur} />
			</FormField>
		{/snippet}
	</form.Field>
	<form.Field name="address_line_2">
		{#snippet children(field)}
			<FormField label={m.properties_address_line_2()} id="property-address-2">
				<Input id="property-address-2" value={field.state.value ?? ''} oninput={(event) => field.handleChange(event.currentTarget.value)} />
			</FormField>
		{/snippet}
	</form.Field>
	<div class="grid gap-4 sm:grid-cols-2">
		<form.Field name="city">
			{#snippet children(field)}
				<FormField label={m.properties_city()} id="property-city" error={getFieldError(field.state.meta.errors)}>
					<Input id="property-city" value={field.state.value} oninput={(event) => field.handleChange(event.currentTarget.value)} onblur={field.handleBlur} />
				</FormField>
			{/snippet}
		</form.Field>
		<form.Field name="region">
			{#snippet children(field)}
				<FormField label={m.properties_region()} id="property-region">
					<Input id="property-region" value={field.state.value ?? ''} oninput={(event) => field.handleChange(event.currentTarget.value)} />
				</FormField>
			{/snippet}
		</form.Field>
		<form.Field name="postal_code">
			{#snippet children(field)}
				<FormField label={m.properties_postal_code()} id="property-postal">
					<Input id="property-postal" value={field.state.value ?? ''} oninput={(event) => field.handleChange(event.currentTarget.value)} dir="ltr" />
				</FormField>
			{/snippet}
		</form.Field>
		<form.Field name="country_code">
			{#snippet children(field)}
				<FormField label={m.properties_country()} id="property-country" error={getFieldError(field.state.meta.errors)}>
					<Input id="property-country" value={field.state.value} oninput={(event) => field.handleChange(event.currentTarget.value.toUpperCase())} onblur={field.handleBlur} dir="ltr" maxlength={3} />
				</FormField>
			{/snippet}
		</form.Field>
	</div>
	<form.Field name="description">
		{#snippet children(field)}
			<FormField label={m.common_description()} id="property-description">
				<textarea id="property-description" value={field.state.value ?? ''} oninput={(event) => field.handleChange(event.currentTarget.value)} rows="3" class="min-h-20 w-full rounded-md border border-input bg-card px-3 py-2 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring/40"></textarea>
			</FormField>
		{/snippet}
	</form.Field>
	</div>
</form>
<SheetFooter class="shrink-0 border-t border-border bg-popover px-4 py-3">
	<Button type="button" variant="outline" disabled={busy} onclick={onCancel}>{m.common_cancel()}</Button>
	<Button type="submit" form="property-form" disabled={busy}>{busy ? m.common_processing() : initial ? m.common_save() : m.properties_add()}</Button>
</SheetFooter>
