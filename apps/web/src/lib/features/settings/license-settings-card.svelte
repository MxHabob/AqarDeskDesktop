<script lang="ts">
	import { onMount } from 'svelte';
	import { Button, Input } from '@aqardesk/ui';
	import { desktopApi, type LicenseStateDto } from '#lib/api/desktop';
	import * as m from '#lib/paraglide/messages';

	let { organizationId }: { organizationId: string } = $props();
	let license = $state<LicenseStateDto | null>(null);
	let productKey = $state('');
	let busy = $state(false);
	let error = $state('');

	onMount(() => {
		void load();
	});

	async function load() {
		try {
			license = await desktopApi.getLicenseState(organizationId);
			error = '';
		} catch (cause) {
			error = cause instanceof Error ? cause.message : String(cause);
		}
	}

	async function run(action: () => Promise<LicenseStateDto>) {
		busy = true;
		error = '';
		try {
			license = await action();
			productKey = '';
			window.dispatchEvent(new Event('aqardesk-license-updated'));
		} catch (cause) {
			error = cause instanceof Error ? cause.message : String(cause);
		} finally {
			busy = false;
		}
	}

	function deactivate() {
		if (!window.confirm(m.license_deactivate_confirm())) return;
		void run(desktopApi.deactivateLicense);
	}
</script>

<section class="grid gap-4 rounded-lg border border-border/70 bg-card p-4 sm:p-5" aria-labelledby="license-settings-title">
	<div class="grid gap-1">
		<h2 id="license-settings-title" class="text-sm font-semibold">{m.license_settings_title()}</h2>
		<p class="text-xs text-muted-foreground">{m.license_settings_description()}</p>
	</div>

	{#if license}
		<div class="flex flex-wrap gap-x-6 gap-y-2 text-sm">
			<p>
				<span class="text-muted-foreground">{m.license_status()}: </span>
				<span class="font-medium capitalize" dir="auto">{license.state.replaceAll('_', ' ')}</span>
			</p>
			{#if license.plan_name || license.plan}
				<p>
					<span class="text-muted-foreground">{m.license_plan()}: </span>
					<span class="font-medium" dir="auto">{license.plan_name ?? license.plan}</span>
				</p>
			{/if}
			{#if license.valid_until}
				<p>
					<span class="text-muted-foreground">{m.license_expires()}: </span>
					<span dir="ltr">{new Intl.DateTimeFormat(undefined, { dateStyle: 'medium' }).format(new Date(license.valid_until))}</span>
				</p>
			{/if}
		</div>
	{/if}

	{#if license?.configuration_error}
		<p class="text-sm text-destructive" role="alert">{license.configuration_error}</p>
	{:else if license?.last_error}
		<p class="text-sm text-destructive" role="alert">{license.last_error}</p>
	{/if}
	{#if error}
		<p class="text-sm text-destructive" role="alert">{error}</p>
	{/if}

	<form
		class="grid gap-2 sm:grid-cols-[minmax(0,1fr)_auto_auto_auto]"
		onsubmit={(event) => {
			event.preventDefault();
			void run(() => desktopApi.activateLicense(productKey));
		}}
	>
		<Input
			bind:value={productKey}
			placeholder={m.license_key_placeholder()}
			autocomplete="off"
			autocapitalize="characters"
			dir="ltr"
			class="font-mono"
			disabled={!license?.is_configured || busy}
			aria-label={m.license_key_label()}
		/>
		<Button type="submit" disabled={!license?.is_configured || !productKey.trim() || busy}>
			{m.license_activate()}
		</Button>
		<Button type="button" variant="outline" disabled={busy} onclick={() => void run(desktopApi.refreshLicense)}>
			{m.license_refresh()}
		</Button>
		<Button
			type="button"
			variant="ghost"
			disabled={busy || !license?.organization_id}
			onclick={deactivate}
		>
			{m.license_deactivate()}
		</Button>
	</form>
</section>
