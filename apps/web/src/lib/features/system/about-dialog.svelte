<script lang="ts">
	import { Globe2, Sparkles } from '@lucide/svelte';
	import { Button, Dialog, DialogContent, DialogDescription, DialogTitle } from '@aqardesk/ui';
	import AqarDeskLogo from '#lib/shared/components/aqardesk-logo.svelte';
	import { openExternalUrl } from '#lib/core/external-url';
	import * as m from '#lib/paraglide/messages';
	import { toast } from '@aqardesk/ui';

	let { open = $bindable(false) }: { open?: boolean } = $props();

	async function visit(url: string) {
		try {
			await openExternalUrl(url);
		} catch (cause) {
			toast.error(cause instanceof Error ? cause.message : String(cause));
		}
	}
</script>

<Dialog bind:open>
	<DialogContent class="max-w-lg gap-5">
		<div class="grid justify-items-center gap-3 py-3 text-center">
			<AqarDeskLogo size={44} class="text-primary" />
			<div class="grid gap-1">
				<DialogTitle class="text-xl font-bold">{m.app_name()}</DialogTitle>
				<DialogDescription>{m.app_tagline()}</DialogDescription>
			</div>
		</div>
		<p class="text-sm text-muted-foreground">{m.system_about_description()}</p>
		<p class="text-xs font-medium text-muted-foreground">{m.system_about_company()}</p>
		<div class="flex flex-wrap justify-between gap-2">
			<Button variant="ghost" size="sm" onclick={() => void visit('https://www.mulverse.com/')}>
				<Globe2 size={14} />
				{m.system_about_website()}
			</Button>
			<div class="flex gap-2">
				<Button
					variant="outline"
					size="sm"
					onclick={() => void visit('https://github.com/MxHabob/AqarDeskDesktop/releases')}
				>
					<Sparkles size={14} />
					{m.system_about_releases()}
				</Button>
				<Button size="sm" onclick={() => (open = false)}>{m.system_about_close()}</Button>
			</div>
		</div>
	</DialogContent>
</Dialog>
