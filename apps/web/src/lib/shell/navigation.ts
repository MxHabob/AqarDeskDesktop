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
	return view === 'overview' ? `/${organizationId}` : `/${organizationId}/${view}`;
}
