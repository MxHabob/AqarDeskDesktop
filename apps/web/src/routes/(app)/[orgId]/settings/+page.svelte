<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { z } from 'zod';
	import {
		Button,
		Input,
		Select,
		SelectContent,
		SelectItem,
		SelectTrigger,
		SelectValue,
		toast
	} from '@aqardesk/ui';
	import type { CurrencySettings, OrganizationSettings, OrganizationSettingsInput } from '@aqardesk/sdk';
	import { desktopApi } from '#lib/api/desktop';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { hasPermission } from '#lib/features/auth/permissions';
	import CurrencySelector from '#lib/features/settings/currency-selector.svelte';
	import ConnectionSettingsCard from '#lib/features/settings/connection-settings-card.svelte';
	import LanguageSettingsCard from '#lib/features/settings/language-settings-card.svelte';
	import UpdateSettingsCard from '#lib/features/settings/update-settings-card.svelte';
	import SettingsBottomBar from '#lib/shared/components/settings-bottom-bar.svelte';
	import SettingsFieldRow from '#lib/shared/components/settings-field-row.svelte';
	import * as m from '#lib/paraglide/messages';

	const app = getAppContext();
	const timeZones = ['UTC', ...Intl.supportedValuesOf('timeZone')];
	const canEditOrganization = $derived(
		hasPermission(app.currentUser?.role, 'manage_administration', app.currentUser?.permissions)
	);
	let saved = $state<OrganizationSettings | null>(null);
	let currencies = $state<CurrencySettings[]>([]);
	let draft = $state<OrganizationSettingsInput>({
		name: '',
		legal_name: null,
		default_currency: '',
		timezone: ''
	});
	let draftLocale = $state(app.locale);
	let loading = $state(true);
	let saving = $state(false);
	const organizationDirty = $derived(
		canEditOrganization &&
			saved !== null &&
			(draft.name !== saved.name ||
				(draft.legal_name ?? '') !== (saved.legal_name ?? '') ||
				draft.default_currency !== saved.default_currency ||
				draft.timezone !== saved.timezone)
	);
	const localeDirty = $derived(draftLocale !== app.locale);
	const dirty = $derived(organizationDirty || localeDirty);
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
		let organizationInput: OrganizationSettingsInput | null = null;
		if (organizationDirty) {
			const parsed = schema.safeParse({
				...draft,
				legal_name: draft.legal_name?.trim() || null
			});
			if (!parsed.success) {
				toast.error(parsed.error.issues[0]?.message ?? m.error_unexpected_description());
				return;
			}
			organizationInput = parsed.data;
		}

		saving = true;
		let didSave = false;
		try {
			if (organizationInput) {
				const organization = await desktopApi.updateOrganizationSettings(organizationInput);
				saved = organization;
				draft = {
					name: organization.name,
					legal_name: organization.legal_name,
					default_currency: organization.default_currency,
					timezone: organization.timezone
				};
				app.organizationName = organization.name;
				app.organizationTimezone = organization.timezone;
				didSave = true;
			}

			if (localeDirty) {
				if (await app.changeLanguage(draftLocale)) didSave = true;
			}

			if (didSave) toast.success(m.common_save());
		} catch (cause) {
			toast.error(cause instanceof Error ? cause.message : String(cause));
		} finally {
			saving = false;
		}
	}

	function reset() {
		if (saved) {
			draft = {
				name: saved.name,
				legal_name: saved.legal_name,
				default_currency: saved.default_currency,
				timezone: saved.timezone
			};
		}
		draftLocale = app.locale;
	}

	function editConnection() {
		app.clearMessages();
		void goto('/connection');
	}

	const timezoneOptions = $derived(
		timeZones.includes(draft.timezone) || !draft.timezone
			? timeZones
			: [...timeZones, draft.timezone].toSorted((left, right) => left.localeCompare(right))
	);
</script>

{#if loading}
	<div class="py-8 text-sm text-muted-foreground" role="status">{m.common_processing()}</div>
{:else if saved}
	<form
		class="grid min-w-0 gap-6 pb-24"
		onsubmit={(event) => {
			event.preventDefault();
			void save();
		}}
	>
		<section class="overflow-hidden rounded-lg border border-border/70 bg-card">
			<div class="divide-y divide-border/70">
				<SettingsFieldRow
					label={m.settings_organization_id()}
					id="organization-id"
					description={m.settings_organization_id_description()}
					copyValue={saved.id}
				>
					<Input id="organization-id" value={saved.id} readonly dir="ltr" class="w-full font-mono text-xs" />
				</SettingsFieldRow>
				<SettingsFieldRow
					label={m.settings_organization_name()}
					id="organization-name"
					description={m.settings_organization_name_description()}
				>
					<Input
						id="organization-name"
						bind:value={draft.name}
						autocomplete="organization"
						disabled={!canEditOrganization}
						class="w-full"
					/>
				</SettingsFieldRow>
				<SettingsFieldRow
					label={m.settings_organization_legal_name()}
					id="organization-legal-name"
					description={m.settings_organization_legal_name_description()}
				>
					<Input
						id="organization-legal-name"
						value={draft.legal_name ?? ''}
						oninput={(event) => (draft.legal_name = event.currentTarget.value)}
						autocomplete="organization-title"
						disabled={!canEditOrganization}
						class="w-full"
					/>
				</SettingsFieldRow>
				<SettingsFieldRow
					label={m.settings_default_currency()}
					id="organization-currency"
					description={m.settings_currency_hint()}
				>
					<div class="w-full">
						<CurrencySelector
							id="organization-currency"
							{currencies}
							value={draft.default_currency}
							onChange={(code) => (draft.default_currency = code)}
							disabled={!canEditOrganization}
						/>
					</div>
				</SettingsFieldRow>
				<SettingsFieldRow
					label={m.settings_timezone()}
					id="organization-timezone"
					description={m.settings_timezone_description()}
				>
					<Select
						type="single"
						value={draft.timezone}
						disabled={!canEditOrganization}
						onValueChange={(timezone) => {
							if (timezone) draft.timezone = timezone;
						}}
					>
						<SelectTrigger id="organization-timezone" class="h-9 w-full rounded-md text-sm" dir="ltr">
							<SelectValue />
						</SelectTrigger>
						<SelectContent class="max-h-72">
							{#each timezoneOptions as timezone (timezone)}
								<SelectItem value={timezone}>{timezone}</SelectItem>
							{/each}
						</SelectContent>
					</Select>
				</SettingsFieldRow>
				<LanguageSettingsCard
					locale={draftLocale}
					onChange={(value) => (draftLocale = value)}
				/>
				<ConnectionSettingsCard
					mode={app.mode}
					connected={app.connected}
					onEdit={editConnection}
				/>
				<UpdateSettingsCard
					busy={app.updateBusy}
					result={app.updateResult}
					progress={app.updateProgress}
					progressPercent={app.progressPercent}
					error={app.updateError}
					downloading={app.isDownloadingUpdate}
					ready={app.isUpdateReady}
					onCheck={app.checkForUpdates}
					onDownload={app.downloadUpdate}
					onInstall={app.installUpdate}
				/>
			</div>
		</section>
	</form>
{/if}

<SettingsBottomBar visible={dirty} isSaving={saving} onsave={save} onreset={reset} />
