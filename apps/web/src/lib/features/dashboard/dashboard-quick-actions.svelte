<script lang="ts">
	import {
		Building2,
		FileText,
		Settings2,
		WalletCards,
		Boxes,
		ContactRound
	} from '@lucide/svelte';
	import { Button } from '@aqardesk/ui';
	import * as m from '#lib/paraglide/messages';
	import type { ViewKey } from '#lib/shell/navigation';

	let { onSelect }: { onSelect: (view: ViewKey) => void } = $props();

	const actions = $derived([
		{
			key: 'properties',
			label: m.nav_properties(),
			icon: Building2
		},
		{
			key: 'units',
			label: m.nav_units(),
			icon: Boxes
		},
		{
			key: 'tenants',
			label: m.nav_tenants(),
			icon: ContactRound
		},
		{
			key: 'contracts',
			label: m.nav_contracts(),
			icon: FileText
		},
		{
			key: 'payments',
			label: m.nav_payments(),
			icon: WalletCards
		},
		{
			key: 'settings',
			label: m.nav_settings(),
			icon: Settings2
		}
	] satisfies { key: ViewKey; label: string; icon: typeof Building2 }[]);
</script>

<section class="flex flex-wrap justify-center gap-2" aria-label={m.dashboard_quick_actions()}>
	{#each actions as action (action.key)}
		{@const Icon = action.icon}
		<Button
			variant="outline"
			class="h-8 gap-2 rounded-sm border-border/70 px-2.5 text-xs text-muted-foreground hover:text-foreground"
			aria-label={action.label}
			onclick={() => onSelect(action.key)}
		>
			<Icon size={13} />
			{action.label}
		</Button>
	{/each}
</section>
