<script lang="ts">
	import { ListFilter, Settings2 } from '@lucide/svelte';
	import { Button } from '@aqardesk/ui';
	import * as m from '#lib/paraglide/messages';

	let {
		months,
		view,
		onMonthsChange,
		onViewChange,
		onSettings
	}: {
		months: number;
		view: 'overview' | 'metrics';
		onMonthsChange: (months: number) => void;
		onViewChange: (view: 'overview' | 'metrics') => void;
		onSettings: () => void;
	} = $props();
</script>

<div class="flex shrink-0 items-center gap-1.5">
	<Button
		variant="outline"
		size="icon"
		class="size-7 rounded-sm border-border/70"
		aria-label={m.nav_settings()}
		title={m.nav_settings()}
		onclick={onSettings}
	>
		<Settings2 size={13} />
	</Button>
	<label class="relative">
		<span class="sr-only">{m.dashboard_period()}</span>
		<select
			class="h-7 rounded-sm border border-border/70 bg-transparent pe-2 ps-7 text-xs text-foreground"
			value={months}
			onchange={(event) => onMonthsChange(Number(event.currentTarget.value))}
		>
			<option value={3}>{m.dashboard_period_3_months()}</option>
			<option value={6}>{m.dashboard_period_6_months()}</option>
			<option value={12}>{m.dashboard_period_12_months()}</option>
		</select>
		<span class="pointer-events-none absolute start-2 top-1/2 -translate-y-1/2 text-muted-foreground" aria-hidden="true">
			<ListFilter size={12} />
		</span>
	</label>
	<div class="flex h-7 border border-border/70">
		<Button
			variant="ghost"
			class={`h-full rounded-none px-3 text-xs ${view === 'overview' ? 'bg-muted text-foreground' : 'text-muted-foreground'}`}
			aria-pressed={view === 'overview'}
			onclick={() => onViewChange('overview')}
		>
			{m.dashboard_overview()}
		</Button>
		<Button
			variant="ghost"
			class={`h-full rounded-none border-s border-border/70 px-3 text-xs ${view === 'metrics' ? 'bg-muted text-foreground' : 'text-muted-foreground'}`}
			aria-pressed={view === 'metrics'}
			onclick={() => onViewChange('metrics')}
		>
			{m.dashboard_metrics()}
		</Button>
	</div>
</div>
