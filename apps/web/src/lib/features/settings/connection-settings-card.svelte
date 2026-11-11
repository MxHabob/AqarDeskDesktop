<script lang="ts">
	import { Database, Globe2, Server } from '@lucide/svelte';
	import { Button, Card, CardDescription, CardHeader, CardTitle } from '@aqardesk/ui';
	import type { ConnectionMode } from '#lib/api/desktop';
	import * as m from '#lib/paraglide/messages';

	let {
		mode,
		connected,
		onEdit
	}: { mode: ConnectionMode; connected: boolean; onEdit: () => void } = $props();
</script>

<Card class="rounded-lg bg-card p-5 shadow-card ring-border/50">
	<CardHeader class="mb-4 p-0">
		<CardTitle class="text-base">{m.settings_connection_title()}</CardTitle>
		<CardDescription>{m.settings_connection_title()}</CardDescription>
	</CardHeader>
	<div class="mb-4 flex items-center gap-3 rounded-lg border border-border p-3">
		<span class="grid size-10 place-items-center rounded-lg bg-accent text-accent-foreground">
			{#if mode === 'local'}<Database size={19} />{:else if mode === 'direct'}<Server size={19} />{:else}<Globe2 size={19} />{/if}
		</span>
		<div class="grid flex-1 gap-1">
			<strong class="text-sm text-foreground">
				{mode === 'local'
					? m.connection_local()
					: mode === 'direct'
						? m.connection_direct()
						: m.connection_api()}
			</strong>
			<span class="text-xs text-muted-foreground">
				{connected ? m.status_connected() : m.status_not_connected()}
			</span>
		</div>
		<span
			class="size-2 rounded-full"
			class:bg-primary-foreground={connected}
			class:bg-muted-foreground={!connected}
		></span>
	</div>
	<Button variant="outline" class="w-full rounded-md" onclick={onEdit}>
		<Database size={16} />{m.connection_edit()}
	</Button>
</Card>
