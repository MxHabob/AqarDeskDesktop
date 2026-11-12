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
	Owner,
	OwnerInput,
	Tenant,
	TenantInput
} from '@aqardesk/sdk';
import type { User } from '#lib/features/auth/types';
import type { Permission } from '#lib/features/auth/permissions';

export type RolePermissionSet = {
	role_id: string;
	name: string;
	is_system: boolean;
	system_role: 'admin' | 'staff' | 'readonly' | null;
	permissions: Permission[];
};

export type ContractDocument = {
	id: string;
	organization_id: string;
	contract_number: string;
	template_id: string;
	template_version_id: string;
	template_name: string;
	template_locale: string;
	property_id: string;
	property_name: string;
	property_code: string;
	unit_id: string;
	unit_number: string;
	tenant_id: string;
	tenant_name: string;
	start_date: string;
	end_date: string;
	currency_code: string;
	currency_decimal_digits: number;
	rent_amount_minor: number;
	deposit_amount_minor: number;
	document_body: string;
	created_by: string;
	created_at: string;
};

export type ContractDocumentInput = {
	template_id: string;
	tenant_id: string;
	property_id: string;
	unit_id: string;
	start_date: string;
	end_date: string;
	currency_code: string;
	rent_amount_minor: number;
	deposit_amount_minor: number;
};

export type PaymentContract = {
	id: string;
	contract_number: string;
	property_name: string;
	property_code: string;
	unit_number: string;
	start_date: string;
	end_date: string;
	currency_code: string;
	currency_decimal_digits: number;
	rent_amount_minor: number;
};

export type ConnectionMode = 'local' | 'direct' | 'api';

export type CurrencyDetailsInput = {
	name: string;
	name_ar?: string | null;
	symbol: string;
	symbol_position: 'prefix' | 'suffix';
	decimal_digits: number;
};

export type NewCurrencyInput = CurrencyDetailsInput & {
	code: string;
	enable_for_organization: boolean;
};

export type CurrencyEditorInput = NewCurrencyInput;

export type UserEditorInput = {
	name: string;
	email: string;
	role_id: string;
	password?: string;
};

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

export type LicenseState =
	| 'unavailable'
	| 'pending'
	| 'trial'
	| 'active'
	| 'expiring'
	| 'offline_grace'
	| 'deactivation_pending'
	| 'expired'
	| 'suspended'
	| 'revoked'
	| 'invalid';

export type LicenseLimit = {
	key: string;
	max_value: number | null;
	unit: string;
};

export type LicenseStateDto = {
	state: LicenseState;
	product: string;
	plan: string | null;
	plan_name: string | null;
	organization_id: string | null;
	valid_until: string | null;
	days_until_expiry: number | null;
	trial_days_remaining: number;
	is_operational: boolean;
	is_restricted: boolean;
	is_configured: boolean;
	configuration_error: string | null;
	last_error: string | null;
	installation_id: string;
	last_sync_at: string | null;
	enabled_features: string[];
	trial_features: string[];
	limits: LicenseLimit[];
	app_version: string;
};

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
	payments_by_currency: {
		currency_code: string;
		currency_decimal_digits: number;
		amount_minor: number;
		payment_count: number;
	}[];
	unit_status_counts: { status: string; count: number }[];
	payment_status_counts: { status: string; count: number }[];
};

export type PropertyDashboardSummary = {
	contract_count: number;
	tenant_count: number;
	rent_by_currency: CurrencyAmountTotal[];
	posted_allocations_by_currency: CurrencyAmountTotal[] | null;
};

export type CurrencyAmountTotal = {
	currency_code: string;
	currency_decimal_digits: number;
	amount_minor: number;
};

export type ApiPage<T> = {
	items: T[];
	has_more: boolean;
};

export type DatabaseAdminStatus = {
	mode: ConnectionMode;
	database_path: string | null;
	file_size_bytes: number | null;
	modified_at_unix: number | null;
	supports_file_management: boolean;
	remote_database: string | null;
};

export type DatabaseBackup = {
	name: string;
	path: string;
	size_bytes: number;
	modified_at_unix: number;
};

export type DatabaseOperationResult = {
	path: string;
	size_bytes: number;
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
	getDatabaseAdminStatus: () => invoke<DatabaseAdminStatus>('database_admin_get_status'),
	listDatabaseBackups: () => invoke<DatabaseBackup[]>('database_admin_list_backups'),
	createDatabaseBackup: (destinationPath: string) =>
		invoke<DatabaseOperationResult>('database_admin_backup', { destinationPath }),
	restoreDatabaseBackup: (sourcePath: string) =>
		invoke<DatabaseOperationResult>('database_admin_restore', { sourcePath }),
	checkDatabaseIntegrity: () => invoke<string>('database_admin_check_integrity'),
	changeDatabaseLocation: (destinationPath: string) =>
		invoke<DatabaseOperationResult>('database_admin_change_location', { destinationPath }),
	checkApi: (serverUrl: string) => invoke<void>('platform_connection_check_api', { serverUrl }),
	getSetupStatus: () =>
		invoke<{ needs_admin: boolean; current_user: User | null }>('platform_auth_get_setup_status'),
	getLicenseState: (organizationId: string) =>
		invoke<LicenseStateDto>('license_get_state', { organizationId }),
	activateLicense: (productKey: string) =>
		invoke<LicenseStateDto>('license_activate_key', { productKey }),
	startLicenseTrial: () => invoke<LicenseStateDto>('license_start_trial'),
	refreshLicense: () => invoke<LicenseStateDto>('license_refresh'),
	deactivateLicense: () => invoke<LicenseStateDto>('license_deactivate'),
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
	listCurrenciesPage: (limit: number, offset: number) =>
		invoke<ApiPage<CurrencySettings>>('currencies_list_page', { limit, offset }),
	createCurrency: (input: NewCurrencyInput) => invoke<void>('currencies_create', { input }),
	updateCurrency: (currencyCode: string, input: CurrencyDetailsInput) =>
		invoke<void>('currencies_update', { currencyCode, input }),
	setOrganizationCurrencyEnabled: (currencyCode: string, isEnabled: boolean) =>
		invoke<void>('currencies_set_organization_enabled', { currencyCode, isEnabled }),
	getProfileSettings: () => invoke<ProfileSettings>('profile_get_settings'),
	updateProfileSettings: (input: ProfileSettingsInput) =>
		invoke<ProfileSettings>('profile_update_settings', { input }),
	listAuditEvents: (limit = 50, offset = 0) =>
		invoke<AuditEntry[]>('audit_list', { limit, offset }),
	listUsersPage: (limit: number, offset: number) =>
		invoke<ApiPage<User>>('platform_users_list_page', { limit, offset }),
	listContractTemplates: () => invoke<ContractTemplate[]>('contract_templates_list'),
	listContractTemplatesPage: (limit: number, offset: number) =>
		invoke<ApiPage<ContractTemplate>>('contract_templates_list_page', { limit, offset }),
	listAvailableContractTemplates: () =>
		invoke<ContractTemplate[]>('contract_templates_available'),
	createContractTemplate: (input: ContractTemplateInput) =>
		invoke<ContractTemplate>('contract_templates_create', { input }),
	updateContractTemplate: (templateId: string, input: ContractTemplateInput) =>
		invoke<ContractTemplate>('contract_templates_update', { templateId, input }),
	setContractTemplateActive: (templateId: string, isActive: boolean) =>
		invoke<void>('contract_templates_set_active', { templateId, isActive }),
	listContractDocuments: (propertyId?: string) =>
		invoke<ContractDocument[]>('contract_documents_list', { propertyId: propertyId ?? null }),
	listContractDocumentsPage: (limit: number, offset: number, propertyId?: string) =>
		invoke<ApiPage<ContractDocument>>('contract_documents_list_page', {
			limit,
			offset,
			propertyId: propertyId ?? null
		}),
	listPaymentContracts: (tenantId: string) =>
		invoke<PaymentContract[]>('platform_payments_contracts', { tenantId }),
	createContractDocument: (input: ContractDocumentInput) =>
		invoke<ContractDocument>('contract_documents_create', { input }),
	getDashboardSummary: (months: number) => invoke<DashboardSummary>('dashboard_get_summary', { months }),
	getPropertyDashboardSummary: (propertyId: string) =>
		invoke<PropertyDashboardSummary>('dashboard_get_property_summary', { propertyId }),
	listUsers: () => invoke<User[]>('platform_users_list'),
	listRolePermissions: () =>
		invoke<RolePermissionSet[]>('platform_role_permissions_list'),
	listRolePermissionsPage: (limit: number, offset: number) =>
		invoke<ApiPage<RolePermissionSet>>('platform_role_permissions_list_page', { limit, offset }),
	updateRolePermissions: (roleId: string, permissions: Permission[]) =>
		invoke<RolePermissionSet>('platform_role_permissions_update', { roleId, permissions }),
	createRole: (name: string, permissions: Permission[]) =>
		invoke<RolePermissionSet>('platform_roles_create', { name, permissions }),
	deleteRole: (roleId: string) => invoke<void>('platform_roles_delete', { roleId }),
	createUser: ({ role_id, ...input }: { name: string; email: string; password: string; role_id: string }) =>
		invoke<User>('platform_users_create', { ...input, roleId: role_id }),
	updateUser: (userId: string, { role_id, ...input }: UserEditorInput) =>
		invoke<User>('platform_users_update', { userId, ...input, roleId: role_id }),
	setUserActive: (userId: string, isActive: boolean) =>
		invoke<void>('platform_users_set_active', { userId, isActive }),
	listProperties: () => invoke<Property[]>('platform_properties_list'),
	listPropertiesPage: (limit: number, offset: number) =>
		invoke<ApiPage<Property>>('platform_properties_list_page', { limit, offset }),
	createProperty: (input: PropertyInput) =>
		invoke<Property>('platform_properties_create', { input }),
	updateProperty: (propertyId: string, input: PropertyInput) =>
		invoke<Property>('platform_properties_update', { propertyId, input }),
	archiveProperty: (propertyId: string) =>
		invoke<void>('platform_properties_archive', { propertyId }),
	listOwners: () => invoke<Owner[]>('platform_owners_list'),
	listOwnersPage: (limit: number, offset: number) =>
		invoke<ApiPage<Owner>>('platform_owners_list_page', { limit, offset }),
	createOwner: (input: OwnerInput) => invoke<Owner>('platform_owners_create', { input }),
	updateOwner: (ownerId: string, input: OwnerInput) =>
		invoke<Owner>('platform_owners_update', { ownerId, input }),
	listUnits: () => invoke<PropertyUnit[]>('platform_units_list', { propertyId: null }),
	listUnitsPage: (limit: number, offset: number, propertyId?: string) =>
		invoke<ApiPage<PropertyUnit>>('platform_units_list_page', {
			limit,
			offset,
			propertyId: propertyId ?? null
		}),
	createUnit: (input: PropertyUnitInput) =>
		invoke<PropertyUnit>('platform_units_create', { input }),
	updateUnit: (unitId: string, input: PropertyUnitInput) =>
		invoke<PropertyUnit>('platform_units_update', { unitId, input }),
	archiveUnit: (unitId: string) => invoke<void>('platform_units_archive', { unitId }),
	listTenants: () => invoke<Tenant[]>('platform_tenants_list'),
	listTenantsPage: (limit: number, offset: number) =>
		invoke<ApiPage<Tenant>>('platform_tenants_list_page', { limit, offset }),
	createTenant: (input: TenantInput) =>
		invoke<Tenant>('platform_tenants_create', { input }),
	updateTenant: (tenantId: string, input: TenantInput) =>
		invoke<Tenant>('platform_tenants_update', { tenantId, input }),
	archiveTenant: (tenantId: string) => invoke<void>('platform_tenants_archive', { tenantId }),
	listPayments: () => invoke<Payment[]>('platform_payments_list'),
	listPaymentsPage: (limit: number, offset: number, propertyId?: string) =>
		invoke<ApiPage<Payment>>('platform_payments_list_page', {
			limit,
			offset,
			propertyId: propertyId ?? null
		}),
	recordPayment: (input: PaymentInput) =>
		invoke<Payment>('platform_payments_record', { input }),
	voidPayment: (paymentId: string) => invoke<void>('platform_payments_void', { paymentId }),
	checkForUpdates: () => invoke<UpdateResult>('system_check_for_updates'),
	downloadUpdate: () => invoke<string>('system_download_update'),
	installUpdate: () => invoke<void>('system_install_update')
};
