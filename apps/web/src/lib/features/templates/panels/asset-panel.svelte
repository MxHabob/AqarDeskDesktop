<script lang="ts">
	import { toast } from '@aqardesk/ui';
	import { Button } from '@aqardesk/ui/components/ui/button';
	import { ArrowUpTray, Photo } from '../icons';
	import * as m from '#lib/paraglide/messages';
	import type { EditorStore } from '../stores/editor-store.svelte';

	let { store }: { store: EditorStore } = $props();
	let fileInput: HTMLInputElement | undefined = $state();

	function addImage(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const file = input.files?.[0];
		if (!file) return;

		if (!/^image\/(?:png|jpeg|webp)$/.test(file.type) || file.size > 1_500_000) {
			toast.error(m.template_asset_invalid_file());
			input.value = '';
			return;
		}

		const reader = new FileReader();
		reader.onload = () => {
			if (typeof reader.result !== 'string') {
				toast.error(m.template_asset_upload_failed());
				input.value = '';
				return;
			}
			store.addElement('Image', {
				src: reader.result,
				width: 50,
				height: 35,
				fit: 'contain'
			});
			toast.success(m.template_asset_upload_success());
			input.value = '';
		};
		reader.onerror = () => {
			toast.error(m.template_asset_upload_failed());
			input.value = '';
		};
		reader.readAsDataURL(file);
	}
</script>

<section class="flex h-full min-h-0 flex-col">
	<div class="flex items-center justify-between border-b border-border px-3 py-2.5">
		<div class="flex items-center gap-2">
			<Photo size={16} class="text-muted-foreground" />
			<h3 class="text-sm font-semibold">{m.template_title_assets()}</h3>
		</div>
		<Button
			variant="ghost"
			size="icon-sm"
			aria-label={m.template_btn_add_asset()}
			title={m.template_btn_add_asset()}
			onclick={() => fileInput?.click()}
		>
			<ArrowUpTray size={15} />
		</Button>
		<input
			bind:this={fileInput}
			class="hidden"
			type="file"
			accept="image/png,image/jpeg,image/webp"
			onchange={addImage}
		/>
	</div>

	<div class="flex flex-1 flex-col items-center justify-center gap-3 p-5 text-center">
		<Photo size={28} class="text-muted-foreground/50" />
		<p class="text-sm text-muted-foreground">{m.template_asset_upload_help()}</p>
		<Button variant="outline" onclick={() => fileInput?.click()}>
			<ArrowUpTray size={15} />
			{m.template_btn_add_asset()}
		</Button>
		<p class="text-xs text-muted-foreground">{m.template_asset_file_limit()}</p>
	</div>
</section>
