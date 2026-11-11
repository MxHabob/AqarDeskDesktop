<script lang="ts">
	import { createQuery } from '@tanstack/svelte-query';
	import { toast } from '@aqardesk/ui';
	import type { PageProps } from './$types';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import { confirmStore } from '#lib/core/confirm.svelte';
	import { desktopApi } from '#lib/api/desktop';
	import { queryClient } from '#lib/core/query-client';
	import PropertySheet from '#lib/features/properties/property-sheet.svelte';
	import PropertiesDirectory from '#lib/features/properties/properties-directory.svelte';
	import { propertyQueryKeys } from '#lib/features/properties/queries';
	import type { Property, PropertyInput } from '@aqardesk/sdk';
	import * as m from '#lib/paraglide/messages';

	let { data }: PageProps = $props();
	const app = getAppContext();
	const organizationId = $derived(data.orgId);
	const propertiesQuery = createQuery(() => ({
		queryKey: propertyQueryKeys.all(organizationId),
		queryFn: desktopApi.listProperties,
		enabled: !!app.currentUser
	}));
	let open = $state(false);
	let selectedProperty = $state<Property | undefined>();
	let saving = $state(false);
	const properties = $derived(propertiesQuery.data ?? []);
	$effect(() => {
		if (propertiesQuery.error) app.setError(propertiesQuery.error);
	});

	async function saveProperty(values: PropertyInput) {
		saving = true;
		try {
			if (selectedProperty) {
				await desktopApi.updateProperty(selectedProperty.id, values);
			} else {
				await desktopApi.createProperty(values);
			}
			await queryClient.invalidateQueries({ queryKey: propertyQueryKeys.all(organizationId) });
			return true;
		} catch (cause) {
			app.setError(cause);
			return false;
		} finally {
			saving = false;
		}
	}

	async function archiveProperty(property: Property) {
		const confirmed = await confirmStore.prompt({
			title: m.properties_archive_confirm_title(),
			description: m.properties_archive_confirm_description({ name: property.name }),
			confirmText: m.properties_archive(),
			cancelText: m.common_cancel(),
			variant: 'destructive'
		});
		if (!confirmed) return;
		try {
			await desktopApi.archiveProperty(property.id);
			await queryClient.invalidateQueries({ queryKey: propertyQueryKeys.all(organizationId) });
			toast.success(m.properties_archived());
		} catch (cause) {
			app.setError(cause);
		}
	}
</script>

{#if app.currentUser}
	<div class="flex min-h-0 flex-1 flex-col">
		<PropertiesDirectory
			properties={properties}
			loading={propertiesQuery.isLoading}
			readonly={app.currentUser.role === 'readonly'}
			onAdd={() => { selectedProperty = undefined; open = true; }}
			onEdit={(property) => { selectedProperty = property; open = true; }}
			onArchive={(property) => void archiveProperty(property)}
		/>
		<PropertySheet
			bind:open
			initial={selectedProperty}
			busy={saving}
			onsave={saveProperty}
			onclose={() => (open = false)}
		/>
	</div>
{/if}
