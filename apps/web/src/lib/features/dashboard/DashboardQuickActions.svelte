<script lang="ts">
	import { ArrowUpRight, Building2, FileText, UsersRound, WalletCards } from '@lucide/svelte';
	import { Button, Card } from '@aqardesk/ui';
	import * as m from '#lib/paraglide/messages';
	import type { ViewKey } from '#lib/shell/navigation';

	let { onSelect }: { onSelect: (view: ViewKey) => void } = $props();
</script>

<section class="grid gap-3">
	<div>
		<h2 class="text-base font-semibold text-foreground">{m.dashboard_quick_actions()}</h2>
		<p class="mt-1 text-sm text-muted-foreground">{m.app_tagline()}</p>
	</div>
	<div class="grid gap-3 sm:grid-cols-2">
		<Card class="rounded-lg bg-card p-4 shadow-card ring-border/50">
			<Button
				variant="ghost"
				class="h-auto w-full justify-start gap-4 whitespace-normal rounded-md p-0 text-start"
				onclick={() => onSelect('properties')}
			>
				<span class="grid size-11 shrink-0 place-items-center rounded-lg bg-accent text-accent-foreground">
					<Building2 size={21} />
				</span>
				<span class="grid min-w-0 flex-1 gap-1">
					<strong class="text-sm">{m.dashboard_properties_title()}</strong>
					<small class="text-xs font-normal leading-relaxed text-muted-foreground">
						{m.dashboard_properties_description()}
					</small>
				</span>
				<ArrowUpRight size={17} class="shrink-0 text-muted-foreground" />
			</Button>
		</Card>
		<Card class="rounded-lg bg-card p-4 shadow-card ring-border/50">
			<Button
				variant="ghost"
				class="h-auto w-full justify-start gap-4 whitespace-normal rounded-md p-0 text-start"
				onclick={() => onSelect('tenants')}
			>
				<span class="grid size-11 shrink-0 place-items-center rounded-lg bg-accent text-accent-foreground">
					<UsersRound size={21} />
				</span>
				<span class="grid min-w-0 flex-1 gap-1">
					<strong class="text-sm">{m.dashboard_tenants_title()}</strong>
					<small class="text-xs font-normal leading-relaxed text-muted-foreground">
						{m.dashboard_tenants_description()}
					</small>
				</span>
				<ArrowUpRight size={17} class="shrink-0 text-muted-foreground" />
			</Button>
		</Card>
		<Card class="rounded-lg bg-card p-4 shadow-card ring-border/50">
			<Button
				variant="ghost"
				class="h-auto w-full justify-start gap-4 whitespace-normal rounded-md p-0 text-start"
				onclick={() => onSelect('contracts')}
			>
				<span class="grid size-11 shrink-0 place-items-center rounded-lg bg-accent text-accent-foreground">
					<FileText size={21} />
				</span>
				<span class="grid min-w-0 flex-1 gap-1">
					<strong class="text-sm">{m.dashboard_contracts_title()}</strong>
					<small class="text-xs font-normal leading-relaxed text-muted-foreground">
						{m.dashboard_contracts_description()}
					</small>
				</span>
				<ArrowUpRight size={17} class="shrink-0 text-muted-foreground" />
			</Button>
		</Card>
		<Card class="rounded-lg bg-card p-4 shadow-card ring-border/50">
			<Button
				variant="ghost"
				class="h-auto w-full justify-start gap-4 whitespace-normal rounded-md p-0 text-start"
				onclick={() => onSelect('payments')}
			>
				<span class="grid size-11 shrink-0 place-items-center rounded-lg bg-accent text-accent-foreground">
					<WalletCards size={21} />
				</span>
				<span class="grid min-w-0 flex-1 gap-1">
					<strong class="text-sm">{m.dashboard_payments_title()}</strong>
					<small class="text-xs font-normal leading-relaxed text-muted-foreground">
						{m.dashboard_payments_description()}
					</small>
				</span>
				<ArrowUpRight size={17} class="shrink-0 text-muted-foreground" />
			</Button>
		</Card>
	</div>
</section>
