<script lang="ts">
	import { Button } from '@aqardesk/ui';
	import * as m from '#lib/paraglide/messages';

	type Props = {
		visible: boolean;
		isSaving: boolean;
		onsave: () => void | Promise<void>;
		onreset: () => void;
	};

	let { visible, isSaving, onsave, onreset }: Props = $props();
</script>

{#if visible}
	<div
		class="fixed bottom-4 left-1/2 z-50 flex w-[calc(100%-2rem)] max-w-lg -translate-x-1/2 items-center justify-between gap-3 rounded-xl border border-border/80 bg-card/95 px-4 py-3 shadow-xl backdrop-blur-md"
		role="region"
		aria-label={m.settings_unsaved_changes()}
	>
		<p class="min-w-0 truncate text-sm font-medium text-muted-foreground">
			{m.settings_unsaved_changes()}
		</p>
		<div class="flex shrink-0 items-center gap-2">
			<Button type="button" variant="ghost" size="sm" disabled={isSaving} onclick={onreset}>
				{m.settings_discard_changes()}
			</Button>
			<Button type="button" size="sm" disabled={isSaving} onclick={onsave}>
				{isSaving ? m.common_processing() : m.settings_save_changes()}
			</Button>
		</div>
	</div>
{/if}
