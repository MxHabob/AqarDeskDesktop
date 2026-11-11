<script lang="ts">
	import { onMount } from 'svelte';
	import { z } from 'zod';
	import { RotateCcw } from '@lucide/svelte';
	import {
		Button,
		Input,
		Select,
		SelectContent,
		SelectItem,
		SelectTrigger,
		SelectValue,
		toast
	} from '@aqardesk/ui';
	import type { ProfileSettings, ProfileSettingsInput } from '@aqardesk/sdk';
	import { desktopApi } from '#lib/api/desktop';
	import { getAppContext } from '#lib/core/app-context.svelte';
	import SettingsBottomBar from '#lib/shared/components/settings-bottom-bar.svelte';
	import SettingsFieldRow from '#lib/shared/components/settings-field-row.svelte';
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

	function resetAppearance() {
		if (app.resetAppearance()) toast.success(m.account_appearance_reset_success());
	}

	function previewToastPosition(value: string | undefined) {
		if (
			value === 'top-left' ||
			value === 'top-center' ||
			value === 'top-right' ||
			value === 'bottom-left' ||
			value === 'bottom-center' ||
			value === 'bottom-right'
		) {
			if (app.changeToastPosition(value)) {
				toast.success(m.account_toast_preview(), { duration: 2500 });
			}
		}
	}

	const roleLabel = $derived(
		saved?.role === 'admin'
			? m.users_role_admin()
			: saved?.role === 'staff'
				? m.users_role_staff()
				: m.users_role_readonly()
	);
</script>

{#if loading}
	<div class="py-8 text-sm text-muted-foreground" role="status">{m.common_processing()}</div>
{:else if saved}
	<div class="grid gap-5 pb-24">
		<form
			class="grid min-w-0"
			onsubmit={(event) => {
				event.preventDefault();
				void save();
			}}
		>
			<section class="overflow-hidden rounded-lg border border-border/70 bg-card">
				<div class="divide-y divide-border/70">
					<SettingsFieldRow label={m.users_name()} id="profile-name">
						<Input id="profile-name" bind:value={draft.name} autocomplete="name" class="w-full" />
					</SettingsFieldRow>
					<SettingsFieldRow label={m.auth_email()} id="profile-email">
						<Input
							id="profile-email"
							bind:value={draft.email}
							type="email"
							dir="ltr"
							autocomplete="email"
							class="w-full"
						/>
					</SettingsFieldRow>
					<SettingsFieldRow label={m.users_role()}>
						<p class="text-sm">{roleLabel}</p>
					</SettingsFieldRow>
					<SettingsFieldRow label={m.settings_current_password()} id="profile-current-password">
						<Input
							id="profile-current-password"
							bind:value={draft.current_password}
							type="password"
							dir="ltr"
							autocomplete="current-password"
							class="w-full"
						/>
					</SettingsFieldRow>
					<SettingsFieldRow
						label={m.settings_new_password()}
						id="profile-new-password"
						description={m.settings_password_change_hint()}
					>
						<Input
							id="profile-new-password"
							bind:value={draft.new_password}
							type="password"
							dir="ltr"
							autocomplete="new-password"
							class="w-full"
						/>
					</SettingsFieldRow>
				</div>
			</section>
		</form>

		<section class="overflow-hidden rounded-lg border border-border/70 bg-card">
			<div class="flex items-center justify-between gap-3 border-b border-border/70 bg-muted/40 px-4 py-3">
				<h2 class="text-xs font-medium text-muted-foreground">{m.account_appearance_title()}</h2>
				<Button variant="outline" size="sm" onclick={resetAppearance}>
					<RotateCcw size={14} />
					{m.account_appearance_reset()}
				</Button>
			</div>
			<div class="divide-y divide-border/70">
				<SettingsFieldRow
					label={m.account_theme()}
					description={m.account_theme_description()}
					id="account-theme"
				>
					<Select
						type="single"
						value={app.theme}
						onValueChange={(value) => {
							if (value === 'light' || value === 'dark') app.changeTheme(value);
						}}
					>
						<SelectTrigger id="account-theme" class="h-9 w-full rounded-md text-sm">
							<SelectValue />
						</SelectTrigger>
						<SelectContent>
							<SelectItem value="light">{m.account_theme_light()}</SelectItem>
							<SelectItem value="dark">{m.account_theme_dark()}</SelectItem>
						</SelectContent>
					</Select>
				</SettingsFieldRow>
				<SettingsFieldRow
					label={m.account_contrast()}
					description={m.account_contrast_description()}
					id="account-contrast"
				>
					<Select
						type="single"
						value={app.contrast}
						onValueChange={(value) => {
							if (value === 'normal' || value === 'high') app.changeContrast(value);
						}}
					>
						<SelectTrigger id="account-contrast" class="h-9 w-full rounded-md text-sm">
							<SelectValue />
						</SelectTrigger>
						<SelectContent>
							<SelectItem value="normal">{m.account_contrast_normal()}</SelectItem>
							<SelectItem value="high">{m.account_contrast_high()}</SelectItem>
						</SelectContent>
					</Select>
				</SettingsFieldRow>
				<SettingsFieldRow
					label={m.account_toast_position()}
					description={m.account_toast_position_description()}
					id="account-toast-position"
				>
					<Select
						type="single"
						value={app.toastPosition}
						onValueChange={previewToastPosition}
					>
						<SelectTrigger id="account-toast-position" class="h-9 w-full rounded-md text-sm">
							<SelectValue />
						</SelectTrigger>
						<SelectContent>
							<SelectItem value="top-left">{m.account_toast_top_left()}</SelectItem>
							<SelectItem value="top-center">{m.account_toast_top_center()}</SelectItem>
							<SelectItem value="top-right">{m.account_toast_top_right()}</SelectItem>
							<SelectItem value="bottom-left">{m.account_toast_bottom_left()}</SelectItem>
							<SelectItem value="bottom-center">{m.account_toast_bottom_center()}</SelectItem>
							<SelectItem value="bottom-right">{m.account_toast_bottom_right()}</SelectItem>
						</SelectContent>
					</Select>
				</SettingsFieldRow>
			</div>
		</section>
	</div>
{/if}
<SettingsBottomBar visible={dirty} isSaving={saving} onsave={save} onreset={reset} />
