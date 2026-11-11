<script lang="ts">
	import { onMount } from 'svelte';
	import { z } from 'zod';
	import { Input, Label, toast } from '@aqardesk/ui';
	import type { ProfileSettings, ProfileSettingsInput } from '@aqardesk/sdk';
	import { desktopApi } from '#lib/api/desktop';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import SettingsBottomBar from '#lib/shared/components/settings-bottom-bar.svelte';
	import * as m from '#lib/paraglide/messages';

	const app = getAppContext();
	let saved = $state<ProfileSettings | null>(null);
	let draft = $state<ProfileSettingsInput>({
		name: '',
		email: '',
		current_password: '',
		new_password: ''
	});
	let loading = $state(true);
	let saving = $state(false);
	const dirty = $derived(
		saved !== null &&
			(draft.name !== saved.name ||
				draft.email !== saved.email ||
				Boolean(draft.current_password) ||
				Boolean(draft.new_password))
	);
	const schema = z
		.object({
			name: z.string().trim().min(1, m.users_name()),
			email: z.email(m.auth_email()),
			current_password: z.string(),
			new_password: z.string()
		})
		.superRefine((value, context) => {
			if (value.current_password || value.new_password) {
				if (!value.current_password) {
					context.addIssue({
						code: 'custom',
						message: m.settings_current_password(),
						path: ['current_password']
					});
				}
				if (value.new_password.length < 12) {
					context.addIssue({
						code: 'custom',
						message: m.auth_password_hint(),
						path: ['new_password']
					});
				}
			}
		});

	onMount(() => {
		void load();
	});

	async function load() {
		loading = true;
		try {
			const profile = await desktopApi.getProfileSettings();
			saved = profile;
			draft = {
				name: profile.name,
				email: profile.email,
				current_password: '',
				new_password: ''
			};
		} catch (cause) {
			toast.error(cause instanceof Error ? cause.message : String(cause));
		} finally {
			loading = false;
		}
	}

	async function save() {
		const parsed = schema.safeParse({
			...draft,
			name: draft.name.trim(),
			email: draft.email.trim().toLowerCase()
		});
		if (!parsed.success) {
			toast.error(parsed.error.issues[0]?.message ?? m.error_unexpected_description());
			return;
		}
		saving = true;
		try {
			const profile = await desktopApi.updateProfileSettings({
				...parsed.data,
				current_password: parsed.data.current_password || null,
				new_password: parsed.data.new_password || null
			});
			saved = profile;
			draft = {
				name: profile.name,
				email: profile.email,
				current_password: '',
				new_password: ''
			};
			app.updateCurrentUserProfile(profile.name, profile.email);
			toast.success(m.common_save());
		} catch (cause) {
			toast.error(cause instanceof Error ? cause.message : String(cause));
		} finally {
			saving = false;
		}
	}

	function reset() {
		if (!saved) return;
		draft = {
			name: saved.name,
			email: saved.email,
			current_password: '',
			new_password: ''
		};
	}

	const roleLabel = $derived(
		saved?.role === 'admin'
			? m.users_role_admin()
			: saved?.role === 'staff'
				? m.users_role_staff()
				: m.users_role_readonly()
	);
</script>

<header class="grid gap-1">
	<h2 class="text-lg font-semibold">{m.nav_account()}</h2>
	<p class="text-sm text-muted-foreground">{m.settings_my_profile_description()}</p>
</header>

{#if loading}
	<div class="py-8 text-sm text-muted-foreground" role="status">{m.common_processing()}</div>
{:else if saved}
	<form
		class="grid min-w-0 divide-y divide-border/70 pb-20"
		onsubmit={(event) => {
			event.preventDefault();
			void save();
		}}
	>
		<div class="grid gap-2 py-4 sm:grid-cols-[12rem_minmax(0,1fr)] sm:items-center sm:gap-6">
			<Label for="profile-name">{m.users_name()}</Label>
			<Input id="profile-name" bind:value={draft.name} autocomplete="name" />
		</div>
		<div class="grid gap-2 py-4 sm:grid-cols-[12rem_minmax(0,1fr)] sm:items-center sm:gap-6">
			<Label for="profile-email">{m.auth_email()}</Label>
			<Input id="profile-email" bind:value={draft.email} type="email" dir="ltr" autocomplete="email" />
		</div>
		<div class="grid gap-2 py-4 sm:grid-cols-[12rem_minmax(0,1fr)] sm:items-center sm:gap-6">
			<p class="text-sm font-medium">{m.users_role()}</p>
			<p class="text-sm">{roleLabel}</p>
		</div>
		<div class="grid gap-2 py-4 sm:grid-cols-[12rem_minmax(0,1fr)] sm:items-start sm:gap-6">
			<Label for="profile-current-password">{m.settings_current_password()}</Label>
			<Input
				id="profile-current-password"
				bind:value={draft.current_password}
				type="password"
				dir="ltr"
				autocomplete="current-password"
			/>
		</div>
		<div class="grid gap-2 py-4 sm:grid-cols-[12rem_minmax(0,1fr)] sm:items-start sm:gap-6">
			<Label for="profile-new-password">{m.settings_new_password()}</Label>
			<div class="grid gap-1">
				<Input
					id="profile-new-password"
					bind:value={draft.new_password}
					type="password"
					dir="ltr"
					autocomplete="new-password"
				/>
				<p class="text-xs text-muted-foreground">{m.settings_password_change_hint()}</p>
			</div>
		</div>
	</form>
{/if}
<SettingsBottomBar visible={dirty} isSaving={saving} onsave={save} onreset={reset} />
