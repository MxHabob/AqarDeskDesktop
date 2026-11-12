<script lang="ts">
	import { Database, FolderOpen, Globe2, LoaderCircle, Server } from '@lucide/svelte';
	import { save } from '@tauri-apps/plugin-dialog';
	import {
		Button,
		Card,
		Input,
		Select,
		SelectContent,
		SelectItem,
		SelectTrigger,
		SelectValue,
		toast
	} from '@aqardesk/ui';
	import type { ConnectionMode } from '#lib/api/desktop';
	import * as m from '#lib/paraglide/messages';
	import FormField from '#lib/shared/components/form-field.svelte';

	type Props = {
		desktopShell: boolean;
		mode: ConnectionMode;
		databasePath: string;
		serverUrl: string;
		host: string;
		port: number;
		databaseName: string;
		username: string;
		password: string;
		sslMode: string;
		busy: boolean;
		onSubmit: (event: SubmitEvent) => void;
		onApiCheck: () => void;
		onModeChange: (mode: ConnectionMode) => void;
		onDatabasePathChange: (value: string) => void;
		onServerUrlChange: (value: string) => void;
		onHostChange: (value: string) => void;
		onPortChange: (value: number) => void;
		onDatabaseNameChange: (value: string) => void;
		onUsernameChange: (value: string) => void;
		onPasswordChange: (value: string) => void;
		onSslModeChange: (value: string) => void;
	};

	let {
		desktopShell,
		mode,
		databasePath,
		serverUrl,
		host,
		port,
		databaseName,
		username,
		password,
		sslMode,
		busy,
		onSubmit,
		onApiCheck,
		onModeChange,
		onDatabasePathChange,
		onServerUrlChange,
		onHostChange,
		onPortChange,
		onDatabaseNameChange,
		onUsernameChange,
		onPasswordChange,
		onSslModeChange
	}: Props = $props();

	async function chooseDatabasePath() {
		try {
			const path = await save({
				title: m.database_choose_location(),
				defaultPath: databasePath || 'aqardesk.sqlite',
				filters: [{ name: 'SQLite database', extensions: ['sqlite', 'sqlite3', 'db'] }]
			});
			if (path) onDatabasePathChange(path);
		} catch (cause) {
			toast.error(cause instanceof Error ? cause.message : String(cause));
		}
	}
</script>

<Card class="rounded-xl border-border/70 bg-card p-5 shadow-card sm:p-6">
	<form class="grid gap-5" onsubmit={onSubmit}>
		<div role="group" class="grid gap-2 sm:grid-cols-3" aria-label={m.connection_mode_label()}>
			<Button
				variant="outline"
				class={mode === 'local'
					? 'h-auto min-h-20 flex-col items-center justify-center gap-2 rounded-lg border-primary bg-accent px-2 py-3 text-center text-accent-foreground shadow-sm'
					: 'h-auto min-h-20 flex-col items-center justify-center gap-2 rounded-lg border-border/70 px-2 py-3 text-center hover:border-primary/50'}
				aria-pressed={mode === 'local'}
				disabled={busy}
				onclick={() => onModeChange('local')}
			>
				<Database size={18} class={mode === 'local' ? 'text-primary' : 'text-muted-foreground'} />
				<strong class="text-sm leading-tight">{m.connection_local()}</strong>
			</Button>
			<Button
				variant="outline"
				class={mode === 'direct'
					? 'h-auto min-h-20 flex-col items-center justify-center gap-2 rounded-lg border-primary bg-accent px-2 py-3 text-center text-accent-foreground shadow-sm'
					: 'h-auto min-h-20 flex-col items-center justify-center gap-2 rounded-lg border-border/70 px-2 py-3 text-center hover:border-primary/50'}
				aria-pressed={mode === 'direct'}
				disabled={busy}
				onclick={() => onModeChange('direct')}
			>
				<Server size={18} class={mode === 'direct' ? 'text-primary' : 'text-muted-foreground'} />
				<strong class="text-sm leading-tight">{m.connection_direct()}</strong>
			</Button>
			<Button
				variant="outline"
				class={mode === 'api'
					? 'h-auto min-h-20 flex-col items-center justify-center gap-2 rounded-lg border-primary bg-accent px-2 py-3 text-center text-accent-foreground shadow-sm'
					: 'h-auto min-h-20 flex-col items-center justify-center gap-2 rounded-lg border-border/70 px-2 py-3 text-center hover:border-primary/50'}
				aria-pressed={mode === 'api'}
				disabled={busy}
				onclick={() => onModeChange('api')}
			>
				<Globe2 size={18} class={mode === 'api' ? 'text-primary' : 'text-muted-foreground'} />
				<strong class="text-sm leading-tight">{m.connection_api()}</strong>
			</Button>
		</div>

		{#if mode === 'local'}
			<FormField label={m.connection_database_file()} id="database-path">
				<div class="flex min-w-0 gap-2">
					<Input
						id="database-path"
						value={databasePath}
						oninput={(event) => onDatabasePathChange(event.currentTarget.value)}
						required
						disabled={busy}
						dir="ltr"
						autocomplete="off"
						class="min-w-0 rounded-md font-mono text-xs"
					/>
					{#if desktopShell}
						<Button
							variant="outline"
							class="h-9 shrink-0 rounded-md"
							type="button"
							disabled={busy}
							onclick={chooseDatabasePath}
						>
							<FolderOpen size={16} />
							<span>{m.connection_choose_file()}</span>
						</Button>
					{/if}
				</div>
			</FormField>
		{:else if mode === 'api'}
			<FormField label={m.connection_server_url()} id="server-url">
				<Input
					id="server-url"
					type="url"
					value={serverUrl}
					oninput={(event) => onServerUrlChange(event.currentTarget.value)}
					required
					disabled={busy}
					dir="ltr"
					placeholder="https://api.example.com"
					autocomplete="url"
					class="rounded-md"
				/>
			</FormField>
		{:else}
			<div class="grid gap-4 sm:grid-cols-2">
				<FormField label={m.connection_host()} id="database-host">
					<Input
						id="database-host"
						value={host}
						oninput={(event) => onHostChange(event.currentTarget.value)}
						required
						disabled={busy}
						dir="ltr"
						autocomplete="off"
						class="rounded-md"
					/>
				</FormField>
				<FormField label={m.connection_port()} id="database-port">
					<Input
						id="database-port"
						type="number"
						value={port}
						oninput={(event) => onPortChange(Number(event.currentTarget.value))}
						min="1"
						max="65535"
						required
						disabled={busy}
						dir="ltr"
						class="rounded-md"
					/>
				</FormField>
				<FormField label={m.connection_database_name()} id="database-name">
					<Input
						id="database-name"
						value={databaseName}
						oninput={(event) => onDatabaseNameChange(event.currentTarget.value)}
						required
						disabled={busy}
						dir="ltr"
						autocomplete="off"
						class="rounded-md"
					/>
				</FormField>
				<FormField label={m.connection_username()} id="database-username">
					<Input
						id="database-username"
						value={username}
						oninput={(event) => onUsernameChange(event.currentTarget.value)}
						required
						disabled={busy}
						dir="ltr"
						autocomplete="username"
						class="rounded-md"
					/>
				</FormField>
				<FormField label={m.connection_password()} id="database-password">
					<Input
						id="database-password"
						type="password"
						value={password}
						oninput={(event) => onPasswordChange(event.currentTarget.value)}
						disabled={busy}
						dir="ltr"
						autocomplete="new-password"
						class="rounded-md"
					/>
				</FormField>
				<FormField label={m.connection_ssl_mode()} id="ssl-mode">
					<Select
						type="single"
						value={sslMode}
						items={[
							{ value: 'prefer', label: 'prefer' },
							{ value: 'require', label: 'require' },
							{ value: 'disable', label: 'disable' }
						]}
						disabled={busy}
						onValueChange={onSslModeChange}
					>
						<SelectTrigger id="ssl-mode" class="h-9 w-full rounded-md text-sm">
							<SelectValue />
						</SelectTrigger>
						<SelectContent>
							<SelectItem value="prefer" label="prefer">prefer</SelectItem>
							<SelectItem value="require" label="require">require</SelectItem>
							<SelectItem value="disable" label="disable">disable</SelectItem>
						</SelectContent>
					</Select>
				</FormField>
			</div>
		{/if}

		<div class="flex flex-col-reverse gap-2 sm:flex-row sm:justify-end">
			{#if mode === 'api'}
				<Button
					variant="outline"
					class="h-10 rounded-md"
					type="button"
					disabled={busy}
					onclick={onApiCheck}
				>
					{#if busy}<LoaderCircle size={16} class="animate-spin" />{/if}
					{m.connection_test_api()}
				</Button>
			{/if}
			<Button class="h-10 rounded-md px-5 sm:min-w-40" type="submit" disabled={busy}>
				{#if busy}<LoaderCircle size={16} class="animate-spin" />{/if}
				{busy ? m.common_processing() : m.common_connect()}
			</Button>
		</div>
	</form>
</Card>
