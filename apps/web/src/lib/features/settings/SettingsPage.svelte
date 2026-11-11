<script lang="ts">
	import { Check, Database, Download, Globe2, RefreshCw, Server } from '@lucide/svelte';
	import { Button, Card, CardDescription, CardHeader, CardTitle } from '@aqardesk/ui';
	import type { ConnectionMode, UpdateProgress, UpdateResult } from '#lib/api/desktop';
	import type { AppLocale } from '#lib/core/i18n';
	import PageHeader from '#lib/shared/components/PageHeader.svelte';
	import MessageBanner from '#lib/shared/components/MessageBanner.svelte';
	import * as m from '#lib/paraglide/messages';

	type Props = {
		locale: AppLocale;
		mode: ConnectionMode;
		connected: boolean;
		busy: boolean;
		error: string;
		notice: string;
		updateResult: UpdateResult | null;
		updateProgress: UpdateProgress | null;
		progressPercent: number | null;
		onLanguageChange: (locale: AppLocale) => void;
		onEditConnection: () => void;
		onCheckUpdates: () => void;
		onDownloadUpdate: () => void;
		onInstallUpdate: () => void;
	};

	let {
		locale,
		mode,
		connected,
		busy,
		error,
		notice,
		updateResult,
		updateProgress,
		progressPercent,
		onLanguageChange,
		onEditConnection,
		onCheckUpdates,
		onDownloadUpdate,
		onInstallUpdate
	}: Props = $props();
</script>

<PageHeader title={m.settings_title()} description={m.settings_description()} eyebrow={m.nav_workspace()} />
{#if error}<MessageBanner message={error} />{/if}
{#if notice}<MessageBanner message={notice} variant="success" />{/if}

<div class="grid items-start gap-4 lg:grid-cols-2">
	<Card class="rounded-lg bg-white p-5 shadow-sm ring-slate-200">
		<CardHeader class="mb-4 p-0"><CardTitle class="text-base">{m.common_language()}</CardTitle><CardDescription>{m.common_language()}</CardDescription></CardHeader>
		<div class="grid gap-2">
			<Button variant="outline" class={locale === 'ar' ? 'h-12 justify-start rounded-md border-emerald-700 bg-emerald-50 text-emerald-950' : 'h-12 justify-start rounded-md'} onclick={() => onLanguageChange('ar')}>
				<span class="grid size-8 place-items-center rounded-md bg-white text-sm">ع</span><span class="flex-1 text-start">{m.language_arabic()}</span>{#if locale === 'ar'}<Check size={17} />{/if}
			</Button>
			<Button variant="outline" class={locale === 'en' ? 'h-12 justify-start rounded-md border-emerald-700 bg-emerald-50 text-emerald-950' : 'h-12 justify-start rounded-md'} onclick={() => onLanguageChange('en')}>
				<span class="grid size-8 place-items-center rounded-md bg-white text-xs">EN</span><span class="flex-1 text-start">{m.language_english()}</span>{#if locale === 'en'}<Check size={17} />{/if}
			</Button>
		</div>
	</Card>

	<Card class="rounded-lg bg-white p-5 shadow-sm ring-slate-200">
		<CardHeader class="mb-4 p-0"><CardTitle class="text-base">{m.settings_connection_title()}</CardTitle><CardDescription>{m.settings_connection_title()}</CardDescription></CardHeader>
		<div class="mb-4 flex items-center gap-3 rounded-lg border border-slate-200 p-3">
			<span class="grid size-10 place-items-center rounded-lg bg-emerald-50 text-emerald-800">
				{#if mode === 'local'}<Database size={19} />{:else if mode === 'direct'}<Server size={19} />{:else}<Globe2 size={19} />{/if}
			</span>
			<div class="grid flex-1 gap-1"><strong class="text-sm text-slate-800">{mode === 'local' ? m.connection_local() : mode === 'direct' ? m.connection_direct() : m.connection_api()}</strong><span class="text-xs text-slate-500">{connected ? m.status_connected() : m.status_not_connected()}</span></div>
			<span class="size-2 rounded-full" class:bg-emerald-500={connected} class:bg-slate-400={!connected}></span>
		</div>
		<Button variant="outline" class="w-full rounded-md" onclick={onEditConnection}><Database size={16} />{m.connection_edit()}</Button>
	</Card>

	<Card class="rounded-lg bg-white p-5 shadow-sm ring-slate-200 lg:col-span-2">
		<CardHeader class="mb-4 p-0"><CardTitle class="text-base">{m.settings_update_title()}</CardTitle><CardDescription>{m.settings_update_description()}</CardDescription></CardHeader>
		<div class="grid gap-4 md:grid-cols-2">
			<div class="grid content-start gap-2">
				<div class="flex justify-between gap-3 border-b border-slate-100 py-2 text-sm"><span class="text-slate-600">{m.settings_current_version()}</span><strong>{updateResult?.current_version ?? '—'}</strong></div>
				{#if updateResult}
					<div class="flex justify-between gap-3 border-b border-slate-100 py-2 text-sm"><span class="text-slate-600">{m.settings_latest_version()}</span><strong>{updateResult.latest_version}</strong></div>
					<p class="text-sm text-emerald-800">{updateResult.has_update ? m.settings_update_available() : m.settings_up_to_date()}</p>
					{#if updateResult.release_notes}<p class="max-h-24 overflow-y-auto text-xs leading-relaxed text-slate-600 scrollbar-none">{m.settings_release_notes()}: {updateResult.release_notes}</p>{/if}
				{/if}
			</div>
			<div class="grid content-start gap-3">
				{#if updateProgress}
					<div class="flex justify-between text-xs text-slate-600"><span>{m.settings_download_progress()}</span>{#if progressPercent !== null}<strong>{progressPercent}%</strong>{/if}</div>
					<progress class="h-2 w-full accent-emerald-700" value={progressPercent ?? 12} max="100">{progressPercent ?? 12}%</progress>
				{/if}
				<div class="flex flex-wrap gap-2">
					<Button variant="outline" class="rounded-md" disabled={busy} onclick={onCheckUpdates}><RefreshCw size={15} />{m.settings_check_updates()}</Button>
					{#if updateResult?.has_update}
						<Button class="rounded-md bg-emerald-800 text-white hover:bg-emerald-900" disabled={busy} onclick={onDownloadUpdate}><Download size={15} />{m.settings_download_update()}</Button>
					{/if}
					{#if updateResult?.has_update && updateProgress && progressPercent === 100}
						<Button class="rounded-md bg-emerald-800 text-white hover:bg-emerald-900" disabled={busy} onclick={onInstallUpdate}>{m.settings_install_update()}</Button>
					{/if}
				</div>
			</div>
		</div>
	</Card>
</div>
