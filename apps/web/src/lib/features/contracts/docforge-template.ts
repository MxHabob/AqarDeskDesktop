import {
	createDocument,
	createNode,
	deserializeDocument,
	generateNodeId,
	getPages,
	serializeDocument,
	type FooterNode,
	type HeaderNode,
	type PageSize,
	type TableNode,
	type TemplateDocument,
	type TemplateNode,
	type TextNode
} from '@docforge/core';

const pageSizes: Record<string, PageSize> = {
	a4: 'A4',
	a5: 'A5',
	letter: 'Letter'
};

export function docForgeDocumentFromTemplate(
	documentJson: string | null | undefined,
	content: string,
	locale: string,
	paperSize: 'a4' | 'a5' | 'letter'
): TemplateDocument {
	if (documentJson?.trim()) {
		const document = deserializeDocument(documentJson);
		assertDocForgeDocument(document);
		document.meta.pageSetup.size = pageSizes[paperSize];
		ensureUnpositionedTextNodes(document, paperSize);
		return document;
	}

	const direction = locale.toLowerCase().startsWith('ar') ? 'rtl' : 'ltr';
	const document = createDocument('contract', 'contract', direction === 'rtl' ? 'ar' : 'en');
	document.meta.pageSetup.size = pageSizes[paperSize];
	document.meta.pageSetup.margins = { top: 18, bottom: 18, left: 18, right: 18 };
	const pageWidthMm = paperSize === 'a5' ? 148 : paperSize === 'letter' ? 216 : 210;
	let contentY = 38;

	const header: HeaderNode = {
		type: 'Header',
		id: generateNodeId('header'),
		name: 'Header',
		props: { height: 24 },
		children: []
	};
	const footer: FooterNode = {
		type: 'Footer',
		id: generateNodeId('footer'),
		name: 'Footer',
		props: { height: 18, showPageNumber: true, pageNumberFormat: '{{page}}' },
		children: []
	};

	for (const [index, rawLine] of content.split(/\r?\n/).entries()) {
		const line = rawLine.trim();
		if (!line) continue;

		const heading = /^(#{1,3})\s+(.+)$/.exec(line);
		const listItem = /^(?:[-*+]\s+|\d+[.)]\s+)(.+)$/.exec(line);
		const level = heading?.[1]?.length ?? 0;
		const name = level === 1 ? 'Title' : level > 1 ? 'Heading' : listItem ? 'List item' : 'Paragraph';
		const text = heading?.[2] ?? listItem?.[1] ?? line;
		const height = level === 1 ? 12 : level > 1 ? 10 : 8;
		const node = createTextNode(text, name, direction, index, {
			x: 18,
			y: contentY,
			width: pageWidthMm - 36,
			height
		});
		document.pages[0]!.children.push(node);
		contentY += height + 4;
	}

	document.pages[0]!.children.unshift(header);
	document.pages[0]!.children.push(footer);
	return document;
}

export function docForgeDocumentFromPlaceholders(
	value: string | null | undefined,
	content: string,
	locale: string,
	paperSize: 'a4' | 'a5' | 'letter'
) {
	const documentJson = docForgeDocumentJsonFromPlaceholders(value);
	return docForgeDocumentFromTemplate(documentJson ?? undefined, content, locale, paperSize);
}

export function docForgeDocumentJsonFromPlaceholders(value: string | null | undefined) {
	if (!value?.trim()) return null;
	const parsed: unknown = JSON.parse(value);
	if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
		throw new Error('Contract template settings must be a JSON object.');
	}
	const document = (parsed as Record<string, unknown>).docforge_document;
	if (document === undefined) return null;
	if (typeof document !== 'string') {
		throw new Error('The saved DocForge template document must be a string.');
	}
	return document.trim() ? document : null;
}

export function docForgeDocumentToTemplateContent(document: TemplateDocument) {
	const lines: string[] = [];
	for (const [pageIndex, page] of getPages(document).entries()) {
		if (pageIndex > 0) lines.push('[[PAGE_BREAK]]', '');
		for (const node of page.children) {
			if (node.type !== 'Text') continue;
			const text = node.props.content.trim();
			if (!text) continue;
			const name = node.name?.toLowerCase() ?? '';
			if (node.name === 'Title' || name.endsWith('-title')) lines.push(`# ${text}`);
			else if (
				node.name === 'Heading' ||
				name.endsWith('-heading') ||
				name.includes('-clause-heading-')
			) {
				lines.push(`## ${text}`);
			}
			else if (node.name === 'List item') lines.push(`- ${text}`);
			else lines.push(text);
			lines.push('');
		}
	}
	return lines.join('\n').trim();
}

export function serializeDocForgeTemplate(document: TemplateDocument) {
	assertDocForgeDocument(document);
	return serializeDocument(document);
}

export function resolveDocForgeDocument(
	document: TemplateDocument,
	values: Record<string, unknown>
) {
	const unresolved = new Set<string>();
	const resolved = JSON.parse(JSON.stringify(document)) as TemplateDocument;

	function lookup(path: string, value: unknown = values): unknown {
		if (!path) return value;
		if (value && typeof value === 'object' && !Array.isArray(value)) {
			const record = value as Record<string, unknown>;
			if (Object.prototype.hasOwnProperty.call(record, path)) return record[path];
		}
		return path.split('.').reduce<unknown>((current, key) => {
			if (!current || typeof current !== 'object' || Array.isArray(current)) return undefined;
			return (current as Record<string, unknown>)[key];
		}, value);
	}

	function resolveString(text: string) {
		return text.replace(/\{\{\s*([a-zA-Z0-9_.-]+)\s*\}\}/g, (token, path: string) => {
			const value = lookup(path);
			if (value === undefined || value === null) {
				unresolved.add(path);
				return token;
			}
			if (Array.isArray(value)) {
				return value
					.map((item) => {
						if (!item || typeof item !== 'object') return String(item);
						const record = item as Record<string, unknown>;
						if (typeof record.display === 'string') return record.display;
						return Object.values(record).join(' · ');
					})
					.join('\n');
			}
			if (typeof value === 'object') {
				unresolved.add(path);
				return token;
			}
			return String(value);
		});
	}

	function resolveStrings(value: unknown) {
		if (typeof value === 'string') return resolveString(value);
		if (Array.isArray(value)) {
			for (let index = 0; index < value.length; index += 1) {
				value[index] = resolveStrings(value[index]);
			}
			return value;
		}
		if (value && typeof value === 'object') {
			const record = value as Record<string, unknown>;
			for (const [key, nested] of Object.entries(record)) {
				record[key] = resolveStrings(nested);
			}
		}
		return value;
	}

	function populateTableRows(nodes: TemplateNode[]) {
		for (const node of nodes) {
			if (node.type === 'Table') {
				const props = node.props as TableNode['props'] & {
					rows?: Record<string, unknown>[];
				};
				const sourceRows =
					typeof props.dataSource === 'string' ? lookup(props.dataSource) : undefined;
				if (Array.isArray(sourceRows)) {
					props.rows = sourceRows.map((row) => {
						const result: Record<string, unknown> = {};
						for (const column of props.columns) {
							if (column.field) result[column.field] = lookup(column.field, row);
						}
						return result;
					});
				} else if (props.staticRows) {
					props.rows = props.staticRows.map((row) =>
						Object.fromEntries(
							props.columns.flatMap((column, index) =>
								column.field ? [[column.field, row.cells[index] ?? '']] : []
							)
						)
					);
				}
			}
			if ('children' in node) populateTableRows(node.children);
			if (node.type === 'Conditional' && node.elseChildren) {
				populateTableRows(node.elseChildren);
			}
		}
	}

	populateTableRows(resolved.pages);
	resolveStrings(resolved);
	return { document: resolved, unresolved: [...unresolved] };
}

function createTextNode(
	content: string,
	name: string,
	direction: 'rtl' | 'ltr',
	index: number,
	position: { x: number; y: number; width: number; height: number }
): TextNode {
	const node = createNode<TextNode>('Text', {
		name,
		props: {
			content,
			position,
			fontSize: name === 'Title' ? 22 : name === 'Heading' ? 17 : 14,
			fontFamily: direction === 'rtl' ? 'Tahoma' : 'Arial',
			fontWeight: name === 'Paragraph' ? 'normal' : 'bold',
			color: name === 'Title' || name === 'Heading' ? '#17324d' : '#253244',
			textAlign: 'start',
			direction,
			lineHeight: 1.7
		}
	});
	node.id = generateNodeId(`text-${index}`);
	node.positionMode = 'absolute';
	return node;
}

function ensureUnpositionedTextNodes(
	document: TemplateDocument,
	paperSize: 'a4' | 'a5' | 'letter'
) {
	const pageWidthMm = paperSize === 'a5' ? 148 : paperSize === 'letter' ? 216 : 210;
	for (const page of document.pages) {
		let contentY = 38;
		for (const node of page.children) {
			if (node.type !== 'Text' || node.props.position) continue;
			const rootPosition = (node as TextNode & { position?: { x?: number; y?: number } }).position;
			if (rootPosition) continue;
			const height = Math.max(
				8,
				((node.props.fontSize ?? 14) * 0.352778 * (node.props.lineHeight ?? 1.4))
			);
			node.props.position = {
				x: 18,
				y: contentY,
				width: pageWidthMm - 36,
				height
			};
			node.positionMode = 'absolute';
			contentY += height + 4;
		}
	}
}

function assertDocForgeDocument(value: unknown): asserts value is TemplateDocument {
	if (
		!value ||
		typeof value !== 'object' ||
		Array.isArray(value) ||
		!('meta' in value) ||
		!('pages' in value) ||
		!Array.isArray(value.pages) ||
		value.pages.length === 0
	) {
		throw new Error('The saved DocForge template document is invalid.');
	}
	const pages = value.pages as TemplateNode[];
	if (pages.some((page) => !page || page.type !== 'Page' || !Array.isArray((page as { children?: unknown }).children))) {
		throw new Error('The saved DocForge template pages are invalid.');
	}
}
