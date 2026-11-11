<script lang="ts">
	import { Card } from '@aqardesk/ui';
	import { Volume2 } from '@lucide/svelte';
	import type { DashboardSummary } from '#lib/api/desktop';
	import { getLocale } from '#lib/paraglide/runtime';
	import * as m from '#lib/paraglide/messages';

	let { summary }: { summary: DashboardSummary } = $props();
	let dismissed = $state(false);

	const formatter = $derived(new Intl.NumberFormat(getLocale()));
	const summaryText = $derived(
		m.dashboard_summary_sentence({
			properties: formatter.format(summary.active_properties_count),
			units: formatter.format(summary.units_count),
			tenants: formatter.format(summary.tenants_count),
			leases: formatter.format(summary.active_leases_count)
		})
	);

	function listenToSummary() {
		if (!('speechSynthesis' in window)) return;
		window.speechSynthesis.cancel();
		const utterance = new SpeechSynthesisUtterance(summaryText);
		utterance.lang = getLocale() === 'ar' ? 'ar-SA' : 'en-US';
		window.speechSynthesis.speak(utterance);
	}
</script>

{#if !dismissed}
	<div class="relative isolate mx-auto mb-2 w-full max-w-[52rem] px-2 pb-2 pt-2">
		<div
			class="pointer-events-none absolute inset-x-3 bottom-3 top-1 border border-border bg-card"
			aria-hidden="true"
		></div>
		<div
			class="pointer-events-none absolute inset-x-1.5 bottom-1.5 top-2 border border-border bg-card"
			aria-hidden="true"
		></div>

		<Card
			class="relative aspect-[1.596] min-h-[32.5rem] w-full justify-between gap-6 p-6 sm:p-10"
		>
			<div class="grid gap-8 sm:gap-10">
				<div class="flex items-center justify-between gap-4">
					<h2 class="text-xl font-medium text-muted-foreground sm:text-2xl">
						{m.dashboard_statistics_title()}
					</h2>
					<span class="shrink-0 text-sm text-muted-foreground sm:text-base">
						{m.dashboard_summary_just_now()}
					</span>
				</div>

				<p class="text-[clamp(1.5rem,3vw,2.1rem)] leading-[1.4] tracking-tight text-foreground">
					{summaryText}
				</p>
			</div>

			<footer class="flex items-center justify-between gap-4 text-base text-muted-foreground sm:text-xl">
				<button
					type="button"
					class="inline-flex min-w-0 items-center gap-3 text-start transition-colors hover:text-foreground"
					onclick={listenToSummary}
				>
					<Volume2 size={20} class="shrink-0" />
					<span>{m.dashboard_summary_listen()}</span>
				</button>
				<button
					type="button"
					class="shrink-0 transition-colors hover:text-foreground"
					onclick={() => (dismissed = true)}
				>
					{m.dashboard_summary_dismiss()}
				</button>
			</footer>
		</Card>
	</div>
{/if}
