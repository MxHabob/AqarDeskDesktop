<script lang="ts">
	import * as m from '#lib/paraglide/messages';
	import SettingsCapabilityNotice from '#lib/features/settings/settings-capability-notice.svelte';
</script>

<header class="grid gap-1">
	<h2 class="text-lg font-semibold">{m.settings_audit()}</h2>
	<p class="text-sm text-muted-foreground">{m.settings_audit_description()}</p>
</header>
<SettingsCapabilityNotice
	title={m.settings_audit()}
	description={m.settings_audit_unavailable()}
/>
