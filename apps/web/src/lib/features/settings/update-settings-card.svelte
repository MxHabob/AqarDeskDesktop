<script lang="ts">
	import { Download, RefreshCw } from '@lucide/svelte';
	import { Button, Card, CardDescription, CardHeader, CardTitle } from '@aqardesk/ui';
	import type { UpdateProgress, UpdateResult } from '#lib/api/desktop';
	import * as m from '#lib/paraglide/messages';

	type Props = {
		busy: boolean;
		result: UpdateResult | null;
		progress: UpdateProgress | null;
		progressPercent: number | null;
		onCheck: () => void;
		onDownload: () => void;
		onInstall: () => void;
	};

	let { busy, result, progress, progressPercent, onCheck, onDownload, onInstall }: Props = $props();
</script>

<Card class="rounded-lg bg-card p-5 shadow-card ring-border/50 lg:col-span-2">
	<CardHeader class="mb-4 p-0">
		<CardTitle class="text-base">{m.settings_update_title()}</CardTitle>
		<CardDescription>{m.settings_update_description()}</CardDescription>
	</CardHeader>
	<div class="grid gap-4 md:grid-cols-2">
		<div class="grid content-start gap-2">
			<div class="flex justify-between gap-3 border-b border-border py-2 text-sm">
				<span class="text-muted-foreground">{m.settings_current_version()}</span>
				<strong>{result?.current_version ?? '—'}</strong>
			</div>
			{#if result}
				<div class="flex justify-between gap-3 border-b border-border py-2 text-sm">
					<span class="text-muted-foreground">{m.settings_latest_version()}</span>
					<strong>{result.latest_version}</strong>
				</div>
				<p class="text-sm text-foreground">
					{result.has_update ? m.settings_update_available() : m.settings_up_to_date()}
				</p>
				{#if result.release_notes}
					<p class="max-h-24 overflow-y-auto text-xs leading-relaxed text-muted-foreground scrollbar-none">
						{m.settings_release_notes()}: {result.release_notes}
					</p>
				{/if}
			{/if}
		</div>
		<div class="grid content-start gap-3">
			{#if progress}
				<div class="flex justify-between text-xs text-muted-foreground">
					<span>{m.settings_download_progress()}</span>
					{#if progressPercent !== null}<strong>{progressPercent}%</strong>{/if}
				</div>
				<progress class="h-2 w-full accent-primary" value={progressPercent ?? 12} max="100">
					{progressPercent ?? 12}%
				</progress>
			{/if}
			<div class="flex flex-wrap gap-2">
				<Button variant="outline" class="rounded-md" disabled={busy} onclick={onCheck}>
					<RefreshCw size={15} />{m.settings_check_updates()}
				</Button>
				{#if result?.has_update}
					<Button class="rounded-md" disabled={busy} onclick={onDownload}>
						<Download size={15} />{m.settings_download_update()}
					</Button>
				{/if}
				{#if result?.has_update && progress && progressPercent === 100}
					<Button class="rounded-md" disabled={busy} onclick={onInstall}>
						{m.settings_install_update()}
					</Button>
				{/if}
			</div>
		</div>
	</div>
</Card>
