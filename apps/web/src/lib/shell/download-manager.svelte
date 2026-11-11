<script lang="ts">
	import { onMount } from 'svelte';
	import { Popover } from 'bits-ui';
	import { Download, LoaderCircle, RefreshCw } from '@lucide/svelte';
	import { Button } from '@aqardesk/ui';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { formatNumber } from '#lib/core/i18n';
	import * as m from '#lib/paraglide/messages';

	const app = getAppContext();
	let open = $state(false);
	const isBusy = $derived(app.updateBusy);
	const shouldShow = $derived(
		isBusy ||
			app.updateError !== null ||
			app.isUpdateReady ||
			app.updateResult?.has_update === true
	);
	const progressPercent = $derived(
		app.updateProgress?.total_bytes
			? Math.min(
					100,
					Math.round(
						(app.updateProgress.downloaded_bytes / app.updateProgress.total_bytes) * 100
					)
				)
			: null
	);

	onMount(() => {
		void app.initializeUpdates();
	});

	function formatMegabytes(bytes: number) {
		return `${formatNumber(bytes / (1024 * 1024), {
			minimumFractionDigits: 1,
			maximumFractionDigits: 1
		})} MB`;
	}
</script>

{#if shouldShow}
	<Popover.Root bind:open>
		<Popover.Trigger>
			{#snippet child({ props })}
				<Button
					{...props}
					type="button"
					variant="outline"
					size="icon"
					class="relative size-8 shrink-0 rounded-lg"
					title={m.settings_update_title()}
					aria-label={m.settings_update_title()}
					aria-busy={isBusy}
				>
					{#if app.isCheckingUpdates}
						<RefreshCw size={14} class="animate-spin text-muted-foreground" />
					{:else if app.isDownloadingUpdate || app.isInstallingUpdate}
						<LoaderCircle size={14} class="animate-spin text-primary" />
					{:else}
						<Download size={14} class={app.updateError ? 'text-destructive' : 'text-muted-foreground'} />
					{/if}
					{#if app.updateResult?.has_update || app.updateError || app.isUpdateReady}
						<span
							class="absolute -end-0.5 -top-0.5 size-2 rounded-full bg-primary ring-2 ring-card"
							class:bg-destructive={app.updateError !== null}
							aria-hidden="true"
						></span>
					{/if}
				</Button>
			{/snippet}
		</Popover.Trigger>
		<Popover.Content
			side="bottom"
			align="end"
			sideOffset={8}
			class="z-[var(--layer-popover)] w-[min(360px,calc(100vw-2rem))] space-y-4 rounded-xl border border-border bg-popover p-4 text-start text-popover-foreground shadow-floating"
		>
			<div class="flex items-center justify-between gap-3">
				<h2 class="text-sm font-semibold">{m.settings_update_title()}</h2>
				<Button
					variant="ghost"
					size="icon"
					class="size-7"
					title={m.settings_check_updates()}
					aria-label={m.settings_check_updates()}
					disabled={isBusy}
					onclick={() => void app.checkForUpdates()}
				>
					<RefreshCw size={14} class={app.isCheckingUpdates ? 'animate-spin' : ''} />
				</Button>
			</div>

			{#if app.isCheckingUpdates}
				<p class="text-xs text-muted-foreground" role="status" aria-live="polite">
					{m.download_manager_checking()}
				</p>
			{:else if app.isDownloadingUpdate}
				<div class="space-y-2" role="status" aria-live="polite">
					<div class="flex items-center justify-between gap-3 text-xs">
						<span>{m.download_manager_downloading()}</span>
						{#if app.updateProgress}
							<span class="font-mono text-muted-foreground">
								{#if app.updateProgress.total_bytes}
									{formatNumber(progressPercent ?? 0, { maximumFractionDigits: 0 })}%
								{:else}
									{formatMegabytes(app.updateProgress.downloaded_bytes)}
								{/if}
							</span>
						{/if}
					</div>
					<div
						class="h-1.5 w-full overflow-hidden rounded-full bg-muted"
						role="progressbar"
						aria-label={m.settings_download_progress()}
						aria-valuemin="0"
						aria-valuemax="100"
						aria-valuenow={progressPercent ?? undefined}
					>
						<div
							class="h-full rounded-full bg-primary transition-[width]"
							class:animate-download-progress={progressPercent === null}
							style:width={progressPercent === null ? '35%' : `${progressPercent}%`}
						></div>
					</div>
				</div>
			{:else if app.isInstallingUpdate}
				<p class="text-xs text-muted-foreground" role="status" aria-live="polite">
					{m.download_manager_installing()}
				</p>
			{:else if app.updateError}
				<div class="space-y-3">
					<p class="break-words text-xs text-destructive" role="alert">
						{m.download_manager_failed()} {app.updateError}
					</p>
					<Button
						variant="outline"
						size="sm"
						class="w-full"
						disabled={isBusy}
						onclick={() => void app.retryUpdate()}
					>
						{m.common_retry()}
					</Button>
				</div>
			{:else if app.updateResult?.has_update}
				<div class="space-y-3">
					<div class="grid gap-1">
						<p class="text-xs text-foreground">
							{m.settings_update_available()} · {app.updateResult.latest_version}
						</p>
						{#if app.updateResult.release_notes}
							<p class="max-h-24 overflow-y-auto whitespace-pre-wrap text-xs leading-relaxed text-muted-foreground scrollbar-none">
								{app.updateResult.release_notes}
							</p>
						{/if}
					</div>
					{#if app.isUpdateReady}
						<p class="text-xs text-muted-foreground" role="status" aria-live="polite">
							{m.download_manager_ready()}
						</p>
						<Button
							size="sm"
							class="w-full"
							disabled={isBusy}
							onclick={() => void app.installUpdate()}
						>
							{m.settings_install_update()}
						</Button>
					{:else}
						<Button
							size="sm"
							class="w-full"
							disabled={isBusy}
							onclick={() => void app.downloadUpdate()}
						>
							<Download size={14} />
							{m.settings_download_update()}
						</Button>
					{/if}
				</div>
			{/if}
		</Popover.Content>
	</Popover.Root>
{/if}

<style>
	:global(.animate-download-progress) {
		animation: download-progress-slide 1.4s ease-in-out infinite;
	}

	@keyframes download-progress-slide {
		from {
			transform: translateX(-100%);
		}

		to {
			transform: translateX(300%);
		}
	}
</style>
