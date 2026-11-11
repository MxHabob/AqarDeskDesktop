<script lang="ts">
	import { tick } from 'svelte';
	import { Dialog, DialogContent, Input } from '@aqardesk/ui';
	import { Search, X } from '@lucide/svelte';
	import { Button } from '@aqardesk/ui';
	import * as m from '#lib/paraglide/messages';
	import type { User } from '#lib/features/auth/types';
	import type { ViewKey } from './navigation';
	import type { SearchItem } from './search-modal.types';
	import SearchModalResults from './search-modal-results.svelte';
	import SearchModalFooter from './search-modal-footer.svelte';

	let {
		open = $bindable(),
		currentUser,
		onSelect
	}: {
		open: boolean;
		currentUser: User;
		onSelect: (view: ViewKey) => void;
	} = $props();

	let query = $state('');
	let selectedIndex = $state(0);
	let inputElement = $state<HTMLInputElement | null>(null);

	const items = $derived<SearchItem[]>([
		{ key: 'overview', label: m.nav_overview(), keywords: ['dashboard', 'home', 'الرئيسية'] },
		{ key: 'properties', label: m.nav_properties(), keywords: ['building', 'عقار'] },
		{ key: 'units', label: m.nav_units(), keywords: ['apartment', 'وحدة'] },
		{ key: 'tenants', label: m.nav_tenants(), keywords: ['customer', 'مستأجر'] },
		{ key: 'contracts', label: m.nav_contracts(), keywords: ['lease', 'عقد'] },
		{ key: 'payments', label: m.nav_payments(), keywords: ['rent', 'invoice', 'دفعة'] },
		...(currentUser.role === 'admin'
			? [{ key: 'users' as const, label: m.nav_users(), keywords: ['team', 'users', 'مستخدم'] }]
			: []),
		{ key: 'settings', label: m.nav_settings(), keywords: ['preferences', 'إعدادات'] }
	]);

	const filteredItems = $derived.by(() => {
		const normalized = query.trim().toLocaleLowerCase();
		if (!normalized) return items;
		return items.filter((item) =>
			[item.label, ...item.keywords].some((value) => value.toLocaleLowerCase().includes(normalized))
		);
	});

	$effect(() => {
		if (!open) return;
		query = '';
		selectedIndex = 0;
		void tick().then(() => inputElement?.focus());
	});

	$effect(() => {
		if (selectedIndex >= filteredItems.length) selectedIndex = Math.max(0, filteredItems.length - 1);
	});

	function selectItem(item: SearchItem) {
		open = false;
		onSelect(item.key);
	}

	function handleKeydown(event: KeyboardEvent) {
		if (!open) return;
		if (filteredItems.length === 0) return;
		if (event.key === 'ArrowDown') {
			event.preventDefault();
			selectedIndex = Math.min(selectedIndex + 1, filteredItems.length - 1);
		} else if (event.key === 'ArrowUp') {
			event.preventDefault();
			selectedIndex = Math.max(selectedIndex - 1, 0);
		} else if (event.key === 'Enter' && filteredItems[selectedIndex]) {
			event.preventDefault();
			selectItem(filteredItems[selectedIndex]);
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<Dialog bind:open>
	<DialogContent class="max-w-xl gap-0 overflow-hidden p-0" showCloseButton={false}>
		<div class="flex h-14 items-center gap-3 border-b border-border/70 px-4">
			<Search size={17} class="shrink-0 text-muted-foreground" />
			<Input
				bind:ref={inputElement}
				bind:value={query}
				class="h-full border-0 bg-transparent px-0 shadow-none focus-visible:ring-0"
				placeholder={m.search_placeholder()}
				aria-label={m.search_placeholder()}
				autocomplete="off"
				oninput={() => (selectedIndex = 0)}
			/>
			<Button
				variant="ghost"
				size="icon"
				class="size-8 shrink-0 rounded-lg text-muted-foreground"
				aria-label={m.common_cancel()}
				onclick={() => (open = false)}
			>
				<X size={15} />
			</Button>
		</div>
		<SearchModalResults
			items={filteredItems}
			{selectedIndex}
			onSelect={selectItem}
			onHover={(index) => (selectedIndex = index)}
		/>
		<SearchModalFooter />
	</DialogContent>
</Dialog>
