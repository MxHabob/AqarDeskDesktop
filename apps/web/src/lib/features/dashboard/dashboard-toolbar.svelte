<script lang="ts">
	import { ListFilter, Settings2 } from '@lucide/svelte';
	import {
		Button,
		Select,
		SelectContent,
		SelectItem,
		SelectTrigger,
		SelectValue
	} from '@aqardesk/ui';
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
	<Select
		type="single"
		value={String(months)}
		items={[
			{ value: '3', label: m.dashboard_period_3_months() },
			{ value: '6', label: m.dashboard_period_6_months() },
			{ value: '12', label: m.dashboard_period_12_months() }
		]}
		onValueChange={(value) => onMonthsChange(Number(value))}
	>
		<SelectTrigger class="h-7 w-28 gap-1.5 rounded-sm border-border/70 px-2">
			<ListFilter size={12} class="shrink-0 text-muted-foreground" />
			<SelectValue aria-label={m.dashboard_period()} />
		</SelectTrigger>
		<SelectContent>
			<SelectItem value="3" label={m.dashboard_period_3_months()}>{m.dashboard_period_3_months()}</SelectItem>
			<SelectItem value="6" label={m.dashboard_period_6_months()}>{m.dashboard_period_6_months()}</SelectItem>
			<SelectItem value="12" label={m.dashboard_period_12_months()}>{m.dashboard_period_12_months()}</SelectItem>
		</SelectContent>
	</Select>
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
