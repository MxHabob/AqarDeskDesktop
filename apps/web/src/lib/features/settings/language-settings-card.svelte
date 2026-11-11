<script lang="ts">
	import { Check } from '@lucide/svelte';
	import { Button, Card, CardDescription, CardHeader, CardTitle } from '@aqardesk/ui';
	import type { AppLocale } from '#lib/core/i18n';
	import * as m from '#lib/paraglide/messages';

	let { locale, onChange }: { locale: AppLocale; onChange: (locale: AppLocale) => void } = $props();
</script>

<Card class="rounded-lg bg-card p-5 shadow-card ring-border/50">
	<CardHeader class="mb-4 p-0">
		<CardTitle class="text-base">{m.common_language()}</CardTitle>
		<CardDescription>{m.common_language()}</CardDescription>
	</CardHeader>
	<div class="grid gap-2">
		<Button
			variant="outline"
			class={locale === 'ar'
				? 'h-12 justify-start rounded-md border-primary bg-accent text-accent-foreground'
				: 'h-12 justify-start rounded-md'}
			onclick={() => onChange('ar')}
		>
			<span class="grid size-8 place-items-center rounded-md bg-secondary text-sm">ع</span>
			<span class="flex-1 text-start">{m.language_arabic()}</span>
			{#if locale === 'ar'}<Check size={17} />{/if}
		</Button>
		<Button
			variant="outline"
			class={locale === 'en'
				? 'h-12 justify-start rounded-md border-primary bg-accent text-accent-foreground'
				: 'h-12 justify-start rounded-md'}
			onclick={() => onChange('en')}
		>
			<span class="grid size-8 place-items-center rounded-md bg-secondary text-xs">EN</span>
			<span class="flex-1 text-start">{m.language_english()}</span>
			{#if locale === 'en'}<Check size={17} />{/if}
		</Button>
	</div>
</Card>
