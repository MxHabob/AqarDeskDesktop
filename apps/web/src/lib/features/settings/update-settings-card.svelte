<script lang="ts">
	import { Download, RefreshCw } from '@lucide/svelte';
	import { Button } from '@aqardesk/ui';
	import { formatNumber } from '#lib/core/i18n';
	import type { UpdateProgress, UpdateResult } from '#lib/api/desktop';
	import SettingsFieldRow from '#lib/shared/components/settings-field-row.svelte';
	import * as m from '#lib/paraglide/messages';

	type Props = {
		busy: boolean;
		result: UpdateResult | null;
		progress: UpdateProgress | null;
		progressPercent: number | null;
		error: string | null;
		downloading: boolean;
		ready: boolean;
		onCheck: () => void;
		onDownload: () => void;
		onInstall: () => void;
	};

	let {
		busy,
		result,
		progress,
		progressPercent,
		error,
		downloading,
		ready,
		onCheck,
		onDownload,
		onInstall
	}: Props = $props();
</script>

<SettingsFieldRow label={m.settings_update_title()} description={m.settings_update_description()}>
	<div class="grid w-full gap-3 sm:max-w-xl">
		<div class="flex flex-wrap items-center justify-between gap-2 text-sm">
			<span class="text-muted-foreground">{m.settings_current_version()}</span>
			<strong>{result?.current_version ?? '—'}</strong>
			{#if result}
				<span class="text-muted-foreground">{m.settings_latest_version()}</span>
				<strong>{result.latest_version}</strong>
			{/if}
		</div>
		{#if result}
			<p class="text-sm text-foreground">
				{result.has_update ? m.settings_update_available() : m.settings_up_to_date()}
			</p>
			{#if result.release_notes}
				<p class="max-h-24 overflow-y-auto text-xs leading-relaxed text-muted-foreground scrollbar-none">
					{m.settings_release_notes()}: {result.release_notes}
				</p>
			{/if}
		{/if}
		{#if error}
			<p class="text-sm text-destructive" role="alert">{error}</p>
		{/if}
		{#if downloading}
			<p class="text-xs text-muted-foreground" role="status" aria-live="polite">
				{m.download_manager_downloading()}
			</p>
		{/if}
		{#if progress}
			<div class="flex justify-between text-xs text-muted-foreground">
				<span>{m.settings_download_progress()}</span>
				{#if progressPercent !== null}<strong>{formatNumber(progressPercent, { maximumFractionDigits: 0 })}%</strong>{/if}
			</div>
			<progress class="h-2 w-full accent-primary" value={progressPercent ?? 12} max="100">
				{progressPercent ?? 12}%
			</progress>
		{/if}
		<div class="flex flex-wrap gap-2">
			<Button variant="outline" class="rounded-md" disabled={busy} onclick={onCheck}>
				<RefreshCw size={15} />{m.settings_check_updates()}
			</Button>
			{#if result?.has_update && !ready}
				<Button class="rounded-md" disabled={busy} onclick={onDownload}>
					<Download size={15} />{m.settings_download_update()}
				</Button>
			{/if}
			{#if result?.has_update && ready}
				<Button class="rounded-md" disabled={busy} onclick={onInstall}>
					{m.settings_install_update()}
				</Button>
			{/if}
		</div>
	</div>
</SettingsFieldRow>
