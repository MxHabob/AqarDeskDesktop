import { desktopApi } from '#lib/api/desktop';

export const dashboardQueryKeys = {
	summary: (organizationId: string, months: number) =>
		['dashboard', organizationId, 'summary', months] as const,
	propertySummary: (organizationId: string, propertyId: string) =>
		['dashboard', organizationId, 'property', propertyId] as const
};

export const dashboardQueries = {
	summary: (months: number) => desktopApi.getDashboardSummary(months),
	propertySummary: (propertyId: string) => desktopApi.getPropertyDashboardSummary(propertyId)
};
