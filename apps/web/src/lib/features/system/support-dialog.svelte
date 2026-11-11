<script lang="ts">
	import { ExternalLink, Globe2 } from '@lucide/svelte';
	import { Button, Dialog, DialogContent, DialogDescription, DialogTitle, toast } from '@aqardesk/ui';
	import { openExternalUrl } from '#lib/core/external-url';
	import * as m from '#lib/paraglide/messages';

	let { open = $bindable(false) }: { open?: boolean } = $props();

	async function visitSupport() {
		try {
			await openExternalUrl('https://www.mulverse.com/');
		} catch (cause) {
			toast.error(cause instanceof Error ? cause.message : String(cause));
		}
	}
</script>

<Dialog bind:open>
	<DialogContent class="max-w-md">
		<div class="grid gap-2">
			<DialogTitle>{m.system_support_title()}</DialogTitle>
			<DialogDescription>{m.system_support_description()}</DialogDescription>
		</div>
		<Button class="w-full" onclick={() => void visitSupport()}>
			<Globe2 size={15} />
			{m.system_about_website()}
			<ExternalLink size={13} />
		</Button>
	</DialogContent>
</Dialog>
