export const ownerQueryKeys = {
	all: (organizationId: string) => ['owners', organizationId] as const,
	pages: (organizationId: string) => ['owners', organizationId, 'pages'] as const
};
