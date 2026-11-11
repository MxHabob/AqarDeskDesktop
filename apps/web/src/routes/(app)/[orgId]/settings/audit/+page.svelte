<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from '@aqardesk/ui';
	import { OrbisGrid, type GridActionEvent, type OrbisColumnDef } from '@aqardesk/svelte-grid';
	import type { AuditEntry } from '@aqardesk/sdk';
	import SettingsDetailsDialog from '#lib/features/settings/settings-details-dialog.svelte';
	import { getLocale } from '#lib/paraglide/runtime';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { formatDate, formatGridCell } from '#lib/core/i18n';
	import { desktopApi } from '#lib/api/desktop';
	import * as m from '#lib/paraglide/messages';

	const pageSize = 50;
	const app = getAppContext();
	let entries = $state<AuditEntry[]>([]);
	let loading = $state(true);
	let hasMore = $state(false);
	let selectedEntry = $state<AuditEntry | null>(null);
	let detailsOpen = $state(false);
	const rtl = $derived(getLocale() === 'ar');
	const detailFields = $derived(
		selectedEntry
			? [
					{
						label: m.settings_audit_actor(),
						value: selectedEntry.actor_name ?? selectedEntry.actor_user_id ?? m.settings_audit_actor()
					},
					{ label: m.settings_audit_action(), value: selectedEntry.action },
					{ label: m.settings_audit_entity(), value: selectedEntry.entity_type },
					{ label: m.settings_audit_entity_id(), value: selectedEntry.entity_id, direction: 'ltr' as const },
					{ label: m.settings_audit_time(), value: formatTimestamp(selectedEntry.created_at) },
					...(selectedEntry.details_json
						? [
								{
									label: m.settings_audit_details(),
									value: selectedEntry.details_json,
									direction: 'ltr' as const,
									preformatted: true
								}
							]
						: [])
				]
			: []
	);
	const columns: OrbisColumnDef[] = $derived([
		{
			id: 'actor_name',
			header: m.settings_audit_actor(),
			width: 180,
			accessor: (row) => row.actor_name ?? row.actor_user_id ?? m.settings_audit_actor()
		},
		{ id: 'action', header: m.settings_audit_action(), width: 180 },
		{ id: 'entity_type', header: m.settings_audit_entity(), width: 140 },
		{ id: 'entity_id', header: m.settings_audit_entity_id(), width: 220 },
		{ id: 'created_at', header: m.settings_audit_time(), type: 'datetime', width: 200 }
	]);

	onMount(() => {
		void load(false);
	});

	async function load(more: boolean) {
		if (!more) loading = true;
		try {
			const next = await desktopApi.listAuditEvents(pageSize, more ? entries.length : 0);
			entries = more ? [...entries, ...next] : next;
			hasMore = next.length === pageSize;
		} catch (cause) {
			toast.error(cause instanceof Error ? cause.message : String(cause));
			if (more) throw cause;
		} finally {
			loading = false;
		}
	}

	function formatTimestamp(value: string) {
		return formatDate(value, 'datetime', undefined, app.organizationTimezone) || value;
	}

	function handleGridAction(event: GridActionEvent) {
		if (event.actionId !== 'dblclick') return;
		const entry = entries.find((item) => item.id === String(event.row.id ?? ''));
		if (!entry) return;
		selectedEntry = entry;
		detailsOpen = true;
	}
</script>

<div class="min-h-0 min-w-0 flex-1">
	<OrbisGrid
		{columns}
		source={entries}
		{loading}
		{rtl}
		formatCellValue={(row, column) => formatGridCell(row, column, app.organizationTimezone)}
		readonly
		rowIdKey="id"
		height="100%"
		{hasMore}
		onloadmore={() => load(true)}
		emptyTitle={m.settings_audit_empty()}
		onaction={handleGridAction}
	/>
</div>

<SettingsDetailsDialog
	bind:open={detailsOpen}
	title={m.settings_details_title()}
	fields={detailFields}
/>
