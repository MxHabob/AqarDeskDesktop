<script lang="ts">
	import { Button } from '@aqardesk/ui/components/ui/button';
	import { Plus, Trash } from '../icons';
	import * as m from '#lib/paraglide/messages';
	import type { EditorStore } from '../stores/editor-store.svelte';

	let { store }: { store: EditorStore } = $props();
</script>

<section class="flex h-full min-h-0 flex-col">
	<div class="flex items-center justify-between border-b border-border px-3 py-2.5">
		<h3 class="text-sm font-semibold">{m.common_td_panel_pages()}</h3>
		<Button
			variant="ghost"
			size="icon-sm"
			aria-label={m.template_page_add()}
			title={m.template_page_add()}
			onclick={() => store.addPage()}
		>
			<Plus size={16} />
		</Button>
	</div>

	<div class="min-h-0 flex-1 space-y-1 overflow-y-auto p-2">
		{#each store.document.pages as page, index (page.id)}
			<div class="flex items-center gap-1">
				<button
					type="button"
					class="min-w-0 flex-1 rounded-md px-3 py-2 text-start text-sm hover:bg-muted"
					class:bg-muted={store.activePageIndex === index}
					aria-current={store.activePageIndex === index ? 'page' : undefined}
					onclick={() => store.setActivePage(index)}
				>
					{m.template_page_label()} {index + 1}
				</button>
				<Button
					variant="ghost"
					size="icon-xs"
					aria-label={m.template_page_remove()}
					title={m.template_page_remove()}
					disabled={store.document.pages.length <= 1}
					onclick={() => store.removePage(index)}
				>
					<Trash size={14} />
				</Button>
			</div>
		{/each}
	</div>
</section>
