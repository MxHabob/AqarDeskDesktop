<script lang="ts">
	import { Building2, ShieldCheck } from '@lucide/svelte';
	import { Button, Card } from '@aqardesk/ui';
	import type { ConnectionMode } from '#lib/api/desktop';
	import * as m from '#lib/paraglide/messages';

	let { mode, onEditConnection }: { mode: ConnectionMode; onEditConnection: () => void } = $props();
</script>

<div class="grid gap-3 sm:grid-cols-2">
	<Card class="flex-row items-center gap-3 rounded-lg bg-card p-4 shadow-card ring-border/50">
		<span class="grid size-10 shrink-0 place-items-center rounded-lg bg-accent text-accent-foreground">
			<ShieldCheck size={20} />
		</span>
		<div class="grid gap-1">
			<strong class="text-sm text-foreground">{m.setup_step_ready()}</strong>
			<span class="text-xs text-muted-foreground">{m.auth_account_ready()}</span>
		</div>
	</Card>
	<Card class="flex-row items-center gap-3 rounded-lg bg-card p-4 shadow-card ring-border/50">
		<span class="grid size-10 shrink-0 place-items-center rounded-lg bg-secondary text-secondary-foreground">
			<Building2 size={19} />
		</span>
		<div class="grid min-w-0 flex-1 gap-1">
			<strong class="text-sm text-foreground">{m.settings_connection_title()}</strong>
			<span class="text-xs text-muted-foreground">
				{mode === 'local' ? m.connection_local() : mode === 'direct' ? m.connection_direct() : m.connection_api()}
			</span>
		</div>
		<Button variant="outline" class="shrink-0 rounded-md" onclick={onEditConnection}>
			{m.connection_edit()}
		</Button>
	</Card>
</div>
