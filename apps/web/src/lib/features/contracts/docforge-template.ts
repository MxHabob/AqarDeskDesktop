import {
	createDocument,
	createNode,
	deserializeDocument,
	generateNodeId,
	getPages,
	serializeDocument,
	type FooterNode,
	type HeaderNode,
	type ImageNode,
	type PageNode,
	type PageSize,
	type TableNode,
	type TemplateDocument,
	type TemplateNode,
	type TextNode
} from '@docforge/core';
import type { ContractDocument } from '#lib/api/desktop';

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

export function docForgeDocumentFromContract(
	contract: Pick<
		ContractDocument,
		'contract_number'
		| 'template_name'
		| 'template_locale'
		| 'document_body'
		| 'docforge_document_json'
		| 'document_style'
	>,
	footerFallback: string
) {
	if (contract.docforge_document_json?.trim()) {
		const document = deserializeDocument(contract.docforge_document_json);
		assertDocForgeDocument(document);
		document.meta.name = contract.template_name;
		return document;
	}

	const { document_style: design } = contract;
	const paperSize = pageSizes[design.paper_size];
	const direction = contract.template_locale.toLowerCase().startsWith('ar') ? 'rtl' : 'ltr';
	const document = createDocument(
		'contract',
		'contract',
		direction === 'rtl' ? 'ar' : 'en'
	);
	document.meta.name = contract.template_name;
	document.meta.pageSetup.size = paperSize;
	document.meta.pageSetup.orientation = 'portrait';
	document.meta.pageSetup.margins = {
		top: design.page_margin_mm,
		bottom: design.page_margin_mm,
		left: design.page_margin_mm,
		right: design.page_margin_mm
	};

	const dimensions = paperSize === 'A5'
		? { width: 148, height: 210 }
		: paperSize === 'Letter'
			? { width: 215.9, height: 279.4 }
			: { width: 210, height: 297 };
	const contentWidth = dimensions.width - 2 * design.page_margin_mm;
	const fontSize = design.font_size;
	const footerText = design.footer_text.trim() || footerFallback;
	const footerHeight = design.show_footer ? 16 : 0;
	const contentTop =
		design.page_margin_mm +
		(design.organization_logo_data_url ||
		design.show_branding ||
		design.show_contract_number
			? 31
			: 8);
	const contentBottom = dimensions.height - design.page_margin_mm - footerHeight;
	let pageIndex = 0;
	let contentY = contentTop;
	let hasBodyContent = false;

	function addPage() {
		const page: PageNode = {
			type: 'Page',
			id: generateNodeId('page'),
			name: `Page ${pageIndex + 1}`,
			children: []
		};
		pageIndex += 1;
		document.pages.push(page);
		contentY = contentTop;
		hasBodyContent = false;

		if (design.organization_logo_data_url) {
			const logo: ImageNode = {
				type: 'Image',
				id: generateNodeId('organization-logo'),
				name: 'Organization logo',
				props: {
					src: design.organization_logo_data_url,
					position: {
						x: design.page_margin_mm,
						y: design.page_margin_mm,
						width: 28,
						height: 20
					},
					width: 28,
					height: 20,
					fit: 'contain'
				}
			};
			page.children.push(logo);
		}

		if (design.show_branding && design.brand_label.trim()) {
			const brandX = design.page_margin_mm + (design.organization_logo_data_url ? 34 : 0);
			const brand = createTextNode(design.brand_label.trim(), 'Brand', direction, pageIndex, {
				x: brandX,
				y: design.page_margin_mm + 2,
				width: contentWidth - (brandX - design.page_margin_mm),
				height: 10
			});
			brand.props.fontSize = fontSize + 2;
			brand.props.fontFamily = design.font_family;
			brand.props.fontWeight = 'bold';
			brand.props.color = design.accent_color;
			page.children.push(brand);
		}

		if (design.show_contract_number) {
			const number = createTextNode(
				contract.contract_number,
				'Contract number',
				'ltr',
				pageIndex,
				{
					x: design.page_margin_mm,
					y: design.page_margin_mm + 21,
					width: contentWidth,
					height: 8
				}
			);
			number.props.fontSize = 11;
			number.props.fontFamily = design.font_family;
			number.props.color = '#64748b';
			number.props.textAlign = 'end';
			page.children.push(number);
		}

		if (design.show_footer) {
			const footer = createTextNode(
				footerText,
				'Contract footer',
				direction,
				pageIndex,
				{
					x: design.page_margin_mm,
					y: dimensions.height - design.page_margin_mm - footerHeight,
					width: contentWidth,
					height: footerHeight
				}
			);
			footer.props.fontSize = 10;
			footer.props.fontFamily = design.font_family;
			footer.props.color = '#64748b';
			footer.props.textAlign = design.alignment;
			page.children.push(footer);
		}
	}

	addPage();
	for (const rawLine of contract.document_body.split(/\r?\n/)) {
		const line = rawLine.trim();
		if (!line) continue;
		if (line === '[[PAGE_BREAK]]') {
			if (hasBodyContent) addPage();
			continue;
		}

		const heading = /^(#{1,3})\s+(.+)$/.exec(line);
		const listItem = /^(?:[-*+]\s+|\d+[.)]\s+)(.+)$/.exec(line);
		const headingLevel = heading?.[1]?.length ?? 0;
		const name = headingLevel ? 'Heading' : listItem ? 'List item' : 'Paragraph';
		const text = heading?.[2] ?? (listItem ? `• ${listItem[1]}` : line);
		const size = headingLevel === 1 ? fontSize + 4 : headingLevel > 1 ? fontSize + 2 : fontSize;
		const rowHeight = size * 0.352778 * 1.55;
		const charsPerLine = Math.max(16, Math.floor(contentWidth / (size * 0.18)));
		const wrappedLines = wrapText(text, charsPerLine);
		let offset = 0;

		while (offset < wrappedLines.length) {
			if (contentY + rowHeight > contentBottom && hasBodyContent) addPage();
			const availableRows = Math.max(1, Math.floor((contentBottom - contentY) / rowHeight));
			const chunk = wrappedLines.slice(offset, offset + availableRows);
			const blockHeight = chunk.length * rowHeight + 1.5;
			const node = createTextNode(chunk.join('\n'), name, direction, pageIndex, {
				x: design.page_margin_mm,
				y: contentY,
				width: contentWidth,
				height: blockHeight
			});
			node.props.fontSize = size;
			node.props.fontFamily = design.font_family;
			node.props.fontWeight = headingLevel ? 'bold' : 'normal';
			node.props.color = headingLevel ? design.accent_color : '#172033';
			node.props.textAlign = design.alignment;
			node.props.lineHeight = 1.55;
			document.pages[pageIndex - 1]!.children.push(node);
			contentY += blockHeight + (headingLevel ? 3 : 1.5);
			hasBodyContent = true;
			offset += chunk.length;
			if (offset < wrappedLines.length) addPage();
		}
	}
	if (!hasBodyContent && document.pages.length > 1) document.pages.pop();

	return document;
}

export function docForgeContextFromContract(
	contract: Pick<ContractDocument, 'docforge_context_json' | 'document_style'>
): Record<string, unknown> {
	let context: Record<string, unknown> = {};
	if (contract.docforge_context_json?.trim()) {
		const parsed: unknown = JSON.parse(contract.docforge_context_json);
		if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
			throw new Error('The saved contract template data is invalid.');
		}
		context = parsed as Record<string, unknown>;
	}

	const organizationValue = context.organization;
	const organization =
		organizationValue && typeof organizationValue === 'object' && !Array.isArray(organizationValue)
			? (organizationValue as Record<string, unknown>)
			: {};
	return {
		...context,
		organization: {
			...organization,
			logo_url: contract.document_style.organization_logo_data_url ?? ''
		}
	};
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

function wrapText(value: string, maxLength: number) {
	const lines: string[] = [];
	for (const paragraph of value.split(/\r?\n/)) {
		const words = paragraph.split(/\s+/).filter(Boolean);
		let line = '';
		for (const word of words) {
			if (word.length > maxLength) {
				if (line) lines.push(line);
				line = '';
				for (let index = 0; index < word.length; index += maxLength) {
					lines.push(word.slice(index, index + maxLength));
				}
				continue;
			}
			const nextLine = line ? `${line} ${word}` : word;
			if (nextLine.length > maxLength) {
				lines.push(line);
				line = word;
			} else {
				line = nextLine;
			}
		}
		if (line) lines.push(line);
	}
	return lines.length ? lines : [''];
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
