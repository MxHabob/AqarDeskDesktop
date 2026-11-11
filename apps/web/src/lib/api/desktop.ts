import { invoke } from '@tauri-apps/api/core';
import type { User } from '#lib/features/auth/types';

export type ConnectionMode = 'local' | 'direct' | 'api';

export type ConnectionConfig = {
	mode: ConnectionMode;
	local?: { database_path: string } | null;
	direct?: {
		host: string;
		port: number;
		database_name: string;
		username: string;
		ssl_mode?: string | null;
	} | null;
	api?: { server_url: string } | null;
};

export type UpdateResult = {
	current_version: string;
	latest_version: string;
	has_update: boolean;
	release_notes: string | null;
};

export type UpdateProgress = { downloaded_bytes: number; total_bytes: number | null };

export const desktopApi = {
	getConnection: () => invoke<ConnectionConfig | null>('connection_get_config'),
	setConnection: (input: {
		mode: ConnectionMode;
		databasePath?: string | null;
		serverUrl?: string | null;
		host?: string | null;
		port?: number | null;
		databaseName?: string | null;
		username?: string | null;
		password?: string | null;
		sslMode?: string | null;
	}) => invoke<ConnectionConfig>('connection_set_config', input),
	connect: () => invoke<void>('connection_connect'),
	checkApi: (serverUrl: string) => invoke<void>('connection_check_api', { serverUrl }),
	getSetupStatus: () =>
		invoke<{ needs_admin: boolean; current_user: User | null }>('auth_get_setup_status'),
	createAdmin: (input: {
		organizationName: string;
		name: string;
		email: string;
		password: string;
	}) => invoke<User>('auth_bootstrap_admin', input),
	signIn: (email: string, password: string) => invoke<User>('auth_login', { email, password }),
	signOut: () => invoke<void>('auth_logout'),
	listUsers: () => invoke<User[]>('users_list'),
	createUser: (input: { name: string; email: string; password: string; role: string }) =>
		invoke<void>('users_create', input),
	setUserActive: (userId: string, isActive: boolean) =>
		invoke<void>('users_set_active', { userId, isActive }),
	checkForUpdates: () => invoke<UpdateResult>('system_check_for_updates'),
	downloadUpdate: () => invoke<string>('system_download_update'),
	installUpdate: () => invoke<void>('system_install_update')
};
