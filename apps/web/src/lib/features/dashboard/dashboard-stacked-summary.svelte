<script lang="ts">
	import { Card } from '@aqardesk/ui';
	import { Volume2 } from '@lucide/svelte';
	import type { DashboardSummary } from '#lib/api/desktop';
	import { formatNumber } from '#lib/core/i18n';
	import { getLocale } from '#lib/paraglide/runtime';
	import * as m from '#lib/paraglide/messages';

	let { summary }: { summary: DashboardSummary } = $props();
	let dismissed = $state(false);
	let speechRequestId = 0;

	const summaryText = $derived(
		m.dashboard_summary_sentence({
			properties: formatNumber(summary.active_properties_count, { maximumFractionDigits: 0 }),
			units: formatNumber(summary.units_count, { maximumFractionDigits: 0 }),
			tenants: formatNumber(summary.tenants_count, { maximumFractionDigits: 0 })
		})
	);

	function loadVoices(synthesis: SpeechSynthesis): Promise<SpeechSynthesisVoice[]> {
		const voices = synthesis.getVoices();
		if (voices.length > 0) return Promise.resolve(voices);

		return new Promise((resolve) => {
			const finish = () => {
				window.clearTimeout(timeoutId);
				synthesis.removeEventListener('voiceschanged', onVoicesChanged);
				resolve(synthesis.getVoices());
			};
			const onVoicesChanged = () => finish();
			const timeoutId = window.setTimeout(finish, 800);
			synthesis.addEventListener('voiceschanged', onVoicesChanged, { once: true });
		});
	}

	async function listenToSummary() {
		if (!('speechSynthesis' in window)) return;
		const requestId = ++speechRequestId;
		const synthesis = window.speechSynthesis;
		synthesis.cancel();

		const locale = getLocale() === 'ar' ? 'ar-SA' : 'en-US';
		const language = locale.split('-')[0];
		const voices = await loadVoices(synthesis);
		if (requestId !== speechRequestId || dismissed) return;

		const utterance = new SpeechSynthesisUtterance(summaryText);
		const matchingVoices = voices.filter(
			(voice) =>
				voice.lang.toLowerCase().startsWith(`${language.toLowerCase()}-`) ||
				voice.lang.toLowerCase() === language.toLowerCase()
		);
		utterance.voice =
			matchingVoices.find((voice) => voice.lang.toLowerCase() === locale.toLowerCase()) ??
			matchingVoices.find((voice) => voice.default) ??
			matchingVoices[0] ??
			null;
		utterance.lang = utterance.voice?.lang ?? locale;
		utterance.rate = 0.9;
		utterance.pitch = 1;
		utterance.volume = 1;
		synthesis.speak(utterance);
	}
</script>

{#if !dismissed}
	<div class="relative isolate h-[182px] min-w-0">
		<div
			class="pointer-events-none absolute -inset-x-2 top-[-8px] bottom-2 -rotate-1 border border-border bg-card"
			aria-hidden="true"
		></div>
		<div
			class="pointer-events-none absolute -inset-x-1.5 top-[-4px] bottom-1 rotate-[0.4deg] border border-border bg-card"
			aria-hidden="true"
		></div>

		<Card class="relative z-10 h-full min-w-0 justify-between gap-2 bg-card p-3">
			<div class="grid gap-2">
				<div class="flex items-center justify-between gap-2">
					<h2 class="truncate text-xs font-medium text-muted-foreground">
						{m.dashboard_statistics_title()}
					</h2>
					<span class="shrink-0 text-[10px] text-muted-foreground">
						{m.dashboard_summary_just_now()}
					</span>
				</div>

				<p class="line-clamp-3 text-sm leading-5 tracking-tight text-foreground">
					{summaryText}
				</p>
			</div>

			<footer class="flex items-center justify-between gap-2 text-[11px] text-muted-foreground">
				<button
					type="button"
					class="inline-flex min-w-0 items-center gap-1.5 text-start transition-colors hover:text-foreground"
					onclick={listenToSummary}
				>
					<Volume2 size={14} class="shrink-0" />
					<span class="truncate">{m.dashboard_summary_listen()}</span>
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
