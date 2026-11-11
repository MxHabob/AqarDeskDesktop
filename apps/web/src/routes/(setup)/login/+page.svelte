<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { Database, Globe2 } from '@lucide/svelte';
	import { Button } from '@aqardesk/ui';
	import AqarDeskLogo from '#lib/shared/components/aqardesk-logo.svelte';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import LoginForm from '#lib/features/auth/login-form.svelte';
	import LoginVideoBackground from '#lib/features/auth/login-video-background.svelte';
	import * as m from '#lib/paraglide/messages';

	const app = getAppContext();
	let desktopShell = $state(false);

	onMount(() => {
		desktopShell = '__TAURI_INTERNALS__' in window;
	});

	async function authenticate(event: SubmitEvent) {
		if (app.needsAdmin) await app.createInitialAdmin(event);
		else await app.signIn(event);
		if (app.currentUser) await goto(`/${app.currentUser.organization_id}`);
	}

	async function openConnectionSettings() {
		app.clearMessages();
		await goto('/connection');
	}
</script>

<main class="relative flex h-full min-h-0 overflow-y-auto bg-background font-sans text-foreground scrollbar-none">
	{#if !desktopShell}
	<header class="absolute inset-x-0 top-0 z-50 flex items-center justify-between px-4 py-3 sm:px-6 xl:px-8">
		<a href="/" class="flex items-center gap-2 text-sm font-semibold text-foreground no-underline">
			<AqarDeskLogo size={20} class="text-foreground" />{m.app_name()}
		</a>
		<Button
			variant="ghost"
			size="sm"
			class="rounded-md"
			onclick={() => void app.changeLanguage(app.locale === 'ar' ? 'en' : 'ar')}
		>
			<Globe2 size={15} />{app.locale === 'ar' ? m.language_english() : m.language_arabic()}
		</Button>
	</header>
	{/if}

	<LoginVideoBackground />

	<section class="flex min-h-0 w-full flex-col items-center justify-center p-8 pb-2 lg:w-1/2 lg:p-12 lg:pb-2">
		<div class="flex min-h-0 w-full max-w-md flex-1 flex-col">
			<div class="flex flex-1 flex-col justify-center space-y-8">
				<div class="space-y-2 text-center">
					<h1 class="mb-4 font-serif text-lg lg:text-xl">
						{app.needsAdmin ? m.auth_admin_title() : m.auth_signin_title()}
					</h1>
					<p class="font-sans text-sm text-muted-foreground">
						{app.needsAdmin ? m.auth_admin_description() : m.auth_signin_description()}
					</p>
				</div>

				<LoginForm
					needsAdmin={app.needsAdmin}
					busy={app.busy}
					organizationName={app.organizationName}
					name={app.fullName}
					email={app.email}
					password={app.accountPassword}
					onSubmit={authenticate}
					onOrganizationNameChange={(value) => (app.organizationName = value)}
					onNameChange={(value) => (app.fullName = value)}
					onEmailChange={(value) => (app.email = value)}
					onPasswordChange={(value) => (app.accountPassword = value)}
				/>

				<Button
					variant="ghost"
					class="mx-auto h-9 rounded-md text-muted-foreground"
					onclick={openConnectionSettings}
				>
					<Database size={15} />{m.connection_edit()}
				</Button>
			</div>

			<footer class="mt-auto flex justify-center pt-4 text-xs text-muted-foreground">
				<span class="flex items-center gap-2 text-foreground">
					<span class="size-2 rounded-full bg-primary"></span>{m.status_connected()}
				</span>
			</footer>
		</div>
	</section>
</main>
