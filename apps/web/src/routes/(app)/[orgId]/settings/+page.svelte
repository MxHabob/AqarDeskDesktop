<script lang="ts">
	import { onMount } from 'svelte';
	import { z } from 'zod';
	import { Input, Label, toast } from '@aqardesk/ui';
	import type { CurrencySettings, OrganizationSettings, OrganizationSettingsInput } from '@aqardesk/sdk';
	import { desktopApi } from '#lib/api/desktop';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import CurrencySelector from '#lib/features/settings/currency-selector.svelte';
	import SettingsBottomBar from '#lib/shared/components/settings-bottom-bar.svelte';
	import * as m from '#lib/paraglide/messages';

	const app = getAppContext();
	let saved = $state<OrganizationSettings | null>(null);
	let currencies = $state<CurrencySettings[]>([]);
	let draft = $state<OrganizationSettingsInput>({
		name: '',
		legal_name: null,
		default_currency: '',
		timezone: ''
	});
	let loading = $state(true);
	let saving = $state(false);
	const dirty = $derived(
		saved !== null &&
			(draft.name !== saved.name ||
				(draft.legal_name ?? '') !== (saved.legal_name ?? '') ||
				draft.default_currency !== saved.default_currency ||
				draft.timezone !== saved.timezone)
	);
	const schema = z.object({
		name: z.string().trim().min(1, m.settings_organization_name()),
		legal_name: z.string().nullable(),
		default_currency: z.string().refine(
			(code) =>
				currencies.some(
					(currency) =>
						currency.code === code &&
						currency.is_active &&
						currency.is_enabled_for_organization
				),
			m.settings_currency_hint()
		),
		timezone: z.string().trim().min(1, m.settings_timezone()).max(64)
	});

	onMount(() => {
		void load();
	});

	async function load() {
		loading = true;
		try {
			const [organization, availableCurrencies] = await Promise.all([
				desktopApi.getOrganizationSettings(),
				desktopApi.listCurrencies()
			]);
			currencies = availableCurrencies;
			saved = organization;
			draft = {
				name: organization.name,
				legal_name: organization.legal_name,
				default_currency: organization.default_currency,
				timezone: organization.timezone
			};
		} catch (cause) {
			toast.error(cause instanceof Error ? cause.message : String(cause));
		} finally {
			loading = false;
		}
	}

	async function save() {
		const parsed = schema.safeParse({
			...draft,
			legal_name: draft.legal_name?.trim() || null
		});
		if (!parsed.success) {
			toast.error(parsed.error.issues[0]?.message ?? m.error_unexpected_description());
			return;
		}

		saving = true;
		try {
			const organization = await desktopApi.updateOrganizationSettings(parsed.data);
			saved = organization;
			draft = {
				name: organization.name,
				legal_name: organization.legal_name,
				default_currency: organization.default_currency,
				timezone: organization.timezone
			};
			app.organizationName = organization.name;
			toast.success(m.common_save());
		} catch (cause) {
			toast.error(cause instanceof Error ? cause.message : String(cause));
		} finally {
			saving = false;
		}
	}

	function reset() {
		if (!saved) return;
		draft = {
			name: saved.name,
			legal_name: saved.legal_name,
			default_currency: saved.default_currency,
			timezone: saved.timezone
		};
	}
</script>

<header class="grid gap-1">
	<h2 class="text-lg font-semibold">{m.settings_organization()}</h2>
	<p class="text-sm text-muted-foreground">{m.settings_organization_description()}</p>
</header>

{#if loading}
	<div class="py-8 text-sm text-muted-foreground" role="status">{m.common_processing()}</div>
{:else if saved}
	<form
		class="grid min-w-0 divide-y divide-border/70 pb-20"
		onsubmit={(event) => {
			event.preventDefault();
			void save();
		}}
	>
		<div class="grid gap-2 py-4 sm:grid-cols-[12rem_minmax(0,1fr)] sm:items-center sm:gap-6">
			<Label for="organization-name">{m.settings_organization_name()}</Label>
			<Input id="organization-name" bind:value={draft.name} autocomplete="organization" />
		</div>
		<div class="grid gap-2 py-4 sm:grid-cols-[12rem_minmax(0,1fr)] sm:items-center sm:gap-6">
			<Label for="organization-legal-name">{m.settings_organization_legal_name()}</Label>
			<Input
				id="organization-legal-name"
				value={draft.legal_name ?? ''}
				oninput={(event) => (draft.legal_name = event.currentTarget.value)}
				autocomplete="organization-title"
			/>
		</div>
		<div class="grid gap-2 py-4 sm:grid-cols-[12rem_minmax(0,1fr)] sm:items-center sm:gap-6">
			<Label for="organization-currency">{m.settings_default_currency()}</Label>
			<div class="grid gap-1">
				<CurrencySelector
					id="organization-currency"
					{currencies}
					value={draft.default_currency}
					onChange={(code) => (draft.default_currency = code)}
				/>
				<p class="text-xs text-muted-foreground">{m.settings_currency_hint()}</p>
			</div>
		</div>
		<div class="grid gap-2 py-4 sm:grid-cols-[12rem_minmax(0,1fr)] sm:items-center sm:gap-6">
			<Label for="organization-timezone">{m.settings_timezone()}</Label>
			<Input
				id="organization-timezone"
				bind:value={draft.timezone}
				dir="ltr"
				autocomplete="off"
				placeholder="Asia/Riyadh"
			/>
		</div>
	</form>
{/if}
<SettingsBottomBar visible={dirty} isSaving={saving} onsave={save} onreset={reset} />
