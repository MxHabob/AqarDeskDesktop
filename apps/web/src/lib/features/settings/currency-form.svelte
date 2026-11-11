<script lang="ts">
	import { untrack } from 'svelte';
	import { z } from 'zod';
	import { Button, Input, Select, SelectContent, SelectItem, SelectTrigger, SelectValue, SheetFooter } from '@aqardesk/ui';
	import type { CurrencySettings } from '@aqardesk/sdk';
	import type { CurrencyEditorInput } from '#lib/api/desktop';
	import FormField from '#lib/shared/components/form-field.svelte';
	import * as m from '#lib/paraglide/messages';

	type Values = CurrencyEditorInput;

	let {
		initial,
		busy,
		onSubmit,
		onCancel
	}: {
		initial?: CurrencySettings;
		busy: boolean;
		onSubmit: (input: Values) => Promise<boolean>;
		onCancel: () => void;
	} = $props();

	const initialValues = untrack((): Values => ({
		code: initial?.code ?? '',
		name: initial?.name ?? '',
		name_ar: initial?.name_ar ?? '',
		symbol: initial?.symbol ?? '',
		symbol_position: initial?.symbol_position === 'prefix' ? 'prefix' : 'suffix',
		decimal_digits: initial?.decimal_digits ?? 2,
		enable_for_organization: initial?.is_enabled_for_organization ?? true
	}));
	let values = $state<Values>(initialValues);
	let errors = $state<Partial<Record<keyof Values, string>>>({});

	const schema = z.object({
		code: z.string().trim().toUpperCase().regex(/^[A-Z]{3}$/, m.settings_currency_code_invalid()),
		name: z.string().trim().min(1, m.settings_currency_name_required()).max(120),
		name_ar: z.string().trim().max(120),
		symbol: z.string().trim().min(1, m.settings_currency_symbol_required()).max(12),
		symbol_position: z.enum(['prefix', 'suffix']),
		decimal_digits: z.number().int().min(0).max(4),
		enable_for_organization: z.boolean()
	});

	async function submit(event: SubmitEvent) {
		event.preventDefault();
		const parsed = schema.safeParse({
			...values,
			code: values.code.trim().toUpperCase(),
			name: values.name.trim(),
			name_ar: values.name_ar?.trim() ?? '',
			symbol: values.symbol.trim()
		});
		if (!parsed.success) {
			errors = {};
			for (const issue of parsed.error.issues) {
				const key = issue.path[0] as keyof Values | undefined;
				if (key && !errors[key]) errors[key] = issue.message;
			}
			return;
		}
		errors = {};
		const saved = await onSubmit({
			...parsed.data,
			name_ar: parsed.data.name_ar || null
		});
		if (saved) onCancel();
	}
</script>

<form id="currency-form" class="flex min-h-0 flex-1 flex-col" onsubmit={submit}>
	<div class="grid min-h-0 flex-1 content-start gap-4 overflow-y-auto p-4 scrollbar-none">
		<FormField
			label={m.settings_currency_code()}
			id="currency-code"
			error={errors.code}
			description={initial ? m.settings_currency_code_immutable() : undefined}
		>
			<Input
				id="currency-code"
				value={values.code}
				oninput={(event) => {
					values.code = event.currentTarget.value.toUpperCase();
				}}
				dir="ltr"
				maxlength={3}
				autocomplete="off"
				disabled={Boolean(initial)}
			/>
		</FormField>
		<FormField label={m.settings_currency_name()} id="currency-name" error={errors.name}>
			<Input
				id="currency-name"
				bind:value={values.name}
				autocomplete="off"
			/>
		</FormField>
		<FormField
			label={m.settings_currency_arabic_name()}
			id="currency-name-ar"
			error={errors.name_ar}
		>
			<Input id="currency-name-ar" bind:value={values.name_ar} autocomplete="off" dir="rtl" />
		</FormField>
		<div class="grid gap-4 sm:grid-cols-2">
			<FormField
				label={m.settings_currency_symbol()}
				id="currency-symbol"
				error={errors.symbol}
			>
				<Input id="currency-symbol" bind:value={values.symbol} autocomplete="off" />
			</FormField>
			<FormField
				label={m.settings_currency_decimal_digits()}
				id="currency-decimal-digits"
				error={errors.decimal_digits}
			>
				<Input
					id="currency-decimal-digits"
					type="number"
					min={0}
					max={4}
					step={1}
					value={values.decimal_digits}
					oninput={(event) => {
						const value = event.currentTarget.valueAsNumber;
						values.decimal_digits = Number.isNaN(value) ? -1 : value;
					}}
				/>
			</FormField>
		</div>
		<FormField label={m.settings_currency_symbol_position()} id="currency-symbol-position">
			<Select
				type="single"
				value={values.symbol_position}
				onValueChange={(value) => {
					if (value === 'prefix' || value === 'suffix') values.symbol_position = value;
				}}
			>
				<SelectTrigger id="currency-symbol-position" class="h-9 w-full rounded-md text-sm">
					<SelectValue />
				</SelectTrigger>
				<SelectContent>
					<SelectItem value="prefix">{m.settings_currency_prefix()}</SelectItem>
					<SelectItem value="suffix">{m.settings_currency_suffix()}</SelectItem>
				</SelectContent>
			</Select>
		</FormField>
		{#if !initial}
			<label class="flex items-center gap-2 text-sm">
				<input
					type="checkbox"
					bind:checked={values.enable_for_organization}
					class="size-4 accent-primary"
				/>
				{m.settings_currency_enable_for_organization()}
			</label>
		{/if}
	</div>
</form>
<SheetFooter class="shrink-0 border-t border-border bg-popover px-4 py-3">
	<Button type="button" variant="outline" disabled={busy} onclick={onCancel}>
		{m.common_cancel()}
	</Button>
	<Button type="submit" form="currency-form" disabled={busy}>
		{busy ? m.common_processing() : m.common_save()}
	</Button>
</SheetFooter>
