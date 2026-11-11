<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import {
		Button,
		Card,
		CardContent,
		CardDescription,
		CardHeader,
		CardTitle,
		Input,
		toast
	} from '@aqardesk/ui';
	import {
		CalendarClock,
		Check,
		CreditCard,
		Fingerprint,
		KeyRound,
		RefreshCw,
		ShieldCheck
	} from '@lucide/svelte';
	import { desktopApi, type LicenseStateDto } from '#lib/api/desktop';
	import * as m from '#lib/paraglide/messages';

	const organizationId = $derived(page.params.orgId ?? '');
	let license = $state<LicenseStateDto | null>(null);
	let productKey = $state('');
	let loading = $state(true);
	let busy = $state(false);
	let error = $state('');

	const planDisplay = $derived(
		license?.plan_name ?? license?.plan ?? license?.product ?? '—'
	);
	const statusDisplay = $derived(license ? getStatusLabel(license.state) : '—');
	const statusClass = $derived(license ? getStatusClass(license.state) : 'border-border bg-muted text-muted-foreground');

	onMount(() => {
		void loadLicense();
	});

	async function loadLicense() {
		loading = true;
		error = '';
		try {
			license = await desktopApi.getLicenseState(organizationId);
		} catch (cause) {
			error = cause instanceof Error ? cause.message : String(cause);
		} finally {
			loading = false;
		}
	}

	async function runAction(action: () => Promise<LicenseStateDto>, clearKey = false) {
		busy = true;
		error = '';
		try {
			license = await action();
			if (clearKey) productKey = '';
			window.dispatchEvent(new Event('aqardesk-license-updated'));
			toast.success(m.billing_update_success());
		} catch (cause) {
			error = cause instanceof Error ? cause.message : String(cause);
			toast.error(error);
		} finally {
			busy = false;
		}
	}

	function deactivate() {
		if (!window.confirm(m.license_deactivate_confirm())) return;
		void runAction(desktopApi.deactivateLicense);
	}

	function getStatusLabel(state: LicenseStateDto['state']) {
		switch (state) {
			case 'pending': return m.billing_status_pending();
			case 'trial': return m.billing_status_trial();
			case 'active': return m.billing_status_active();
			case 'expiring': return m.billing_status_expiring();
			case 'offline_grace': return m.billing_status_offline_grace();
			case 'deactivation_pending': return m.billing_status_deactivation_pending();
			case 'expired': return m.billing_status_expired();
			case 'suspended': return m.billing_status_suspended();
			case 'revoked': return m.billing_status_revoked();
			case 'invalid': return m.billing_status_invalid();
			default: return m.billing_status_unavailable();
		}
	}

	function getStatusClass(state: LicenseStateDto['state']) {
		switch (state) {
			case 'active': return 'border-emerald-500/20 bg-emerald-500/10 text-emerald-700 dark:text-emerald-400';
			case 'trial': return 'border-blue-500/20 bg-blue-500/10 text-blue-700 dark:text-blue-400';
			case 'expiring':
			case 'offline_grace': return 'border-amber-500/20 bg-amber-500/10 text-amber-700 dark:text-amber-400';
			case 'expired':
			case 'suspended':
			case 'revoked':
			case 'invalid':
			case 'deactivation_pending': return 'border-destructive/20 bg-destructive/10 text-destructive';
			default: return 'border-border bg-muted text-muted-foreground';
		}
	}

	function formatDate(value: string | null) {
		if (!value) return '—';
		const date = new Date(value);
		return Number.isNaN(date.getTime())
			? '—'
			: new Intl.DateTimeFormat(undefined, { dateStyle: 'medium', timeStyle: 'short' }).format(date);
	}

	function getFeatureLabel(key: string) {
		switch (key) {
			case 'properties': return m.nav_properties();
			case 'units': return m.nav_units();
			case 'tenants': return m.nav_tenants();
			case 'payments': return m.nav_payments();
			case 'contracts': return m.nav_contracts();
			case 'user_management': return m.nav_users();
			case 'contract_templates': return m.settings_contract_templates();
			case 'audit_log': return m.settings_audit();
			case 'multi_currency': return m.settings_currencies();
			default: return key.replaceAll('_', ' ');
		}
	}

	function getLimitLabel(key: string) {
		switch (key) {
			case 'properties.max': return m.nav_properties();
			case 'units.max': return m.nav_units();
			case 'tenants.max': return m.nav_tenants();
			case 'users.max': return m.nav_users();
			case 'currencies.max': return m.settings_currencies();
			case 'contract_templates.max': return m.settings_contract_templates();
			default: return key.replaceAll('_', ' ').replace(/\.max$/, '');
		}
	}
</script>

<svelte:head>
	<title>{m.billing_title()} | AqarDesk</title>
</svelte:head>

<div class="mx-auto grid w-full max-w-6xl gap-5 pb-10">
	<header class="flex flex-wrap items-end justify-between gap-5 pb-1">
		<div class="grid gap-2">
			<h1 class="text-2xl font-semibold tracking-tight">{m.billing_title()}</h1>
			<p class="text-sm text-muted-foreground">{m.billing_description()}</p>
		</div>
		<Button
			variant="outline"
			size="sm"
			disabled={loading || busy}
			onclick={() => void runAction(desktopApi.refreshLicense)}
		>
			<RefreshCw size={15} class={loading || busy ? 'me-2 animate-spin' : 'me-2'} />
			{m.license_refresh()}
		</Button>
	</header>

	{#if error}
		<p class="rounded-md border border-destructive/20 bg-destructive/5 px-5 py-4 text-sm text-destructive" role="alert">
			{error}
			{#if !license}
				<Button variant="outline" size="sm" class="ms-3" onclick={() => void loadLicense()}>
					{m.common_retry()}
				</Button>
			{/if}
		</p>
	{/if}

	{#if loading && !license}
		<div class="rounded-lg border border-border/70 bg-card p-7 text-sm text-muted-foreground" role="status">
			{m.common_loading()}
		</div>
	{:else if license}
		<section class="grid gap-2.5" aria-labelledby="billing-plan-heading">
			<h2 id="billing-plan-heading" class="px-1 text-sm font-medium text-muted-foreground">
				{m.billing_current_plan()}
			</h2>
			<Card class="shadow-card ring-border/50">
				<CardContent class="flex flex-wrap items-center justify-between gap-5 p-6">
					<div class="flex min-w-0 items-center gap-4">
						<span class="grid size-11 shrink-0 place-items-center rounded-xl bg-primary/10 text-primary">
							<CreditCard size={21} />
						</span>
						<div class="grid min-w-0 gap-1">
							<div class="flex flex-wrap items-center gap-2">
								<span class="truncate text-base font-semibold" dir="auto">{planDisplay}</span>
								<span class={`rounded-full border px-2.5 py-0.5 text-xs font-medium ${statusClass}`}>
									{statusDisplay}
								</span>
							</div>
							{#if license.state === 'trial'}
								<p class="text-sm text-muted-foreground">
									{m.license_trial_days()}: {license.trial_days_remaining}
								</p>
							{:else if license.days_until_expiry !== null}
								<p class="text-sm text-muted-foreground">
									{m.license_expires()}: {formatDate(license.valid_until)}
								</p>
							{/if}
						</div>
					</div>
					<div class="flex items-center gap-2 text-sm text-muted-foreground">
						<ShieldCheck size={17} class="text-primary" />
						<span>{statusDisplay}</span>
					</div>
				</CardContent>
			</Card>
		</section>

		<section aria-labelledby="billing-details-heading">
			<Card class="shadow-card ring-border/50">
				<CardHeader class="px-6 pt-6 pb-4">
					<CardTitle id="billing-details-heading" class="text-base">{m.billing_details()}</CardTitle>
				</CardHeader>
				<CardContent class="grid gap-6 px-6 pt-0 pb-6 md:grid-cols-[16rem_minmax(0,1fr)] md:items-center">
					<div class="grid min-h-32 content-center gap-2.5 rounded-lg border border-border/70 bg-muted/30 p-5">
						<span class="text-xs font-medium text-muted-foreground">{m.license_plan()}</span>
						<span class="text-xl font-semibold" dir="auto">{planDisplay}</span>
						<span class={`w-fit rounded-full border px-2.5 py-0.5 text-xs font-medium ${statusClass}`}>
							{statusDisplay}
						</span>
					</div>
					<dl class="grid min-w-0">
						<div class="flex items-center justify-between gap-4 rounded-md px-3 py-2.5 odd:bg-muted/40">
							<dt class="text-sm text-muted-foreground">{m.license_status()}</dt>
							<dd class="text-sm font-medium">{statusDisplay}</dd>
						</div>
						<div class="flex items-center justify-between gap-4 rounded-md px-3 py-2.5 odd:bg-muted/40">
							<dt class="text-sm text-muted-foreground">{m.license_expires()}</dt>
							<dd class="text-sm font-medium tabular-nums" dir="auto">{formatDate(license.valid_until)}</dd>
						</div>
						<div class="flex items-center justify-between gap-4 rounded-md px-3 py-2.5 odd:bg-muted/40">
							<dt class="text-sm text-muted-foreground">{m.billing_last_sync()}</dt>
							<dd class="text-sm font-medium tabular-nums" dir="auto">
								{license.last_sync_at ? formatDate(license.last_sync_at) : m.billing_never_synced()}
							</dd>
						</div>
					</dl>
				</CardContent>
			</Card>
		</section>

		<section aria-labelledby="billing-schedule-heading">
			<Card class="shadow-card ring-border/50">
				<CardHeader class="px-6 pt-6 pb-4">
					<CardTitle id="billing-schedule-heading" class="flex items-center gap-2 text-base">
						<CalendarClock size={17} class="text-muted-foreground" />
						{m.billing_schedule()}
					</CardTitle>
				</CardHeader>
				<CardContent class="grid grid-cols-[minmax(0,1fr)_auto] gap-x-6 gap-y-4 px-6 pt-0 pb-6 text-sm sm:grid-cols-[minmax(0,1fr)_minmax(0,1fr)_auto]">
					<span class="text-xs text-muted-foreground">{m.billing_schedule_item()}</span>
					<span class="hidden text-xs text-muted-foreground sm:block">{m.license_plan()}</span>
					<span class="text-end text-xs text-muted-foreground">{m.license_expires()}</span>
					<span class="font-medium">{statusDisplay}</span>
					<span class="hidden text-muted-foreground sm:block" dir="auto">{planDisplay}</span>
					<span class="text-end font-medium tabular-nums" dir="auto">{formatDate(license.valid_until)}</span>
				</CardContent>
			</Card>
		</section>

		<section aria-labelledby="billing-limits-heading">
			<Card class="shadow-card ring-border/50">
				<CardHeader class="px-6 pt-6 pb-4">
					<CardTitle id="billing-limits-heading" class="text-base">{m.billing_limits()}</CardTitle>
				</CardHeader>
				<CardContent class="px-6 pt-0 pb-6">
					{#if license.limits.length}
						<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
							{#each license.limits as limit (limit.key)}
								<div class="grid gap-1.5 rounded-lg border border-border/70 p-5">
									<span class="text-xs font-medium text-muted-foreground">{getLimitLabel(limit.key)}</span>
									<span class="text-xl font-semibold tabular-nums">
										{limit.max_value === null ? m.billing_unlimited() : new Intl.NumberFormat().format(limit.max_value)}
									</span>
									{#if limit.unit}
										<span class="text-xs text-muted-foreground" dir="auto">{limit.unit.replaceAll('_', ' ')}</span>
									{/if}
								</div>
							{/each}
						</div>
					{:else}
						<p class="text-sm text-muted-foreground">{m.billing_limits_empty()}</p>
					{/if}
				</CardContent>
			</Card>
		</section>

		<section aria-labelledby="billing-features-heading">
			<Card class="shadow-card ring-border/50">
				<CardHeader class="px-6 pt-6 pb-4">
					<CardTitle id="billing-features-heading" class="text-base">{m.billing_features()}</CardTitle>
				</CardHeader>
				<CardContent class="px-6 pt-0 pb-6">
					{#if license.enabled_features.length}
						<div class="flex flex-wrap gap-2.5">
							{#each license.enabled_features as feature (feature)}
								<span class="inline-flex items-center gap-2.5 rounded-md border border-border bg-background px-3.5 py-2.5 text-sm font-medium">
									<Check size={14} class="shrink-0 text-primary" />
									<span>{getFeatureLabel(feature)}</span>
								</span>
							{/each}
						</div>
					{:else}
						<p class="text-sm text-muted-foreground">{m.billing_no_features()}</p>
					{/if}
				</CardContent>
			</Card>
		</section>

		<section class="grid gap-2.5" aria-labelledby="billing-installation-heading">
			<h2 id="billing-installation-heading" class="flex items-center gap-2 px-1 text-sm font-medium text-muted-foreground">
				<Fingerprint size={15} />
				{m.billing_installation_id()}
			</h2>
			<Card class="shadow-card ring-border/50">
				<CardContent class="grid gap-5 p-5 sm:grid-cols-[minmax(0,1fr)_auto] sm:items-center sm:p-6">
					<div class="grid min-w-0 gap-2">
						<p class="text-sm text-muted-foreground">{m.billing_installation_description()}</p>
						<code class="w-fit max-w-full break-all rounded-md border border-border/70 bg-muted/30 px-3 py-2 text-xs font-medium leading-relaxed" dir="ltr">{license.installation_id}</code>
					</div>
					{#if license.is_operational || license.state === 'deactivation_pending'}
						<Button variant="outline" disabled={busy} onclick={deactivate}>
							{m.license_deactivate()}
						</Button>
					{/if}
				</CardContent>
			</Card>
		</section>

		<section aria-labelledby="billing-manage-heading">
			<Card class="shadow-card ring-border/50">
				<CardHeader class="px-6 pt-6 pb-4">
					<CardTitle id="billing-manage-heading" class="flex items-center gap-2 text-base">
						<KeyRound size={17} class="text-muted-foreground" />
						{m.billing_manage()}
					</CardTitle>
					<CardDescription>{m.license_settings_description()}</CardDescription>
				</CardHeader>
				<CardContent class="grid gap-4 px-6 pt-0 pb-6">
					{#if license.configuration_error}
						<p class="text-sm text-destructive" role="alert">{license.configuration_error}</p>
					{:else if license.last_error}
						<p class="text-sm text-destructive" role="alert">{license.last_error}</p>
					{/if}
					<form
						class="grid gap-3 sm:grid-cols-[minmax(0,1fr)_auto_auto]"
						onsubmit={(event) => {
							event.preventDefault();
							void runAction(() => desktopApi.activateLicense(productKey), true);
						}}
					>
						<Input
							bind:value={productKey}
							placeholder={m.license_key_placeholder()}
							autocomplete="off"
							autocapitalize="characters"
							dir="ltr"
							class="font-mono"
							disabled={!license.is_configured || busy}
							aria-label={m.license_key_label()}
						/>
						<Button type="submit" disabled={!license.is_configured || !productKey.trim() || busy}>
							{m.license_activate()}
						</Button>
						{#if license.is_configured && license.state === 'unavailable'}
							<Button
								type="button"
								variant="outline"
								disabled={busy}
								onclick={() => void runAction(desktopApi.startLicenseTrial)}
							>
								{m.license_start_trial()}
							</Button>
						{/if}
					</form>
				</CardContent>
			</Card>
		</section>
	{/if}
</div>
