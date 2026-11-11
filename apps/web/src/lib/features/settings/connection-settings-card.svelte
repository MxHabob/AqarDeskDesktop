<script lang="ts">
	import { Database, Globe2, Server } from '@lucide/svelte';
	import { Button } from '@aqardesk/ui';
	import type { ConnectionMode } from '#lib/api/desktop';
	import SettingsFieldRow from '#lib/shared/components/settings-field-row.svelte';
	import * as m from '#lib/paraglide/messages';

	let {
		mode,
		connected,
		onEdit
	}: { mode: ConnectionMode; connected: boolean; onEdit: () => void } = $props();
</script>

<SettingsFieldRow
	label={m.settings_connection_title()}
	description={connected ? m.status_connected() : m.status_not_connected()}
>
	<div class="flex w-full items-center gap-3">
		<div class="flex min-w-0 items-center gap-2 text-sm">
			{#if mode === 'local'}
				<Database size={16} />
				<span class="truncate">{m.connection_local()}</span>
			{:else if mode === 'direct'}
				<Server size={16} />
				<span class="truncate">{m.connection_direct()}</span>
			{:else}
				<Globe2 size={16} />
				<span class="truncate">{m.connection_api()}</span>
			{/if}
		</div>
	</div>
	{#snippet action()}
		<Button type="button" variant="outline" class="shrink-0 rounded-md" onclick={onEdit}>
			<Database size={16} />{m.connection_edit()}
		</Button>
	{/snippet}
</SettingsFieldRow>
