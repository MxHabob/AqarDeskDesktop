<script lang="ts">
	import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@aqardesk/ui';

	type DetailField = {
		label: string;
		value: string;
		direction?: 'auto' | 'ltr';
		preformatted?: boolean;
	};

	let {
		open = $bindable(false),
		title,
		fields
	}: {
		open: boolean;
		title: string;
		fields: DetailField[];
	} = $props();
</script>

<Dialog bind:open>
	<DialogContent class="max-h-[85dvh] overflow-y-auto">
		<DialogHeader>
			<DialogTitle>{title}</DialogTitle>
			<DialogDescription class="sr-only">{title}</DialogDescription>
		</DialogHeader>
		<dl class="grid gap-3">
			{#each fields as field (field.label)}
				<div class="grid min-w-0 gap-1">
					<dt class="text-xs font-medium text-muted-foreground">{field.label}</dt>
					<dd class="min-w-0 text-sm" dir={field.direction ?? 'auto'}>
						{#if field.preformatted}
							<pre class="max-h-64 overflow-auto whitespace-pre-wrap break-words rounded-md bg-muted/50 p-3 text-xs">{field.value}</pre>
						{:else}
							<p class="break-words">{field.value}</p>
						{/if}
					</dd>
				</div>
			{/each}
		</dl>
	</DialogContent>
</Dialog>
