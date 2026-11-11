<script lang="ts">
	import { ArrowUpRight, Building2, FileText, ShieldCheck, UsersRound, WalletCards } from '@lucide/svelte';
	import { Button, Card } from '@aqardesk/ui';
	import * as m from '#lib/paraglide/messages';
	import type { User } from '#lib/features/auth/types';
	import type { ConnectionMode } from '#lib/api/desktop';
	import type { ViewKey } from '#lib/shell/navigation';

	type Props = {
		currentUser: User;
		mode: ConnectionMode;
		onSelect: (view: ViewKey) => void;
		onEditConnection: () => void;
	};

	let { currentUser, mode, onSelect, onEditConnection }: Props = $props();
</script>

<section class="grid gap-6">
	<header class="flex items-center justify-between gap-5 rounded-xl border border-emerald-100 bg-gradient-to-br from-white to-emerald-50 p-5 sm:p-8">
		<div class="grid gap-2">
			<p class="text-xs font-semibold tracking-wide text-emerald-800">{m.nav_overview()}</p>
			<h1 class="text-2xl font-semibold tracking-tight text-slate-900 sm:text-3xl">{m.dashboard_greeting({ name: currentUser.name })}</h1>
			<p class="max-w-2xl text-sm leading-relaxed text-slate-600">{m.dashboard_subtitle()}</p>
		</div>
		<span class="hidden size-16 shrink-0 place-items-center rounded-xl bg-white text-emerald-800 shadow-sm sm:grid"><Building2 size={30} /></span>
	</header>

	<div class="grid gap-3">
		<div><h2 class="text-base font-semibold text-slate-900">{m.dashboard_quick_actions()}</h2><p class="mt-1 text-sm text-slate-600">{m.app_tagline()}</p></div>
		<div class="grid gap-3 sm:grid-cols-2">
			<Card class="rounded-lg bg-white p-4 shadow-sm ring-slate-200">
				<Button variant="ghost" class="h-auto w-full justify-start gap-4 whitespace-normal rounded-md p-0 text-start" onclick={() => onSelect('properties')}>
					<span class="grid size-11 shrink-0 place-items-center rounded-lg bg-emerald-50 text-emerald-800"><Building2 size={21} /></span>
					<span class="grid min-w-0 flex-1 gap-1"><strong class="text-sm">{m.dashboard_properties_title()}</strong><small class="text-xs font-normal leading-relaxed text-slate-600">{m.dashboard_properties_description()}</small></span><ArrowUpRight size={17} class="shrink-0 text-slate-400" />
				</Button>
			</Card>
			<Card class="rounded-lg bg-white p-4 shadow-sm ring-slate-200">
				<Button variant="ghost" class="h-auto w-full justify-start gap-4 whitespace-normal rounded-md p-0 text-start" onclick={() => onSelect('tenants')}>
					<span class="grid size-11 shrink-0 place-items-center rounded-lg bg-sky-50 text-sky-800"><UsersRound size={21} /></span>
					<span class="grid min-w-0 flex-1 gap-1"><strong class="text-sm">{m.dashboard_tenants_title()}</strong><small class="text-xs font-normal leading-relaxed text-slate-600">{m.dashboard_tenants_description()}</small></span><ArrowUpRight size={17} class="shrink-0 text-slate-400" />
				</Button>
			</Card>
			<Card class="rounded-lg bg-white p-4 shadow-sm ring-slate-200">
				<Button variant="ghost" class="h-auto w-full justify-start gap-4 whitespace-normal rounded-md p-0 text-start" onclick={() => onSelect('contracts')}>
					<span class="grid size-11 shrink-0 place-items-center rounded-lg bg-violet-50 text-violet-800"><FileText size={21} /></span>
					<span class="grid min-w-0 flex-1 gap-1"><strong class="text-sm">{m.dashboard_contracts_title()}</strong><small class="text-xs font-normal leading-relaxed text-slate-600">{m.dashboard_contracts_description()}</small></span><ArrowUpRight size={17} class="shrink-0 text-slate-400" />
				</Button>
			</Card>
			<Card class="rounded-lg bg-white p-4 shadow-sm ring-slate-200">
				<Button variant="ghost" class="h-auto w-full justify-start gap-4 whitespace-normal rounded-md p-0 text-start" onclick={() => onSelect('payments')}>
					<span class="grid size-11 shrink-0 place-items-center rounded-lg bg-amber-50 text-amber-800"><WalletCards size={21} /></span>
					<span class="grid min-w-0 flex-1 gap-1"><strong class="text-sm">{m.dashboard_payments_title()}</strong><small class="text-xs font-normal leading-relaxed text-slate-600">{m.dashboard_payments_description()}</small></span><ArrowUpRight size={17} class="shrink-0 text-slate-400" />
				</Button>
			</Card>
		</div>
	</div>

	<div class="grid gap-3 sm:grid-cols-2">
		<Card class="flex-row items-center gap-3 rounded-lg bg-white p-4 shadow-none ring-slate-200">
			<span class="grid size-10 shrink-0 place-items-center rounded-lg bg-emerald-50 text-emerald-800"><ShieldCheck size={20} /></span>
			<div class="grid gap-1"><strong class="text-sm text-slate-800">{m.setup_step_ready()}</strong><span class="text-xs text-slate-600">{m.auth_account_ready()}</span></div>
		</Card>
		<Card class="flex-row items-center gap-3 rounded-lg bg-white p-4 shadow-none ring-slate-200">
			<span class="grid size-10 shrink-0 place-items-center rounded-lg bg-slate-100 text-slate-700"><Building2 size={19} /></span>
			<div class="grid min-w-0 flex-1 gap-1"><strong class="text-sm text-slate-800">{m.settings_connection_title()}</strong><span class="text-xs text-slate-600">{mode === 'local' ? m.connection_local() : mode === 'direct' ? m.connection_direct() : m.connection_api()}</span></div>
			<Button variant="outline" class="shrink-0 rounded-md" onclick={onEditConnection}>{m.connection_edit()}</Button>
		</Card>
	</div>
</section>
