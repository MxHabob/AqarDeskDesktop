<script lang="ts">
	import { goto } from '$app/navigation';
	import { ArrowRight, Building2, ClipboardList, FileText, Settings2, UserRound, UsersRound } from '@lucide/svelte';
	import { Button, Card, CardDescription, CardHeader, CardTitle } from '@aqardesk/ui';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import * as m from '#lib/paraglide/messages';

	const app = getAppContext();
	const basePath = `/${app.currentUser?.organization_id ?? ''}/settings`;
	const sections = $derived([
		{
			path: 'general',
			title: m.settings_general(),
			description: m.settings_general_description(),
			icon: Settings2
		},
		{
			path: 'organization',
			title: m.settings_organization(),
			description: m.settings_organization_description(),
			icon: Building2
		},
		{
			path: 'profile',
			title: m.settings_my_profile(),
			description: m.settings_my_profile_description(),
			icon: UserRound
		},
		...(app.currentUser?.role === 'admin'
			? [
					{
						path: 'users',
						title: m.settings_user_management(),
						description: m.settings_user_management_description(),
						icon: UsersRound
					},
					{
						path: 'audit',
						title: m.settings_audit(),
						description: m.settings_audit_description(),
						icon: ClipboardList
					},
					{
						path: 'contract-templates',
						title: m.settings_contract_templates(),
						description: m.settings_contract_templates_description(),
						icon: FileText
					}
				]
			: [])
	]);
</script>

<header class="grid gap-1">
	<h2 class="text-lg font-semibold">{m.settings_title()}</h2>
	<p class="text-sm text-muted-foreground">{m.settings_description()}</p>
</header>
<div class="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
	{#each sections as section (section.path)}
		{@const Icon = section.icon}
		<Card class="flex min-w-0 flex-col rounded-lg bg-card p-4 shadow-card ring-border/50">
			<CardHeader class="flex-1 p-0">
				<span class="mb-3 grid size-9 place-items-center rounded-lg bg-accent text-accent-foreground">
					<Icon size={18} />
				</span>
				<CardTitle class="text-sm">{section.title}</CardTitle>
				<CardDescription class="text-xs leading-relaxed">{section.description}</CardDescription>
			</CardHeader>
			<Button
				variant="ghost"
				class="mt-4 justify-between px-0 text-start"
				onclick={() => void goto(`${basePath}/${section.path}`)}
			>
				{m.common_open()}
				<ArrowRight size={15} class="rtl:rotate-180" />
			</Button>
		</Card>
	{/each}
</div>
