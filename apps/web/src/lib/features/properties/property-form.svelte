<script lang="ts">
	import { createForm } from '@tanstack/svelte-form';
	import { z } from 'zod';
	import { untrack } from 'svelte';
	import type { Owner, Property, PropertyInput, PropertyOwnerInput, PropertyType } from '@aqardesk/sdk';
	import { Plus, Trash2 } from '@lucide/svelte';
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
		ownerOptions: Owner[];
		busy: boolean;
		onSubmit: (values: PropertyInput) => Promise<boolean>;
		onCancel: () => void;
	};

	let { initial, ownerOptions, busy, onSubmit, onCancel }: Props = $props();
	type Values = PropertyInput;
	type OwnerDraft = {
		id?: string;
		name: string;
		phone_number: string;
		phone_label: string;
		is_whatsapp: boolean;
		ownership_share_percent: string;
	};
	let ownerDrafts = $state<OwnerDraft[]>(
		untrack(() =>
			initial?.owners.map((owner) => {
				const directoryOwner = ownerOptions.find((item) => item.id === owner.id);
				return {
					id: owner.id,
					name: directoryOwner?.name ?? owner.name,
					phone_number: directoryOwner?.phone_number ?? owner.phone_number,
					phone_label: directoryOwner?.phone_label ?? owner.phone_label ?? '',
					is_whatsapp: directoryOwner?.is_whatsapp ?? owner.is_whatsapp,
					ownership_share_percent:
						owner.ownership_share_percent === null || owner.ownership_share_percent === undefined
							? ''
							: String(owner.ownership_share_percent)
				};
			}) ?? []
		)
	);
	let ownerError = $state('');
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
				district: initial.district ?? '',
				neighborhood: initial.neighborhood ?? '',
				postal_code: initial.postal_code ?? '',
				country_code: initial.country_code,
				description: initial.description ?? '',
				lot_area_square_meters: initial.lot_area_square_meters ?? null,
				building_area_square_meters: initial.building_area_square_meters ?? null,
				frontage_meters: initial.frontage_meters ?? null,
				depth_meters: initial.depth_meters ?? null,
				latitude: initial.latitude ?? null,
				longitude: initial.longitude ?? null,
				owners: initial.owners
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
				district: '',
				neighborhood: '',
				postal_code: '',
				country_code: 'SA',
				description: '',
				lot_area_square_meters: null,
				building_area_square_meters: null,
				frontage_meters: null,
				depth_meters: null,
				latitude: null,
				longitude: null,
				owners: []
			});
	const ownerSchema = z.object({
		id: z.string().optional(),
		name: z.string().trim().min(1, m.properties_owner_name_required()),
		phone_number: z.string().trim().min(1, m.properties_owner_phone_required()),
		phone_label: z.string().optional(),
		is_whatsapp: z.boolean(),
		ownership_share_percent: z.number().min(0).max(100).nullable()
	});
	const ownersSchema = z.array(ownerSchema).superRefine((owners, context) => {
		const totalShare = owners.reduce(
			(total, owner) => total + (owner.ownership_share_percent ?? 0),
			0
		);
		if (totalShare > 100) {
			context.addIssue({
				code: 'custom',
				message: m.properties_owner_share_total(),
				path: []
			});
		}
	});
	const schema = z.object({
		name: z.string().trim().min(1, m.properties_name()),
		code: z.string(),
		property_type: z.enum([
			'residential',
			'commercial',
			'mixed_use',
			'land',
			'other',
			'building',
			'house_villa',
			'investment',
			'warehouse',
			'yard',
			'apartment'
		]),
		status: z.enum(['active', 'inactive']),
		address_line_1: z.string().trim().min(1, m.properties_address()),
		address_line_2: z.string(),
		city: z.string().trim().min(1, m.properties_city()),
		region: z.string(),
		district: z.string(),
		neighborhood: z.string(),
		postal_code: z.string(),
		country_code: z.string().trim().regex(/^[A-Za-z]{2}$/, m.properties_country()),
		description: z.string(),
		lot_area_square_meters: z.number().finite().nonnegative().nullable(),
		building_area_square_meters: z.number().finite().nonnegative().nullable(),
		frontage_meters: z.number().finite().nonnegative().nullable(),
		depth_meters: z.number().finite().nonnegative().nullable(),
		latitude: z.number().finite().min(-90).max(90).nullable(),
		longitude: z.number().finite().min(-180).max(180).nullable(),
		owners: z.array(z.custom<PropertyOwnerInput>())
	});
	const form = createForm(() => ({
		defaultValues,
		validators: { onSubmit: schema },
		onSubmit: async ({ value }) => {
			const parsedOwners = ownersSchema.safeParse(
				ownerDrafts.map((owner): PropertyOwnerInput => ({
					id: owner.id,
					name: owner.name,
					phone_number: owner.phone_number,
					phone_label: owner.phone_label || null,
					is_whatsapp: owner.is_whatsapp,
					ownership_share_percent:
						owner.ownership_share_percent.trim() === ''
							? null
							: Number(owner.ownership_share_percent)
				}))
			);
			if (!parsedOwners.success) {
				ownerError = parsedOwners.error.issues[0]?.message ?? m.properties_owner_invalid();
				return;
			}
			ownerError = '';
			const ok = await onSubmit({
				...value,
				address_line_2: value.address_line_2 || null,
				region: value.region || null,
				district: value.district || null,
				neighborhood: value.neighborhood || null,
				postal_code: value.postal_code || null,
				description: value.description || null,
				country_code: value.country_code.toUpperCase(),
				owners: parsedOwners.data
			});
			if (ok) onCancel();
		}
	}));

	function propertyType(value: string, change: (value: PropertyType) => void) {
		if (
			[
				'residential',
				'commercial',
				'mixed_use',
				'land',
				'other',
				'building',
				'house_villa',
				'investment',
				'warehouse',
				'yard',
				'apartment'
			].includes(value)
		) {
			change(value as PropertyType);
		}
	}

	function propertyStatus(value: string, change: (value: 'active' | 'inactive') => void) {
		if (value === 'active' || value === 'inactive') change(value);
	}

	function addOwner() {
		ownerDrafts = [
			...ownerDrafts,
			{ name: '', phone_number: '', phone_label: '', is_whatsapp: false, ownership_share_percent: '' }
		];
		ownerError = '';
	}

	function selectOwner(index: number, ownerId: string) {
		const share = ownerDrafts[index]?.ownership_share_percent ?? '';
		const selected = ownerOptions.find((owner) => owner.id === ownerId);
		if (!selected) {
			updateOwner(index, {
				id: undefined,
				name: '',
				phone_number: '',
				phone_label: '',
				is_whatsapp: false,
				ownership_share_percent: share
			});
			return;
		}
		updateOwner(index, {
			id: selected.id,
			name: selected.name,
			phone_number: selected.phone_number,
			phone_label: selected.phone_label ?? '',
			is_whatsapp: selected.is_whatsapp,
			ownership_share_percent: share
		});
	}

	function removeOwner(index: number) {
		ownerDrafts = ownerDrafts.filter((_, ownerIndex) => ownerIndex !== index);
		ownerError = '';
	}

	function updateOwner(index: number, patch: Partial<OwnerDraft>) {
		ownerDrafts = ownerDrafts.map((owner, ownerIndex) =>
			ownerIndex === index ? { ...owner, ...patch } : owner
		);
		ownerError = '';
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
					<Input
						id="property-code"
						value={initial?.code ?? field.state.value}
						readonly
						placeholder={initial ? undefined : m.properties_code_generated()}
						dir="ltr"
					/>
				</FormField>
			{/snippet}
		</form.Field>
		<form.Field name="property_type">
			{#snippet children(field)}
				<FormField label={m.properties_type()} id="property-type">
					<Select
						type="single"
						value={field.state.value}
						items={[
							{ value: 'residential', label: m.properties_type_residential() },
							{ value: 'commercial', label: m.properties_type_commercial() },
							{ value: 'mixed_use', label: m.properties_type_mixed() },
							{ value: 'land', label: m.properties_type_land() },
							{ value: 'other', label: m.common_other() },
							{ value: 'building', label: m.properties_type_building() },
							{ value: 'house_villa', label: m.properties_type_house_villa() },
							{ value: 'investment', label: m.properties_type_investment() },
							{ value: 'warehouse', label: m.properties_type_warehouse() },
							{ value: 'yard', label: m.properties_type_yard() },
							{ value: 'apartment', label: m.properties_type_apartment() }
						]}
						onValueChange={(value) => propertyType(value, field.handleChange)}
					>
						<SelectTrigger id="property-type" class="h-9 w-full rounded-md text-sm"><SelectValue /></SelectTrigger>
						<SelectContent>
							<SelectItem value="residential" label={m.properties_type_residential()}>{m.properties_type_residential()}</SelectItem>
							<SelectItem value="commercial" label={m.properties_type_commercial()}>{m.properties_type_commercial()}</SelectItem>
							<SelectItem value="mixed_use" label={m.properties_type_mixed()}>{m.properties_type_mixed()}</SelectItem>
							<SelectItem value="land" label={m.properties_type_land()}>{m.properties_type_land()}</SelectItem>
							<SelectItem value="other" label={m.common_other()}>{m.common_other()}</SelectItem>
							<SelectItem value="building" label={m.properties_type_building()}>{m.properties_type_building()}</SelectItem>
							<SelectItem value="house_villa" label={m.properties_type_house_villa()}>{m.properties_type_house_villa()}</SelectItem>
							<SelectItem value="investment" label={m.properties_type_investment()}>{m.properties_type_investment()}</SelectItem>
							<SelectItem value="warehouse" label={m.properties_type_warehouse()}>{m.properties_type_warehouse()}</SelectItem>
							<SelectItem value="yard" label={m.properties_type_yard()}>{m.properties_type_yard()}</SelectItem>
							<SelectItem value="apartment" label={m.properties_type_apartment()}>{m.properties_type_apartment()}</SelectItem>
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
					items={[
						{ value: 'active', label: m.common_enabled() },
						{ value: 'inactive', label: m.common_disabled() }
					]}
					onValueChange={(value) => propertyStatus(value, field.handleChange)}
				>
					<SelectTrigger id="property-status" class="h-9 w-full rounded-md text-sm">
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						<SelectItem value="active" label={m.common_enabled()}>{m.common_enabled()}</SelectItem>
						<SelectItem value="inactive" label={m.common_disabled()}>{m.common_disabled()}</SelectItem>
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
		<form.Field name="district">
			{#snippet children(field)}
				<FormField label={m.properties_district()} id="property-district">
					<Input id="property-district" value={field.state.value} oninput={(event) => field.handleChange(event.currentTarget.value)} />
				</FormField>
			{/snippet}
		</form.Field>
		<form.Field name="neighborhood">
			{#snippet children(field)}
				<FormField label={m.properties_neighborhood()} id="property-neighborhood">
					<Input id="property-neighborhood" value={field.state.value} oninput={(event) => field.handleChange(event.currentTarget.value)} />
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
					<Input id="property-country" value={field.state.value} oninput={(event) => field.handleChange(event.currentTarget.value.toUpperCase())} onblur={field.handleBlur} dir="ltr" maxlength={2} />
				</FormField>
			{/snippet}
		</form.Field>
	</div>
	<div class="grid gap-4 sm:grid-cols-2">
		<form.Field name="lot_area_square_meters">
			{#snippet children(field)}
				<FormField label={m.properties_lot_area()} id="property-lot-area">
					<Input id="property-lot-area" type="number" min="0" step="0.01" value={field.state.value ?? ''} oninput={(event) => field.handleChange(event.currentTarget.value === '' ? null : Number(event.currentTarget.value))} />
				</FormField>
			{/snippet}
		</form.Field>
		<form.Field name="building_area_square_meters">
			{#snippet children(field)}
				<FormField label={m.properties_building_area()} id="property-building-area">
					<Input id="property-building-area" type="number" min="0" step="0.01" value={field.state.value ?? ''} oninput={(event) => field.handleChange(event.currentTarget.value === '' ? null : Number(event.currentTarget.value))} />
				</FormField>
			{/snippet}
		</form.Field>
		<form.Field name="frontage_meters">
			{#snippet children(field)}
				<FormField label={m.properties_frontage()} id="property-frontage">
					<Input id="property-frontage" type="number" min="0" step="0.01" value={field.state.value ?? ''} oninput={(event) => field.handleChange(event.currentTarget.value === '' ? null : Number(event.currentTarget.value))} />
				</FormField>
			{/snippet}
		</form.Field>
		<form.Field name="depth_meters">
			{#snippet children(field)}
				<FormField label={m.properties_depth()} id="property-depth">
					<Input id="property-depth" type="number" min="0" step="0.01" value={field.state.value ?? ''} oninput={(event) => field.handleChange(event.currentTarget.value === '' ? null : Number(event.currentTarget.value))} />
				</FormField>
			{/snippet}
		</form.Field>
		<form.Field name="latitude">
			{#snippet children(field)}
				<FormField label={m.properties_latitude()} id="property-latitude">
					<Input id="property-latitude" type="number" min="-90" max="90" step="0.000001" value={field.state.value ?? ''} oninput={(event) => field.handleChange(event.currentTarget.value === '' ? null : Number(event.currentTarget.value))} dir="ltr" />
				</FormField>
			{/snippet}
		</form.Field>
		<form.Field name="longitude">
			{#snippet children(field)}
				<FormField label={m.properties_longitude()} id="property-longitude">
					<Input id="property-longitude" type="number" min="-180" max="180" step="0.000001" value={field.state.value ?? ''} oninput={(event) => field.handleChange(event.currentTarget.value === '' ? null : Number(event.currentTarget.value))} dir="ltr" />
				</FormField>
			{/snippet}
		</form.Field>
	</div>
	<section class="space-y-3 rounded-md border border-border p-3">
		<div class="flex items-center justify-between gap-3">
			<h3 class="text-sm font-medium">{m.properties_owners()}</h3>
			<Button type="button" variant="outline" onclick={addOwner}>
				<Plus size={16} />{m.properties_owner_add()}
			</Button>
		</div>
		<p class="text-xs text-muted-foreground">{m.properties_owner_shared_edit_notice()}</p>
		{#each ownerDrafts as owner, index (owner.id ?? index)}
			<div class="space-y-3 rounded-md border border-border/70 p-3">
				<FormField label={m.properties_owner_select()} id={`owner-select-${index}`}>
					<Select
						type="single"
						value={owner.id ?? 'new'}
						items={[
							{ value: 'new', label: m.properties_owner_new() },
							...ownerOptions.map((option) => ({
								value: option.id,
								label: `${option.name} — ${option.phone_number}`,
								disabled: ownerDrafts.some(
									(item, itemIndex) => item.id === option.id && itemIndex !== index
								)
							}))
						]}
						onValueChange={(value) => selectOwner(index, value)}
					>
						<SelectTrigger id={`owner-select-${index}`} class="h-9 w-full rounded-md text-sm"><SelectValue /></SelectTrigger>
						<SelectContent>
							<SelectItem value="new" label={m.properties_owner_new()}>{m.properties_owner_new()}</SelectItem>
							{#each ownerOptions as option (option.id)}
								<SelectItem
									value={option.id}
									label={`${option.name} — ${option.phone_number}`}
									disabled={ownerDrafts.some((item, itemIndex) => item.id === option.id && itemIndex !== index)}
								>
									{option.name} — {option.phone_number}
								</SelectItem>
							{/each}
						</SelectContent>
					</Select>
				</FormField>
				<div class="flex justify-end">
					<Button type="button" variant="ghost" onclick={() => removeOwner(index)}>
						<Trash2 size={16} />{m.properties_owner_remove()}
					</Button>
				</div>
				<div class="grid gap-3 sm:grid-cols-2">
					<FormField label={m.properties_owner_name()} id={`owner-name-${index}`}>
						<Input id={`owner-name-${index}`} value={owner.name} oninput={(event) => updateOwner(index, { name: event.currentTarget.value })} />
					</FormField>
					<FormField label={m.properties_owner_phone()} id={`owner-phone-${index}`}>
						<Input id={`owner-phone-${index}`} value={owner.phone_number} oninput={(event) => updateOwner(index, { phone_number: event.currentTarget.value })} dir="ltr" />
					</FormField>
					<FormField label={m.properties_owner_phone_label()} id={`owner-phone-label-${index}`}>
						<Input id={`owner-phone-label-${index}`} value={owner.phone_label} oninput={(event) => updateOwner(index, { phone_label: event.currentTarget.value })} />
					</FormField>
					<FormField label={m.properties_owner_share()} id={`owner-share-${index}`}>
						<Input id={`owner-share-${index}`} type="number" min="0" max="100" step="0.01" value={owner.ownership_share_percent} oninput={(event) => updateOwner(index, { ownership_share_percent: event.currentTarget.value })} />
					</FormField>
				</div>
				<label class="flex items-center gap-2 text-sm">
					<input type="checkbox" checked={owner.is_whatsapp} onchange={(event) => updateOwner(index, { is_whatsapp: event.currentTarget.checked })} />
					{m.properties_owner_whatsapp()}
				</label>
			</div>
		{/each}
		{#if ownerError}
			<p class="text-sm text-destructive" role="alert">{ownerError}</p>
		{/if}
	</section>
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
