export const propertyQueryKeys = {
	all: (organizationId: string) => ['properties', organizationId] as const,
	pages: (organizationId: string) => ['properties', organizationId, 'pages'] as const
};
