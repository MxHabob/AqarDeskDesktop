<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { Globe2, ShieldCheck } from '@lucide/svelte';
	import { Button } from '@aqardesk/ui';
	import AqarDeskLogo from '#lib/shared/components/aqardesk-logo.svelte';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import ConnectionForm from '#lib/features/connection/connection-form.svelte';
	import * as m from '#lib/paraglide/messages';

	const app = getAppContext();
	let desktopShell = $state(false);

	onMount(() => {
		desktopShell = '__TAURI_INTERNALS__' in window;
	});

	async function saveAndContinue(event: SubmitEvent) {
		if (await app.saveAndConnect(event)) {
			await goto(app.currentUser ? `/${app.currentUser.organization_id}` : '/login');
		}
	}

</script>

<main class="grid h-full min-h-0 overflow-y-auto bg-background font-sans text-foreground scrollbar-none lg:grid-cols-[minmax(18rem,0.8fr)_minmax(30rem,1.2fr)]">
	<aside class="relative hidden min-h-0 flex-col justify-between overflow-hidden bg-black p-10 text-background lg:flex">
		<div
			class="absolute inset-0 bg-[url('/assets/bg-login-indigo.jpg')] bg-cover bg-center"
			aria-hidden="true"
		></div>
		<div class="absolute inset-0 bg-black/20" aria-hidden="true"></div>
		<div class="relative z-10 grid gap-4">
			<span class="grid size-14 place-items-center rounded-xl text-foreground">
				<AqarDeskLogo size={40} class="text-primary" />
			</span>
			<p class="text-xs font-semibold tracking-widest text-primary">{m.setup_kicker()}</p>
			<h1 class="text-3xl font-semibold leading-tight">{m.setup_title()}</h1>
			<p class="max-w-md text-sm leading-7 text-background/70">{m.setup_description()}</p>
		</div>
		<div class="relative z-10 grid gap-3 text-sm text-background/80">
			<div class="flex items-center gap-3">
				<span class="grid size-7 place-items-center rounded-full bg-background/10">1</span>
				{m.setup_step_connection()}
			</div>
			<div class="ms-3 h-4 border-s border-background/20"></div>
			<div class="flex items-center gap-3">
				<span class="grid size-7 place-items-center rounded-full bg-background/10">2</span>
				{m.setup_step_account()}
			</div>
			<div class="ms-3 h-4 border-s border-background/20"></div>
			<div class="flex items-center gap-3">
				<span class="grid size-7 place-items-center rounded-full bg-background/10">
					{app.configured ? '✓' : '3'}
				</span>
				{m.setup_step_ready()}
			</div>
		</div>
		<p class="relative z-10 flex items-center gap-2 text-xs text-background/70">
			<ShieldCheck size={15} />{m.connection_security_note()}
		</p>
	</aside>

	<section class="flex min-h-0 flex-col p-5 sm:p-8 lg:px-14">
		<div class="flex items-center justify-between">
			<a href="/" class="flex items-center gap-2 text-sm font-semibold text-foreground no-underline lg:hidden">
				<AqarDeskLogo size={20} class="text-foreground" />{m.app_name()}
			</a>
			<span class="hidden lg:block"></span>
			{#if !desktopShell}<Button
				variant="outline"
				class="rounded-md"
				onclick={() => void app.changeLanguage(app.locale === 'ar' ? 'en' : 'ar')}
			>
				<Globe2 size={16} />
				{app.locale === 'ar' ? m.language_english() : m.language_arabic()}
			</Button>{/if}
		</div>

		<div class="mx-auto my-auto grid w-full max-w-2xl gap-5 py-8">
			<div class="grid gap-2">
				<p class="text-xs font-semibold tracking-wide text-muted-foreground">{m.setup_step_connection()}</p>
				<h2 class="text-2xl font-semibold tracking-tight text-foreground">
					{m.setup_connection_title()}
				</h2>
				<p class="text-sm leading-relaxed text-muted-foreground">
					{m.setup_connection_description()}
				</p>
			</div>

			<ConnectionForm
				mode={app.mode}
				databasePath={app.databasePath}
				serverUrl={app.serverUrl}
				host={app.host}
				port={app.port}
				databaseName={app.databaseName}
				username={app.username}
				password={app.password}
				sslMode={app.sslMode}
				busy={app.busy}
				onSubmit={saveAndContinue}
				onApiCheck={app.checkApi}
				onModeChange={(value) => (app.mode = value)}
				onDatabasePathChange={(value) => (app.databasePath = value)}
				onServerUrlChange={(value) => (app.serverUrl = value)}
				onHostChange={(value) => (app.host = value)}
				onPortChange={(value) => (app.port = value)}
				onDatabaseNameChange={(value) => (app.databaseName = value)}
				onUsernameChange={(value) => (app.username = value)}
				onPasswordChange={(value) => (app.password = value)}
				onSslModeChange={(value) => (app.sslMode = value)}
			/>

			<footer class="flex justify-between text-xs text-muted-foreground">
				<span>© {new Date().getFullYear()} Mulverse</span>
				<span>{m.setup_step_connection()}</span>
			</footer>
		</div>
	</section>
</main>
