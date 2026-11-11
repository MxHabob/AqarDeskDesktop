import { invoke } from '@tauri-apps/api/core';
import type {
	Property,
	PropertyInput,
	PropertyUnit,
	PropertyUnitInput,
	AuditEntry,
	ContractTemplate,
	ContractTemplateInput,
	CurrencySettings,
	OrganizationSettings,
	OrganizationSettingsInput,
	Payment,
	PaymentInput,
	ProfileSettings,
	ProfileSettingsInput,
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
	posted_payments_count: number;
	payment_months: string[];
	monthly_payments: {
		month: string;
		currency_code: string;
		currency_decimal_digits: number;
		amount_minor: number;
		payment_count: number;
	}[];
	unit_status_counts: { status: string; count: number }[];
};

export const desktopApi = {
	getConnection: () => invoke<ConnectionConfig | null>('platform_connection_get_config'),
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
	}) => invoke<ConnectionConfig>('platform_connection_set_config', { input }),
	connect: () => invoke<void>('platform_connection_connect'),
	checkApi: (serverUrl: string) => invoke<void>('platform_connection_check_api', { serverUrl }),
	getSetupStatus: () =>
		invoke<{ needs_admin: boolean; current_user: User | null }>('platform_auth_get_setup_status'),
	createAdmin: (input: {
		organizationName: string;
		name: string;
		email: string;
		password: string;
	}) => invoke<User>('platform_auth_bootstrap_admin', input),
	signIn: (email: string, password: string) =>
		invoke<User>('platform_auth_login', { email, password }),
	signOut: () => invoke<void>('platform_auth_logout'),
	getOrganizationName: () => invoke<string>('platform_organization_get_name'),
	getOrganizationSettings: () => invoke<OrganizationSettings>('organization_get_settings'),
	updateOrganizationSettings: (input: OrganizationSettingsInput) =>
		invoke<OrganizationSettings>('organization_update_settings', { input }),
	listCurrencies: () => invoke<CurrencySettings[]>('currencies_list'),
	setOrganizationCurrencyEnabled: (currencyCode: string, isEnabled: boolean) =>
		invoke<void>('currencies_set_organization_enabled', { currencyCode, isEnabled }),
	getProfileSettings: () => invoke<ProfileSettings>('profile_get_settings'),
	updateProfileSettings: (input: ProfileSettingsInput) =>
		invoke<ProfileSettings>('profile_update_settings', { input }),
	listAuditEvents: (limit = 50, offset = 0) =>
		invoke<AuditEntry[]>('audit_list', { limit, offset }),
	listContractTemplates: () => invoke<ContractTemplate[]>('contract_templates_list'),
	createContractTemplate: (input: ContractTemplateInput) =>
		invoke<ContractTemplate>('contract_templates_create', { input }),
	updateContractTemplate: (templateId: string, input: ContractTemplateInput) =>
		invoke<ContractTemplate>('contract_templates_update', { templateId, input }),
	setContractTemplateActive: (templateId: string, isActive: boolean) =>
		invoke<void>('contract_templates_set_active', { templateId, isActive }),
	getDashboardSummary: (months: number) => invoke<DashboardSummary>('dashboard_get_summary', { months }),
	listUsers: () => invoke<User[]>('platform_users_list'),
	createUser: (input: { name: string; email: string; password: string; role: string }) =>
		invoke<void>('platform_users_create', input),
	setUserActive: (userId: string, isActive: boolean) =>
		invoke<void>('platform_users_set_active', { userId, isActive }),
	listProperties: () => invoke<Property[]>('platform_properties_list'),
	createProperty: (input: PropertyInput) =>
		invoke<Property>('platform_properties_create', { input }),
	updateProperty: (propertyId: string, input: PropertyInput) =>
		invoke<Property>('platform_properties_update', { propertyId, input }),
	archiveProperty: (propertyId: string) =>
		invoke<void>('platform_properties_archive', { propertyId }),
	listUnits: () => invoke<PropertyUnit[]>('platform_units_list', { propertyId: null }),
	createUnit: (input: PropertyUnitInput) =>
		invoke<PropertyUnit>('platform_units_create', { input }),
	updateUnit: (unitId: string, input: PropertyUnitInput) =>
		invoke<PropertyUnit>('platform_units_update', { unitId, input }),
	archiveUnit: (unitId: string) => invoke<void>('platform_units_archive', { unitId }),
	listTenants: () => invoke<Tenant[]>('platform_tenants_list'),
	createTenant: (input: TenantInput) =>
		invoke<Tenant>('platform_tenants_create', { input }),
	updateTenant: (tenantId: string, input: TenantInput) =>
		invoke<Tenant>('platform_tenants_update', { tenantId, input }),
	archiveTenant: (tenantId: string) => invoke<void>('platform_tenants_archive', { tenantId }),
	listPayments: () => invoke<Payment[]>('platform_payments_list'),
	recordPayment: (input: PaymentInput) =>
		invoke<Payment>('platform_payments_record', { input }),
	voidPayment: (paymentId: string) => invoke<void>('platform_payments_void', { paymentId }),
	checkForUpdates: () => invoke<UpdateResult>('system_check_for_updates'),
	downloadUpdate: () => invoke<string>('system_download_update'),
	installUpdate: () => invoke<void>('system_install_update')
};
