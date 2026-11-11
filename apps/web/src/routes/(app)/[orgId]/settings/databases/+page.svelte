<script lang="ts">
	import { onMount } from 'svelte';
	import { open, save } from '@tauri-apps/plugin-dialog';
	import { Archive, Database, Download, FolderCog, LoaderCircle, RefreshCw, RotateCcw, ShieldCheck } from '@lucide/svelte';
	import { Button, toast } from '@aqardesk/ui';
	import { desktopApi, type DatabaseAdminStatus, type DatabaseBackup } from '#lib/api/desktop';
	import { confirmStore } from '#lib/core/confirm.svelte';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import SettingsFieldRow from '#lib/shared/components/settings-field-row.svelte';
	import * as m from '#lib/paraglide/messages';

	const app = getAppContext();
	const sqliteFilters = [{ name: 'SQLite database', extensions: ['sqlite', 'sqlite3', 'db'] }];
	let desktopShell = $state(false);
	let loading = $state(true);
	let busy = $state(false);
	let loadError = $state(false);
	let status = $state<DatabaseAdminStatus | null>(null);
	let backups = $state<DatabaseBackup[]>([]);

	onMount(() => {
		desktopShell = '__TAURI_INTERNALS__' in window;
		void load();
	});

	async function load() {
		loading = true;
		loadError = false;
		try {
			if (!desktopShell) return;
			status = await desktopApi.getDatabaseAdminStatus();
			backups = status.supports_file_management ? await desktopApi.listDatabaseBackups() : [];
		} catch (cause) {
			loadError = true;
			toast.error(cause instanceof Error ? cause.message : String(cause));
		} finally {
			loading = false;
		}
	}

	async function moveDatabase() {
		const currentPath = status?.database_path;
		if (!currentPath) return;
		const currentName = currentPath.split(/[\\/]/).at(-1)?.replace(/\.(sqlite3?|db)$/i, '') ?? 'aqardesk';
		const destination = await choosePath(() =>
			save({
				title: m.database_choose_location(),
				defaultPath: siblingPath(currentPath, `${currentName}-moved.sqlite`),
				filters: sqliteFilters
			})
		);
		if (!destination) return;

		await runOperation(async () => {
			const result = await desktopApi.changeDatabaseLocation(destination);
			app.databasePath = result.path;
			toast.success(m.database_move_success());
		});
	}

	async function createBackup() {
		const currentPath = status?.database_path;
		if (!currentPath) return;
		const currentName = currentPath.split(/[\\/]/).at(-1)?.replace(/\.(sqlite3?|db)$/i, '') ?? 'aqardesk';
		const destination = await choosePath(() =>
			save({
				title: m.database_choose_backup_location(),
				defaultPath: siblingPath(
					currentPath,
					`${currentName}-backup-${new Date().toISOString().slice(0, 10)}.sqlite`
				),
				filters: sqliteFilters
			})
		);
		if (!destination) return;

		await runOperation(async () => {
			await desktopApi.createDatabaseBackup(destination);
			toast.success(m.database_backup_success());
			await refreshBackups();
		});
	}

	async function chooseBackup() {
		const selected = await choosePath(() =>
			open({
				title: m.database_choose_backup(),
				multiple: false,
				directory: false,
				filters: sqliteFilters
			})
		);
		if (selected) await restoreBackup(selected);
	}

	async function restoreBackup(path: string) {
		const confirmed = await confirmStore.prompt({
			title: m.database_restore_confirm_title(),
			description: m.database_restore_confirm_description(),
			confirmText: m.database_restore_confirm(),
			cancelText: m.common_cancel(),
			variant: 'destructive'
		});
		if (!confirmed) return;

		await runOperation(async () => {
			await desktopApi.restoreDatabaseBackup(path);
			if (!(await app.signOut())) window.location.reload();
			toast.success(m.database_restore_success());
		}, false);
	}

	async function checkIntegrity() {
		await runOperation(async () => {
			await desktopApi.checkDatabaseIntegrity();
			toast.success(m.database_integrity_success());
		});
	}

	async function runOperation(operation: () => Promise<void>, refresh = true) {
		if (busy) return;
		busy = true;
		try {
			await operation();
			if (refresh && status?.supports_file_management) {
				status = await desktopApi.getDatabaseAdminStatus();
				backups = await desktopApi.listDatabaseBackups();
			}
		} catch (cause) {
			toast.error(cause instanceof Error ? cause.message : String(cause));
		} finally {
			busy = false;
		}
	}

	async function choosePath(
		picker: () => Promise<string | string[] | null>
	): Promise<string | null> {
		try {
			const path = await picker();
			return typeof path === 'string' ? path : null;
		} catch (cause) {
			toast.error(cause instanceof Error ? cause.message : String(cause));
			return null;
		}
	}

	async function refreshBackups() {
		backups = await desktopApi.listDatabaseBackups();
	}

	function formatSize(bytes: number) {
		if (bytes < 1024) return m.database_size_bytes({ size: String(bytes) });
		const units = ['KB', 'MB', 'GB', 'TB'];
		let size = bytes / 1024;
		let unit = 0;
		while (size >= 1024 && unit < units.length - 1) {
			size /= 1024;
			unit += 1;
		}
		return `${new Intl.NumberFormat(app.locale, { maximumFractionDigits: 1 }).format(size)} ${units[unit]}`;
	}

	function formatTimestamp(value: number) {
		return new Intl.DateTimeFormat(app.locale, {
			dateStyle: 'medium',
			timeStyle: 'short',
			timeZone: app.organizationTimezone
		}).format(new Date(value * 1000));
	}

	function siblingPath(path: string, name: string) {
		const separatorIndex = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'));
		if (separatorIndex < 0) return name;
		return `${path.slice(0, separatorIndex + 1)}${name}`;
	}

	const modeLabel = $derived(
		status?.mode === 'local'
			? m.database_connection_local()
			: status?.mode === 'direct'
				? m.database_connection_direct()
				: m.database_connection_api()
	);
</script>

<header class="grid gap-1">
	<h2 class="text-lg font-semibold">{m.database_management_title()}</h2>
	<p class="text-sm text-muted-foreground">{m.database_management_description()}</p>
</header>

{#if !desktopShell}
	<div class="rounded-lg border border-border/70 bg-card p-4 text-sm text-muted-foreground" role="status">
		{m.database_native_dialog_notice()}
	</div>
{:else if loading}
	<div class="flex items-center gap-2 py-8 text-sm text-muted-foreground" role="status">
		<LoaderCircle size={16} class="animate-spin" />{m.common_processing()}
	</div>
{:else if loadError}
	<div class="flex items-center justify-between gap-3 rounded-lg border border-border/70 bg-card p-4">
		<p class="text-sm text-muted-foreground">{m.database_management_load_error()}</p>
		<Button variant="outline" onclick={() => void load()}><RefreshCw size={15} />{m.common_retry()}</Button>
	</div>
{:else if status}
	<div class="grid gap-5">
		<section class="overflow-hidden rounded-lg border border-border/70 bg-card">
			<h3 class="border-b border-border/70 bg-muted/40 px-4 py-3 text-xs font-medium text-muted-foreground">
				{m.database_management_title()}
			</h3>
			<div class="divide-y divide-border/70">
				<SettingsFieldRow label={m.database_connection_mode()}>
					<div class="flex items-center gap-2 text-sm">
						<span class={`size-2 rounded-full ${app.connected ? 'bg-emerald-500' : 'bg-destructive'}`}></span>
						<span>{modeLabel}</span>
						<span class="text-xs text-muted-foreground">
							{app.connected ? m.database_connection_connected() : m.database_connection_disconnected()}
						</span>
					</div>
				</SettingsFieldRow>
				{#if status.supports_file_management}
					<SettingsFieldRow
						label={m.database_file_location()}
						copyValue={status.database_path ?? undefined}
					>
						<p class="break-all font-mono text-xs" dir="ltr">{status.database_path}</p>
					</SettingsFieldRow>
					<SettingsFieldRow label={m.database_file_size()}>
						<p class="text-sm sm:max-w-2xl">
							{status.file_size_bytes === null ? m.database_file_missing() : formatSize(status.file_size_bytes)}
						</p>
					</SettingsFieldRow>
					<SettingsFieldRow label={m.database_last_modified()}>
						<p class="text-sm sm:max-w-2xl">
							{status.modified_at_unix === null ? '—' : formatTimestamp(status.modified_at_unix)}
						</p>
					</SettingsFieldRow>
				{:else}
					<SettingsFieldRow
						label={m.database_remote_name()}
						copyValue={status.remote_database ?? undefined}
					>
						<p class="break-all font-mono text-xs" dir="ltr">{status.remote_database}</p>
					</SettingsFieldRow>
				{/if}
			</div>
		</section>

		{#if status.supports_file_management}
			<section class="overflow-hidden rounded-lg border border-border/70 bg-card">
				<h3 class="border-b border-border/70 bg-muted/40 px-4 py-3 text-xs font-medium text-muted-foreground">
					{m.database_actions()}
				</h3>
				<div class="divide-y divide-border/70">
					<SettingsFieldRow
						label={m.database_change_location()}
						description={m.database_change_location_description()}
					>
						{#snippet action()}
							<Button variant="outline" disabled={busy || status?.file_size_bytes === null} onclick={() => void moveDatabase()}>
								<FolderCog size={15} />{m.database_change_location()}
							</Button>
						{/snippet}
					</SettingsFieldRow>
					<SettingsFieldRow
						label={m.database_create_backup()}
						description={m.database_create_backup_description()}
					>
						{#snippet action()}
							<Button variant="outline" disabled={busy || !app.connected} onclick={() => void createBackup()}>
								<Download size={15} />{m.database_create_backup()}
							</Button>
						{/snippet}
					</SettingsFieldRow>
					<SettingsFieldRow
						label={m.database_restore_backup()}
						description={m.database_restore_backup_description()}
					>
						{#snippet action()}
							<Button variant="outline" disabled={busy} onclick={() => void chooseBackup()}>
								<RotateCcw size={15} />{m.database_restore_backup()}
							</Button>
						{/snippet}
					</SettingsFieldRow>
					<SettingsFieldRow
						label={m.database_check_integrity()}
						description={m.database_check_integrity_description()}
					>
						{#snippet action()}
							<Button variant="outline" disabled={busy || !app.connected} onclick={() => void checkIntegrity()}>
								<ShieldCheck size={15} />{m.database_check_integrity()}
							</Button>
						{/snippet}
					</SettingsFieldRow>
				</div>
				{#if busy}
					<p class="border-t border-border/70 px-4 py-3 text-xs text-muted-foreground" role="status">
						{m.database_operation_running()}
					</p>
				{/if}
			</section>

			<section class="overflow-hidden rounded-lg border border-border/70 bg-card">
				<div class="flex items-center justify-between gap-3 border-b border-border/70 bg-muted/40 px-4 py-3">
					<div class="grid gap-1">
						<h3 class="text-xs font-medium text-muted-foreground">{m.database_backups_title()}</h3>
						<p class="text-xs text-muted-foreground">{m.database_backups_description()}</p>
					</div>
					<Archive size={16} class="shrink-0 text-muted-foreground" />
				</div>
				{#if backups.length === 0}
					<p class="px-4 py-5 text-sm text-muted-foreground">{m.database_backups_empty()}</p>
				{:else}
					<div class="divide-y divide-border/70">
						{#each backups as backup (backup.path)}
							<div class="grid gap-3 px-4 py-3 sm:grid-cols-[minmax(0,1fr)_auto] sm:items-center">
								<div class="grid min-w-0 gap-1">
									<p class="truncate text-sm font-medium" title={backup.name}>{backup.name}</p>
									<p class="text-xs text-muted-foreground">
										{formatTimestamp(backup.modified_at_unix)} · {formatSize(backup.size_bytes)}
									</p>
								</div>
								<Button
									variant="outline"
									size="sm"
									disabled={busy}
									onclick={() => void restoreBackup(backup.path)}
								>
									<RotateCcw size={14} />{m.database_restore_from_list()}
								</Button>
							</div>
						{/each}
					</div>
				{/if}
			</section>
		{:else}
			<div class="flex items-start gap-3 rounded-lg border border-border/70 bg-card p-4 text-sm text-muted-foreground" role="note">
				<Database size={17} class="mt-0.5 shrink-0" />
				<p>{m.database_remote_notice()}</p>
			</div>
		{/if}

		<p class="text-xs text-muted-foreground">{m.database_existing_file_notice()}</p>
	</div>
{/if}
