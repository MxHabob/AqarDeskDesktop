<script lang="ts">
	import { goto } from '$app/navigation';
	import { Building2, Database, Globe2 } from '@lucide/svelte';
	import { Button } from '@aqardesk/ui';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import LoginForm from '#lib/features/auth/LoginForm.svelte';
	import LoginVideoBackground from '#lib/features/auth/LoginVideoBackground.svelte';
	import * as m from '#lib/paraglide/messages';
	import MessageBanner from '#lib/shared/components/MessageBanner.svelte';

	const app = getAppContext();

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

<main class="relative flex min-h-dvh bg-background font-sans text-foreground scrollbar-none">
	<header class="fixed inset-x-0 top-0 z-50 flex items-center justify-between px-4 py-3 sm:px-6 xl:px-8">
		<a href="/" class="flex items-center gap-2 text-sm font-semibold text-foreground no-underline">
			<Building2 size={20} />{m.app_name()}
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

	<LoginVideoBackground />

	<section class="flex min-h-dvh w-full flex-col items-center justify-center p-8 pb-2 lg:w-1/2 lg:p-12 lg:pb-2">
		<div class="flex min-h-[calc(100svh-4rem)] w-full max-w-md flex-col lg:min-h-[calc(100svh-6rem)]">
			<div class="flex flex-1 flex-col justify-center space-y-8">
				<div class="space-y-2 text-center">
					<h1 class="mb-4 font-serif text-lg lg:text-xl">
						{app.needsAdmin ? m.auth_admin_title() : m.auth_signin_title()}
					</h1>
					<p class="font-sans text-sm text-muted-foreground">
						{app.needsAdmin ? m.auth_admin_description() : m.auth_signin_description()}
					</p>
				</div>

				{#if app.error}<MessageBanner message={app.error} />{/if}
				{#if app.notice}<MessageBanner message={app.notice} variant="success" />{/if}

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
