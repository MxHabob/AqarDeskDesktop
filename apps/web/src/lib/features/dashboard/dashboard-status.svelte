<script lang="ts">
	import { Button } from '@aqardesk/ui';
	import type { ConnectionMode } from '#lib/api/desktop';
	import * as m from '#lib/paraglide/messages';

	let {
		connected,
		mode,
		onEditConnection
	}: { connected: boolean; mode: ConnectionMode; onEditConnection: () => void } = $props();
</script>

<div class="flex flex-wrap items-center justify-start gap-2 sm:justify-end">
	<span class="inline-flex h-8 items-center gap-2 rounded-md border border-border/60 px-2.5 text-xs text-muted-foreground">
		<span class={`size-1.5 rounded-full ${connected ? 'bg-emerald-500' : 'bg-destructive'}`}></span>
		{connected ? m.status_connected() : m.status_not_connected()}
	</span>
	<span class="inline-flex h-8 items-center rounded-md border border-border/60 px-2.5 text-xs text-muted-foreground">
		{mode === 'local' ? m.connection_local() : mode === 'direct' ? m.connection_direct() : m.connection_api()}
	</span>
	<Button variant="outline" size="sm" class="h-8 rounded-md" onclick={onEditConnection}>
		{m.connection_edit()}
	</Button>
</div>
