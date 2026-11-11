<script lang="ts">
	import { Check } from '@lucide/svelte';
	import { Button } from '@aqardesk/ui';
	import type { AppLocale } from '#lib/core/i18n';
	import SettingsFieldRow from '#lib/shared/components/settings-field-row.svelte';
	import * as m from '#lib/paraglide/messages';

	let { locale, onChange }: { locale: AppLocale; onChange: (locale: AppLocale) => void } = $props();
</script>

<SettingsFieldRow label={m.common_language()} description={m.settings_language_description()}>
	<div class="grid w-full gap-2 sm:grid-cols-2">
		<Button
			variant="outline"
			class={locale === 'ar'
				? 'h-10 justify-start rounded-md border-primary bg-accent text-accent-foreground'
				: 'h-10 justify-start rounded-md'}
			onclick={() => onChange('ar')}
		>
			<span class="grid size-7 place-items-center rounded-md bg-secondary text-sm">ع</span>
			<span class="flex-1 text-start">{m.language_arabic()}</span>
			{#if locale === 'ar'}<Check size={17} />{/if}
		</Button>
		<Button
			variant="outline"
			class={locale === 'en'
				? 'h-10 justify-start rounded-md border-primary bg-accent text-accent-foreground'
				: 'h-10 justify-start rounded-md'}
			onclick={() => onChange('en')}
		>
			<span class="grid size-7 place-items-center rounded-md bg-secondary text-xs">EN</span>
			<span class="flex-1 text-start">{m.language_english()}</span>
			{#if locale === 'en'}<Check size={17} />{/if}
		</Button>
	</div>
</SettingsFieldRow>
