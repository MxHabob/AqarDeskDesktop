const contractDocumentRoot = (organizationId: string) =>
	['contract-documents', organizationId] as const;

export const contractDocumentQueryKeys = {
	all: contractDocumentRoot,
	pages: (organizationId: string, propertyId?: string) =>
		[...contractDocumentRoot(organizationId), 'pages', propertyId ?? null] as const,
	property: (organizationId: string, propertyId: string) =>
		[...contractDocumentRoot(organizationId), 'property', propertyId] as const
};
