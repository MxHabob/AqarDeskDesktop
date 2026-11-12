export const unitQueryKeys = {
	all: (organizationId: string) => ['units', organizationId] as const,
	pages: (organizationId: string, propertyId?: string) =>
		['units', organizationId, 'pages', propertyId ?? null] as const
};
