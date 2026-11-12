<script lang="ts">
	import { tick } from 'svelte';
	import { Dialog, DialogContent, DialogTitle, Input } from '@aqardesk/ui';
	import { Search } from '@lucide/svelte';
	import * as m from '#lib/paraglide/messages';
	import { canViewBusinessView } from '#lib/features/auth/permissions';
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

	const items = $derived.by(() => {
		const allItems: SearchItem[] = [
			{ key: 'overview', label: m.nav_overview(), keywords: ['dashboard', 'home', 'الرئيسية'] },
			{ key: 'properties', label: m.nav_properties(), keywords: ['building', 'عقار'] },
			{ key: 'units', label: m.nav_units(), keywords: ['apartment', 'وحدة'] },
			{ key: 'tenants', label: m.nav_tenants(), keywords: ['customer', 'مستأجر'] },
			{ key: 'owners', label: m.nav_owners(), keywords: ['landlord', 'property owner', 'مالك'] },
			{ key: 'contracts', label: m.nav_contracts(), keywords: ['lease', 'عقد'] },
			{ key: 'payments', label: m.nav_payments(), keywords: ['rent', 'invoice', 'دفعة'] },
			...(currentUser.role === 'admin'
				? [{ key: 'users' as const, label: m.nav_users(), keywords: ['team', 'users', 'مستخدم'] }]
				: []),
			{ key: 'settings', label: m.nav_settings(), keywords: ['preferences', 'إعدادات'] }
		];
		return allItems.filter((item) => {
			switch (item.key) {
				case 'properties':
					return (['properties', 'units', 'tenants', 'owners'] as const).some((view) =>
						canViewBusinessView(currentUser.role, view, currentUser.permissions)
					);
				case 'units':
				case 'tenants':
				case 'owners':
				case 'payments':
					return canViewBusinessView(currentUser.role, item.key, currentUser.permissions);
				default:
					return true;
			}
		});
	});

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
		if (
			!event.isComposing &&
			!event.altKey &&
			!event.shiftKey &&
			(event.ctrlKey || event.metaKey) &&
			event.key.toLowerCase() === 'k'
		) {
			event.preventDefault();
			open = !open;
			return;
		}

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
	<DialogContent
		class="m-0 flex h-[min(535px,calc(100dvh-2rem))] w-[calc(100%-2rem)] max-w-[740px] flex-col gap-0 overflow-hidden border-none bg-transparent p-0 shadow-none ring-0 select-text"
		showCloseButton={false}
	>
		<DialogTitle class="sr-only">{m.common_search()}</DialogTitle>
		<div class="flex h-full min-h-0 flex-col overflow-hidden rounded-xl border border-border bg-background">
			<div class="flex h-[55px] shrink-0 items-center gap-3 border-b border-border px-4">
				<Search size={17} class="shrink-0 text-muted-foreground" />
				<Input
					bind:ref={inputElement}
					bind:value={query}
					class="h-full border-0 bg-transparent px-0 shadow-none focus-visible:ring-0"
					placeholder={m.search_placeholder()}
					aria-label={m.search_placeholder()}
					role="combobox"
					aria-controls="global-search-results"
					aria-expanded={open}
					aria-autocomplete="list"
					aria-activedescendant={filteredItems[selectedIndex]
						? `search-result-${filteredItems[selectedIndex].key}`
						: undefined}
					autocomplete="off"
					oninput={() => (selectedIndex = 0)}
				/>
			</div>
			<SearchModalResults
				items={filteredItems}
				{selectedIndex}
				onSelect={selectItem}
				onHover={(index) => (selectedIndex = index)}
			/>
			<SearchModalFooter />
		</div>
	</DialogContent>
</Dialog>
