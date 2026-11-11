<script lang="ts">
	import { Building2, Database, Globe2, ShieldCheck, UsersRound } from '@lucide/svelte';
	import { Button, Card, Input } from '@aqardesk/ui';
	import type { AppLocale } from '#lib/core/i18n';
	import FormField from '#lib/shared/components/FormField.svelte';
	import MessageBanner from '#lib/shared/components/MessageBanner.svelte';
	import * as m from '#lib/paraglide/messages';

	type Props = {
		needsAdmin: boolean;
		busy: boolean;
		error: string;
		notice: string;
		locale: AppLocale;
		organizationName: string;
		name: string;
		email: string;
		password: string;
		onSubmit: (event: SubmitEvent) => void;
		onOpenConnection: () => void;
		onLanguageChange: () => void;
		onOrganizationNameChange: (value: string) => void;
		onNameChange: (value: string) => void;
		onEmailChange: (value: string) => void;
		onPasswordChange: (value: string) => void;
	};

	let {
		needsAdmin,
		busy,
		error,
		notice,
		locale,
		organizationName,
		name,
		email,
		password,
		onSubmit,
		onOpenConnection,
		onLanguageChange,
		onOrganizationNameChange,
		onNameChange,
		onEmailChange,
		onPasswordChange
	}: Props = $props();
</script>

<main class="grid min-h-dvh overflow-y-auto bg-slate-50 font-sans scrollbar-none lg:grid-cols-[minmax(0,1.1fr)_minmax(20rem,0.9fr)]">
	<section class="flex min-h-dvh flex-col p-5 sm:p-8 lg:px-14">
		<header class="flex items-center justify-between">
			<a href="/" class="flex items-center gap-2 text-sm font-semibold text-emerald-950 no-underline"><Building2 size={20} />{m.app_name()}</a>
			<Button variant="outline" class="rounded-md" onclick={onLanguageChange}><Globe2 size={16} />{locale === 'ar' ? m.language_english() : m.language_arabic()}</Button>
		</header>

		<div class="mx-auto my-auto grid w-full max-w-md gap-5 py-10">
			<span class="grid size-12 place-items-center rounded-xl bg-emerald-100 text-emerald-900">
				{#if needsAdmin}<ShieldCheck size={23} />{:else}<UsersRound size={23} />{/if}
			</span>
			<div class="grid gap-2">
				<p class="text-xs font-semibold tracking-wide text-emerald-800">{m.app_name()}</p>
				<h1 class="text-2xl font-semibold tracking-tight text-slate-900">{needsAdmin ? m.auth_admin_title() : m.auth_signin_title()}</h1>
				<p class="text-sm leading-relaxed text-slate-600">{needsAdmin ? m.auth_admin_description() : m.auth_signin_description()}</p>
			</div>
			{#if error}<MessageBanner message={error} />{/if}
			{#if notice}<MessageBanner message={notice} variant="success" />{/if}

			<Card class="rounded-xl bg-white p-5 shadow-sm ring-slate-200">
				<form class="grid gap-4" onsubmit={onSubmit}>
					{#if needsAdmin}
						<FormField label={m.auth_organization()} id="organization-name"><Input id="organization-name" value={organizationName} oninput={(event) => onOrganizationNameChange(event.currentTarget.value)} required autocomplete="organization" class="rounded-md" /></FormField>
						<FormField label={m.auth_full_name()} id="full-name"><Input id="full-name" value={name} oninput={(event) => onNameChange(event.currentTarget.value)} required autocomplete="name" class="rounded-md" /></FormField>
					{/if}
					<FormField label={m.auth_email()} id="account-email"><Input id="account-email" type="email" value={email} oninput={(event) => onEmailChange(event.currentTarget.value)} required dir="ltr" autocomplete="email" class="rounded-md" /></FormField>
					<FormField label={m.auth_password()} id="account-password" description={needsAdmin ? m.auth_password_hint() : undefined}>
						<Input id="account-password" type="password" value={password} oninput={(event) => onPasswordChange(event.currentTarget.value)} required minlength={needsAdmin ? 12 : undefined} dir="ltr" autocomplete={needsAdmin ? 'new-password' : 'current-password'} class="rounded-md" />
					</FormField>
					<Button class="mt-1 h-10 rounded-md bg-emerald-800 text-white hover:bg-emerald-900" type="submit" disabled={busy}>{busy ? m.common_processing() : needsAdmin ? m.auth_create_admin() : m.auth_sign_in()}</Button>
				</form>
				<Button variant="ghost" class="mt-3 h-9 rounded-md text-slate-600" onclick={onOpenConnection}><Database size={15} />{m.connection_edit()}</Button>
			</Card>
		</div>
		<footer class="flex items-center justify-between border-t border-slate-200 pt-4 text-xs text-slate-500">
			<span>{m.setup_step_connection()}</span><span class="flex items-center gap-2 text-emerald-800"><span class="size-2 rounded-full bg-emerald-500"></span>{m.status_connected()}</span>
		</footer>
	</section>

	<aside class="hidden flex-col justify-center gap-7 bg-emerald-950 p-10 text-white lg:flex xl:p-16">
		<span class="grid size-14 place-items-center rounded-xl bg-white/10 text-emerald-100"><ShieldCheck size={28} /></span>
		<div class="grid gap-3"><p class="text-xs font-semibold tracking-wide text-emerald-200">{m.setup_step_ready()}</p><h2 class="text-2xl font-semibold leading-relaxed">{m.dashboard_subtitle()}</h2></div>
		<div class="grid gap-3">
			{#each [m.dashboard_properties_title(), m.dashboard_tenants_title(), m.dashboard_contracts_title(), m.dashboard_payments_title()] as benefit}
				<div class="flex items-center gap-3 rounded-lg border border-white/10 bg-white/5 p-3 text-sm text-emerald-50"><Building2 size={17} />{benefit}</div>
			{/each}
		</div>
	</aside>
</main>
