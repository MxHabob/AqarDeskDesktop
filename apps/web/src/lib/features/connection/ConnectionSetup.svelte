<script lang="ts">
	import { Building2, Database, Globe2, Server, ShieldCheck } from '@lucide/svelte';
	import { Button, Card, Input } from '@aqardesk/ui';
	import type { ConnectionMode } from '#lib/api/desktop';
	import type { AppLocale } from '#lib/core/i18n';
	import FormField from '#lib/shared/components/FormField.svelte';
	import MessageBanner from '#lib/shared/components/MessageBanner.svelte';
	import * as m from '#lib/paraglide/messages';

	type Props = {
		mode: ConnectionMode;
		databasePath: string;
		serverUrl: string;
		host: string;
		port: number;
		databaseName: string;
		username: string;
		password: string;
		sslMode: string;
		configured: boolean;
		busy: boolean;
		error: string;
		notice: string;
		locale: AppLocale;
		onSubmit: (event: SubmitEvent) => void;
		onApiCheck: () => void;
		onModeChange: (mode: ConnectionMode) => void;
		onLanguageChange: () => void;
		onDatabasePathChange: (value: string) => void;
		onServerUrlChange: (value: string) => void;
		onHostChange: (value: string) => void;
		onPortChange: (value: number) => void;
		onDatabaseNameChange: (value: string) => void;
		onUsernameChange: (value: string) => void;
		onPasswordChange: (value: string) => void;
		onSslModeChange: (value: string) => void;
	};

	let {
		mode,
		databasePath,
		serverUrl,
		host,
		port,
		databaseName,
		username,
		password,
		sslMode,
		configured,
		busy,
		error,
		notice,
		locale,
		onSubmit,
		onApiCheck,
		onModeChange,
		onLanguageChange,
		onDatabasePathChange,
		onServerUrlChange,
		onHostChange,
		onPortChange,
		onDatabaseNameChange,
		onUsernameChange,
		onPasswordChange,
		onSslModeChange
	}: Props = $props();
</script>

<main class="grid min-h-dvh overflow-y-auto bg-slate-50 font-sans scrollbar-none lg:grid-cols-[minmax(18rem,0.8fr)_minmax(30rem,1.2fr)]">
	<aside class="hidden flex-col justify-between bg-emerald-950 p-10 text-white lg:flex">
		<a href="/" class="flex items-center gap-3 text-white no-underline">
			<span class="grid size-10 place-items-center rounded-lg bg-white/10"><Building2 size={21} /></span>
			<strong class="text-lg">{m.app_name()}</strong>
		</a>
		<div class="grid gap-4">
			<span class="grid size-14 place-items-center rounded-xl bg-white/10 text-emerald-100"><Building2 size={28} /></span>
			<p class="text-xs font-semibold tracking-widest text-emerald-200">{m.setup_kicker()}</p>
			<h1 class="text-3xl font-semibold leading-tight">{m.setup_title()}</h1>
			<p class="max-w-md text-sm leading-7 text-emerald-100/80">{m.setup_description()}</p>
		</div>
		<div class="grid gap-3 text-sm text-emerald-100">
			<div class="flex items-center gap-3"><span class="grid size-7 place-items-center rounded-full bg-white/10">1</span>{m.setup_step_connection()}</div>
			<div class="ms-3 h-4 border-s border-emerald-700"></div>
			<div class="flex items-center gap-3"><span class="grid size-7 place-items-center rounded-full bg-white/10">2</span>{m.setup_step_account()}</div>
			<div class="ms-3 h-4 border-s border-emerald-700"></div>
			<div class="flex items-center gap-3"><span class="grid size-7 place-items-center rounded-full bg-white/10">{configured ? '✓' : '3'}</span>{m.setup_step_ready()}</div>
		</div>
		<p class="flex items-center gap-2 text-xs text-emerald-100/75"><ShieldCheck size={15} />{m.connection_security_note()}</p>
	</aside>

	<section class="flex min-h-dvh flex-col p-5 sm:p-8 lg:px-14">
		<div class="flex items-center justify-between">
			<a href="/" class="flex items-center gap-2 text-sm font-semibold text-emerald-950 no-underline lg:hidden">
				<Building2 size={20} />{m.app_name()}
			</a>
			<span class="hidden lg:block"></span>
			<Button variant="outline" class="rounded-md" onclick={onLanguageChange}>
				<Globe2 size={16} />{locale === 'ar' ? m.language_english() : m.language_arabic()}
			</Button>
		</div>

		<div class="mx-auto my-auto grid w-full max-w-2xl gap-5 py-8">
			<div class="grid gap-2">
				<p class="text-xs font-semibold tracking-wide text-emerald-800">{m.setup_step_connection()}</p>
				<h2 class="text-2xl font-semibold tracking-tight text-slate-900">{m.setup_connection_title()}</h2>
				<p class="text-sm leading-relaxed text-slate-600">{m.setup_connection_description()}</p>
			</div>

			{#if error}<MessageBanner message={error} />{/if}
			{#if notice}<MessageBanner message={notice} variant="success" />{/if}

			<Card class="rounded-xl bg-white p-5 shadow-sm ring-slate-200 sm:p-6">
				<form class="grid gap-5" onsubmit={onSubmit}>
					<div class="grid gap-2 sm:grid-cols-3">
						<Button variant="outline" class={mode === 'local' ? 'h-auto min-h-24 flex-col items-start justify-start rounded-lg border-emerald-700 bg-emerald-50 p-3 text-start text-emerald-950' : 'h-auto min-h-24 flex-col items-start justify-start rounded-lg p-3 text-start'} onclick={() => onModeChange('local')}>
							<Database size={18} /><strong>{m.connection_local()}</strong><small class="whitespace-normal text-start text-xs font-normal">{m.connection_local_description()}</small>
						</Button>
						<Button variant="outline" class={mode === 'direct' ? 'h-auto min-h-24 flex-col items-start justify-start rounded-lg border-emerald-700 bg-emerald-50 p-3 text-start text-emerald-950' : 'h-auto min-h-24 flex-col items-start justify-start rounded-lg p-3 text-start'} onclick={() => onModeChange('direct')}>
							<Server size={18} /><strong>{m.connection_direct()}</strong><small class="whitespace-normal text-start text-xs font-normal">{m.connection_direct_description()}</small>
						</Button>
						<Button variant="outline" class={mode === 'api' ? 'h-auto min-h-24 flex-col items-start justify-start rounded-lg border-emerald-700 bg-emerald-50 p-3 text-start text-emerald-950' : 'h-auto min-h-24 flex-col items-start justify-start rounded-lg p-3 text-start'} onclick={() => onModeChange('api')}>
							<Globe2 size={18} /><strong>{m.connection_api()}</strong><small class="whitespace-normal text-start text-xs font-normal">{m.connection_api_description()}</small>
						</Button>
					</div>

					{#if mode === 'local'}
						<FormField label={m.connection_database_file()} id="database-path">
							<Input id="database-path" value={databasePath} oninput={(event) => onDatabasePathChange(event.currentTarget.value)} required dir="ltr" autocomplete="off" class="rounded-md" />
						</FormField>
					{:else if mode === 'api'}
						<FormField label={m.connection_server_url()} id="server-url">
							<Input id="server-url" type="url" value={serverUrl} oninput={(event) => onServerUrlChange(event.currentTarget.value)} required dir="ltr" placeholder="https://api.example.com" autocomplete="url" class="rounded-md" />
						</FormField>
					{:else}
						<div class="grid gap-4 sm:grid-cols-2">
							<FormField label={m.connection_host()} id="database-host"><Input id="database-host" value={host} oninput={(event) => onHostChange(event.currentTarget.value)} required dir="ltr" autocomplete="off" class="rounded-md" /></FormField>
							<FormField label={m.connection_port()} id="database-port"><Input id="database-port" type="number" value={port} oninput={(event) => onPortChange(Number(event.currentTarget.value))} min="1" max="65535" required dir="ltr" class="rounded-md" /></FormField>
							<FormField label={m.connection_database_name()} id="database-name"><Input id="database-name" value={databaseName} oninput={(event) => onDatabaseNameChange(event.currentTarget.value)} required dir="ltr" autocomplete="off" class="rounded-md" /></FormField>
							<FormField label={m.connection_username()} id="database-username"><Input id="database-username" value={username} oninput={(event) => onUsernameChange(event.currentTarget.value)} required dir="ltr" autocomplete="username" class="rounded-md" /></FormField>
							<FormField label={m.connection_password()} id="database-password"><Input id="database-password" type="password" value={password} oninput={(event) => onPasswordChange(event.currentTarget.value)} dir="ltr" autocomplete="new-password" class="rounded-md" /></FormField>
							<FormField label={m.connection_ssl_mode()} id="ssl-mode">
								<select id="ssl-mode" value={sslMode} onchange={(event) => onSslModeChange(event.currentTarget.value)} class="h-9 w-full rounded-md border border-slate-300 bg-white px-3 text-sm text-slate-900 outline-none focus-visible:ring-2 focus-visible:ring-emerald-700">
									<option value="prefer">prefer</option><option value="require">require</option><option value="disable">disable</option>
								</select>
							</FormField>
						</div>
						<p class="flex items-start gap-2 text-xs leading-relaxed text-slate-600"><ShieldCheck size={15} class="mt-0.5 shrink-0 text-emerald-800" />{m.connection_security_note()}</p>
					{/if}

					<div class="flex flex-col-reverse gap-2 sm:flex-row sm:justify-end">
						{#if mode === 'api'}
							<Button variant="outline" class="h-10 rounded-md" type="button" disabled={busy} onclick={onApiCheck}>{m.common_retry()}</Button>
						{/if}
						<Button class="h-10 rounded-md bg-emerald-800 px-5 text-white hover:bg-emerald-900" type="submit" disabled={busy}>
							{busy ? m.common_processing() : m.common_connect()}
						</Button>
					</div>
				</form>
			</Card>
			<footer class="flex justify-between text-xs text-slate-500"><span>© {new Date().getFullYear()} AqarDesk</span><span>{m.setup_step_connection()}</span></footer>
		</div>
	</section>
</main>
