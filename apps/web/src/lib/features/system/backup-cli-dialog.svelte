<script lang="ts">
	import { goto } from '$app/navigation';
	import { Database } from '@lucide/svelte';
	import { Button, Dialog, DialogContent, DialogDescription, DialogTitle } from '@aqardesk/ui';
	import * as m from '#lib/paraglide/messages';

	let {
		open = $bindable(false),
		organizationId
	}: {
		open?: boolean;
		organizationId: string;
	} = $props();

	function openDatabaseSettings() {
		open = false;
		void goto(`/${organizationId}/settings/databases`);
	}
</script>

<Dialog bind:open>
	<DialogContent class="max-w-md">
		<div class="grid gap-2">
			<DialogTitle>{m.system_cli_title()}</DialogTitle>
			<DialogDescription>{m.system_cli_description()}</DialogDescription>
		</div>
		<Button class="w-full" onclick={openDatabaseSettings}>
			<Database size={15} />
			{m.system_cli_open()}
		</Button>
	</DialogContent>
</Dialog>
