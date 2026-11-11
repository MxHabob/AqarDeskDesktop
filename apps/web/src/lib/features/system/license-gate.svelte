<script lang="ts">
	import { onMount, type Snippet } from 'svelte';
	import { Button, Input } from '@aqardesk/ui';
	import { desktopApi, type LicenseStateDto } from '#lib/api/desktop';
	import * as m from '#lib/paraglide/messages';

	type Props = {
		organizationId: string;
		routePath: string;
		children: Snippet;
	};

	let { organizationId, routePath, children }: Props = $props();
	let license = $state<LicenseStateDto | null>(null);
	let productKey = $state('');
	let loading = $state(true);
	let busy = $state(false);
	let error = $state('');

	const requiredFeature = $derived.by(() => {
		const parts = routePath.split('/').filter(Boolean);
		const orgIndex = parts.indexOf(organizationId);
		const section = parts.slice(orgIndex + 1);
		const propertySection = section[0] === 'properties' ? section[2] : null;
		const page = propertySection ?? section[0];
		switch (page) {
			case 'properties':
				return 'properties';
			case 'units':
				return 'units';
			case 'tenants':
				return 'tenants';
			case 'payments':
				return 'payments';
			case 'contracts':
				return 'contracts';
			case 'users':
				return 'user_management';
			case 'audit':
				return 'audit_log';
			case 'currencies':
				return 'multi_currency';
			case 'contract-templates':
				return 'contract_templates';
			default:
				return null;
		}
	});
	const isBillingPage = $derived(routePath.split('/').filter(Boolean).at(-1) === 'billing');

	const authorized = $derived(
		isBillingPage ||
			(license?.is_operational === true &&
				(requiredFeature === null || license.enabled_features.includes(requiredFeature)))
	);

	onMount(() => {
		const refresh = () => void loadLicense();
		window.addEventListener('aqardesk-license-updated', refresh);
		const timer = window.setInterval(() => void loadLicense(), 60_000);
		return () => {
			window.clearInterval(timer);
			window.removeEventListener('aqardesk-license-updated', refresh);
		};
	});

	$effect(() => {
		organizationId;
		requiredFeature;
		void loadLicense();
	});

	async function loadLicense() {
		try {
			license = await desktopApi.getLicenseState(organizationId);
			error = '';
		} catch (cause) {
			error = cause instanceof Error ? cause.message : String(cause);
		} finally {
			loading = false;
		}
	}

	async function runAction(action: () => Promise<LicenseStateDto>) {
		busy = true;
		error = '';
		try {
			license = await action();
			productKey = '';
		} catch (cause) {
			error = cause instanceof Error ? cause.message : String(cause);
		} finally {
			busy = false;
		}
	}

	function deactivate() {
		if (!window.confirm(m.license_deactivate_confirm())) return;
		void runAction(desktopApi.deactivateLicense);
	}
</script>

{#if authorized}
	{@render children()}
{:else if loading}
	<main class="grid h-full min-h-64 place-content-center bg-background text-sm text-muted-foreground" role="status">
		{m.common_loading()}
	</main>
{:else}
	<main class="grid h-full min-h-0 place-items-center overflow-y-auto bg-background p-4 text-foreground sm:p-8">
		<section class="w-full max-w-lg rounded-xl border border-border bg-card p-6 shadow-sm sm:p-8" aria-labelledby="license-title">
			<div class="grid gap-2">
				<h1 id="license-title" class="text-xl font-semibold">{m.license_title()}</h1>
				<p class="text-sm leading-6 text-muted-foreground">{m.license_description()}</p>
			</div>

			{#if license}
				<div class="mt-6 grid gap-2 rounded-lg bg-muted/50 p-4 text-sm">
					<p class="flex justify-between gap-4">
						<span class="text-muted-foreground">{m.license_status()}</span>
						<span class="font-medium capitalize" dir="auto">{license.state.replaceAll('_', ' ')}</span>
					</p>
					{#if license.plan_name || license.plan}
						<p class="flex justify-between gap-4">
							<span class="text-muted-foreground">{m.license_plan()}</span>
							<span class="font-medium" dir="auto">{license.plan_name ?? license.plan}</span>
						</p>
					{/if}
					{#if license.state === 'trial'}
						<p class="flex justify-between gap-4">
							<span class="text-muted-foreground">{m.license_trial_days()}</span>
							<span class="font-medium">{license.trial_days_remaining}</span>
						</p>
					{/if}
					{#if license.configuration_error}
						<p class="text-destructive">{license.configuration_error}</p>
					{:else if license.last_error}
						<p class="text-destructive">{license.last_error}</p>
					{/if}
				</div>
			{/if}

			{#if requiredFeature && license?.is_operational}
				<p class="mt-5 text-sm text-muted-foreground">{m.license_feature_missing()}</p>
			{/if}

			{#if error}
				<p class="mt-4 text-sm text-destructive" role="alert">{error}</p>
				{#if !license}
					<Button type="button" variant="outline" class="mt-3" onclick={() => void loadLicense()}>
						{m.common_retry()}
					</Button>
				{/if}
			{/if}

			<form
				class="mt-6 grid gap-3"
				onsubmit={(event) => {
					event.preventDefault();
					void runAction(() => desktopApi.activateLicense(productKey));
				}}
			>
				<label for="license-product-key" class="text-sm font-medium">{m.license_key_label()}</label>
				<Input
					id="license-product-key"
					bind:value={productKey}
					placeholder={m.license_key_placeholder()}
					autocomplete="off"
					autocapitalize="characters"
					dir="ltr"
					class="font-mono"
					disabled={!license?.is_configured || busy}
				/>
				<div class="flex flex-wrap gap-2">
					<Button type="submit" disabled={!license?.is_configured || !productKey.trim() || busy}>
						{busy ? m.common_processing() : m.license_activate()}
					</Button>
					{#if license?.is_configured && license.state === 'unavailable'}
						<Button
							type="button"
							variant="outline"
							disabled={busy}
							onclick={() => void runAction(desktopApi.startLicenseTrial)}
						>
							{m.license_start_trial()}
						</Button>
					{/if}
					{#if license?.is_operational}
						<Button type="button" variant="outline" disabled={busy} onclick={() => void runAction(desktopApi.refreshLicense)}>
							{m.license_refresh()}
						</Button>
					{/if}
					{#if license?.is_operational || license?.state === 'deactivation_pending'}
						<Button type="button" variant="ghost" disabled={busy} onclick={deactivate}>
							{m.license_deactivate()}
						</Button>
					{/if}
				</div>
			</form>
		</section>
	</main>
{/if}
