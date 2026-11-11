import { desktopApi } from '#lib/api/desktop';

export const dashboardQueryKeys = {
	summary: (organizationId: string, months: number) =>
		['dashboard', organizationId, 'summary', months] as const
};

export const dashboardQueries = {
	summary: (months: number) => desktopApi.getDashboardSummary(months)
};
