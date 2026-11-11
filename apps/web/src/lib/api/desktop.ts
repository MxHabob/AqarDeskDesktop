import { invoke } from '@tauri-apps/api/core';
import type {
	Property,
	PropertyInput,
	PropertyUnit,
	PropertyUnitInput,
	Tenant,
	TenantInput
} from '@aqardesk/sdk';
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

export type DashboardSummary = {
	active_properties_count: number;
	units_count: number;
	available_units_count: number;
	tenants_count: number;
	active_leases_count: number;
	open_charges_count: number;
	posted_payments_count: number;
	contract_documents_count: number;
	expiring_leases_count: number;
	payment_months: string[];
	monthly_payments: {
		month: string;
		currency_code: string;
		currency_decimal_digits: number;
		amount_minor: number;
		payment_count: number;
	}[];
	unit_status_counts: { status: string; count: number }[];
	lease_status_counts: { status: string; count: number }[];
	open_charges_by_currency: {
		currency_code: string;
		currency_decimal_digits: number;
		amount_minor: number;
		charge_count: number;
	}[];
};

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
	getOrganizationName: () => invoke<string>('organization_get_name'),
	getDashboardSummary: (months: number) => invoke<DashboardSummary>('dashboard_get_summary', { months }),
	listUsers: () => invoke<User[]>('users_list'),
	createUser: (input: { name: string; email: string; password: string; role: string }) =>
		invoke<void>('users_create', input),
	setUserActive: (userId: string, isActive: boolean) =>
		invoke<void>('users_set_active', { userId, isActive }),
	listProperties: () => invoke<Property[]>('properties_list'),
	createProperty: (input: PropertyInput) => invoke<Property>('properties_create', { input }),
	updateProperty: (propertyId: string, input: PropertyInput) =>
		invoke<Property>('properties_update', { propertyId, input }),
	archiveProperty: (propertyId: string) => invoke<void>('properties_archive', { propertyId }),
	listUnits: () => invoke<PropertyUnit[]>('units_list'),
	createUnit: (input: PropertyUnitInput) => invoke<PropertyUnit>('units_create', { input }),
	updateUnit: (unitId: string, input: PropertyUnitInput) =>
		invoke<PropertyUnit>('units_update', { unitId, input }),
	archiveUnit: (unitId: string) => invoke<void>('units_archive', { unitId }),
	listTenants: () => invoke<Tenant[]>('tenants_list'),
	createTenant: (input: TenantInput) => invoke<Tenant>('tenants_create', { input }),
	updateTenant: (tenantId: string, input: TenantInput) =>
		invoke<Tenant>('tenants_update', { tenantId, input }),
	archiveTenant: (tenantId: string) => invoke<void>('tenants_archive', { tenantId }),
	checkForUpdates: () => invoke<UpdateResult>('system_check_for_updates'),
	downloadUpdate: () => invoke<string>('system_download_update'),
	installUpdate: () => invoke<void>('system_install_update')
};
