type NextPageResult = {
	isError?: boolean;
	isFetchNextPageError?: boolean;
	error?: unknown;
};

export async function fetchNextPageOrThrow<Result extends NextPageResult>(
	fetchNextPage: () => Promise<Result>
): Promise<void> {
	const result = await fetchNextPage();
	if (!result.isError && !result.isFetchNextPageError) return;
	throw result.error instanceof Error
		? result.error
		: new Error(String(result.error ?? 'Failed to load the next page.'));
}
