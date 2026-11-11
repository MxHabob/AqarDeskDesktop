export type ViewKey =
	| 'overview'
	| 'properties'
	| 'units'
	| 'tenants'
	| 'contracts'
	| 'payments'
	| 'users'
	| 'settings';

export function getViewPath(organizationId: string, view: ViewKey) {
	if (view === 'overview') return `/${organizationId}`;
	if (view === 'users') return `/${organizationId}/settings/users`;
	return `/${organizationId}/${view}`;
}
