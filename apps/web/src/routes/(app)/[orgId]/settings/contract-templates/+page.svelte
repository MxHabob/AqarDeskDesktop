<script lang="ts">
	import * as m from '#lib/paraglide/messages';
	import SettingsCapabilityNotice from '#lib/features/settings/settings-capability-notice.svelte';
</script>

<header class="grid gap-1">
	<h2 class="text-lg font-semibold">{m.settings_contract_templates()}</h2>
	<p class="text-sm text-muted-foreground">{m.settings_contract_templates_description()}</p>
</header>
<SettingsCapabilityNotice
	title={m.settings_contract_templates()}
	description={m.settings_contract_templates_unavailable()}
/>
