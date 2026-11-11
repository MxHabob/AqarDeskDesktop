<script lang="ts">
	import { Building2, ShieldCheck } from '@lucide/svelte';
	import { Button } from '@aqardesk/ui';
	import type { ConnectionMode } from '#lib/api/desktop';
	import * as m from '#lib/paraglide/messages';

	let { mode, onEditConnection }: { mode: ConnectionMode; onEditConnection: () => void } = $props();
</script>

<section class="flex flex-col gap-3 rounded-xl border border-border/70 bg-card px-4 py-3 shadow-card sm:flex-row sm:items-center">
	<div class="flex min-w-0 flex-1 items-center gap-3">
		<span class="grid size-9 shrink-0 place-items-center rounded-lg bg-emerald-500/10 text-emerald-700 dark:text-emerald-400">
			<ShieldCheck size={18} />
		</span>
		<div class="min-w-0">
			<strong class="block text-xs font-semibold text-foreground">{m.setup_step_ready()}</strong>
			<span class="text-[11px] text-muted-foreground">{m.auth_account_ready()}</span>
		</div>
	</div>
	<div class="flex min-w-0 items-center gap-3 border-t border-border/70 pt-3 sm:border-s sm:border-t-0 sm:pt-0 sm:ps-4">
		<span class="grid size-9 shrink-0 place-items-center rounded-lg bg-secondary text-secondary-foreground">
			<Building2 size={17} />
		</span>
		<div class="min-w-0 flex-1">
			<strong class="block text-xs font-semibold text-foreground">{m.settings_connection_title()}</strong>
			<span class="text-[11px] text-muted-foreground">
				{mode === 'local' ? m.connection_local() : mode === 'direct' ? m.connection_direct() : m.connection_api()}
			</span>
		</div>
		<Button variant="outline" size="sm" class="shrink-0 rounded-lg" onclick={onEditConnection}>
			{m.connection_edit()}
		</Button>
	</div>
</section>
