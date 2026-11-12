import { desktopApi } from '#lib/api/desktop';

export const userQueryKeys = {
	all: (organizationId: string) => ['users', organizationId] as const,
	pages: (organizationId: string) => ['users', organizationId, 'pages'] as const
};

export const userQueries = {
	list: desktopApi.listUsers
};
