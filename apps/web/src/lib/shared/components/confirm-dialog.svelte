<script lang="ts">
	import { TriangleAlert } from '@lucide/svelte';
	import {
		Button,
		Card,
		Dialog,
		DialogContent,
		DialogDescription,
		DialogFooter,
		DialogHeader,
		DialogTitle
	} from '@aqardesk/ui';
	import { confirmStore } from '#lib/core/confirm.svelte';

	$effect(() => {
		confirmStore.isOpen;
		confirmStore.cancelIfClosed();
	});
</script>

<Dialog bind:open={confirmStore.isOpen}>
	<DialogContent showCloseButton={false}>
		<DialogHeader>
			<DialogTitle>{confirmStore.options?.title}</DialogTitle>
			<DialogDescription class="sr-only">
				{confirmStore.options?.description}
			</DialogDescription>
		</DialogHeader>

		<Card
			class="flex items-start gap-3 rounded-lg border-destructive/20 bg-destructive/5 p-4 shadow-none"
		>
			<span class="grid size-9 shrink-0 place-items-center rounded-lg bg-destructive/10 text-destructive">
				<TriangleAlert size={18} />
			</span>
			<p class="text-sm leading-relaxed text-foreground">
				{confirmStore.options?.description}
			</p>
		</Card>

		<DialogFooter>
			<Button variant="outline" class="rounded-md" onclick={() => confirmStore.respond(false)}>
				{confirmStore.options?.cancelText}
			</Button>
			<Button
				variant={confirmStore.options?.variant === 'destructive' ? 'destructive' : 'default'}
				class="rounded-md"
				onclick={() => confirmStore.respond(true)}
			>
				{confirmStore.options?.confirmText}
			</Button>
		</DialogFooter>
	</DialogContent>
</Dialog>
