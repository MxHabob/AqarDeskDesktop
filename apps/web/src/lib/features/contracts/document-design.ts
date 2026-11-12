import type { ContractDocumentStyle } from '#lib/api/desktop';

export const DEFAULT_CONTRACT_DOCUMENT_STYLE: ContractDocumentStyle = {
	paper_size: 'a4',
	accent_color: '#C7962F',
	font_family: 'Tahoma',
	font_size: 16,
	alignment: 'start',
	page_margin_mm: 18,
	show_branding: true,
	brand_label: '',
	show_contract_number: true,
	show_footer: true,
	footer_text: ''
};

export function contractDocumentStyleFromPlaceholders(
	value: string | null | undefined,
	defaultPaperSize: ContractDocumentStyle['paper_size'] = DEFAULT_CONTRACT_DOCUMENT_STYLE.paper_size
): ContractDocumentStyle {
	const defaultStyle = { ...DEFAULT_CONTRACT_DOCUMENT_STYLE, paper_size: defaultPaperSize };
	if (!value?.trim()) return defaultStyle;
	const parsed: unknown = JSON.parse(value);
	if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
		throw new Error('Contract template settings must be a JSON object.');
	}
	const design = (parsed as Record<string, unknown>).design;
	if (!design || typeof design !== 'object' || Array.isArray(design)) {
		return defaultStyle;
	}
	const values = design as Record<string, unknown>;
	const paperSize = values.paper_size ?? defaultStyle.paper_size;
	const accentColor = values.accent_color ?? DEFAULT_CONTRACT_DOCUMENT_STYLE.accent_color;
	const fontFamily = values.font_family ?? DEFAULT_CONTRACT_DOCUMENT_STYLE.font_family;
	const fontSize = values.font_size ?? DEFAULT_CONTRACT_DOCUMENT_STYLE.font_size;
	const alignment = values.alignment ?? DEFAULT_CONTRACT_DOCUMENT_STYLE.alignment;
	const pageMargin = values.page_margin_mm ?? DEFAULT_CONTRACT_DOCUMENT_STYLE.page_margin_mm;
	const showBranding = values.show_branding ?? DEFAULT_CONTRACT_DOCUMENT_STYLE.show_branding;
	const brandLabel = values.brand_label ?? DEFAULT_CONTRACT_DOCUMENT_STYLE.brand_label;
	const showContractNumber =
		values.show_contract_number ?? DEFAULT_CONTRACT_DOCUMENT_STYLE.show_contract_number;
	const showFooter = values.show_footer ?? DEFAULT_CONTRACT_DOCUMENT_STYLE.show_footer;
	const footerText = values.footer_text ?? DEFAULT_CONTRACT_DOCUMENT_STYLE.footer_text;
	if (
		!(paperSize === 'a4' || paperSize === 'a5' || paperSize === 'letter') ||
		typeof accentColor !== 'string' ||
		!/^#[\da-f]{6}$/i.test(accentColor) ||
		!(
			fontFamily === 'Tahoma' ||
			fontFamily === 'Arial' ||
			fontFamily === 'Georgia' ||
			fontFamily === 'Noto Naskh Arabic'
		) ||
		!(fontSize === 12 || fontSize === 14 || fontSize === 16 || fontSize === 18) ||
		!(alignment === 'start' || alignment === 'justify') ||
		typeof pageMargin !== 'number' ||
		!Number.isInteger(pageMargin) ||
		pageMargin < 12 ||
		pageMargin > 30 ||
		typeof showBranding !== 'boolean' ||
		typeof brandLabel !== 'string' ||
		brandLabel.length > 100 ||
		typeof showContractNumber !== 'boolean' ||
		typeof showFooter !== 'boolean' ||
		typeof footerText !== 'string' ||
		footerText.length > 400
	) {
		throw new Error('Contract document design settings are invalid.');
	}
	return {
		...defaultStyle,
		paper_size: paperSize,
		accent_color: accentColor,
		font_family: fontFamily,
		font_size: fontSize,
		alignment,
		page_margin_mm: pageMargin,
		show_branding: showBranding,
		brand_label: brandLabel,
		show_contract_number: showContractNumber,
		show_footer: showFooter,
		footer_text: footerText
	};
}

export function contractTemplatePlaceholdersWithDesign(
	value: string | null | undefined,
	design: ContractDocumentStyle
) {
	let parsed: Record<string, unknown> = {};
	if (value?.trim()) {
		const current: unknown = JSON.parse(value);
		if (!current || typeof current !== 'object' || Array.isArray(current)) {
			throw new Error('Contract template settings must be a JSON object.');
		}
		parsed = current as Record<string, unknown>;
	}
	return JSON.stringify({ ...parsed, design });
}

export function contractDocumentFont(fontFamily: ContractDocumentStyle['font_family']) {
	switch (fontFamily) {
		case 'Arial':
			return 'Arial, sans-serif';
		case 'Georgia':
			return 'Georgia, serif';
		case 'Noto Naskh Arabic':
			return '"Noto Naskh Arabic", Tahoma, serif';
		default:
			return 'Tahoma, Arial, sans-serif';
	}
}

export function contractDocumentPaperSize(paperSize: ContractDocumentStyle['paper_size']) {
	switch (paperSize) {
		case 'a5':
			return 'A5';
		case 'letter':
			return 'Letter';
		default:
			return 'A4';
	}
}

export function contractDocumentPaperDimensions(paperSize: ContractDocumentStyle['paper_size']) {
	switch (paperSize) {
		case 'a5':
			return { width: '148mm', height: '210mm' };
		case 'letter':
			return { width: '215.9mm', height: '279.4mm' };
		default:
			return { width: '210mm', height: '297mm' };
	}
}

export function buildPrintDocumentHtml(input: {
	title: string;
	direction: 'rtl' | 'ltr';
	design: ContractDocumentStyle;
	headerHtml: string;
	bodyHtml: string;
	footerHtml: string;
}) {
	return `<!doctype html>
		<html lang="${input.direction === 'rtl' ? 'ar' : 'en'}" dir="${input.direction}">
			<head>
				<meta charset="utf-8">
				<title>${escapeHtml(input.title)}</title>
				<style>
					@page {
						size: ${contractDocumentPaperSize(input.design.paper_size)};
						margin: ${input.design.page_margin_mm}mm;
						@top-left { content: ""; }
						@top-center { content: ""; }
						@top-right { content: ""; }
						@bottom-left { content: ""; }
						@bottom-center { content: ""; }
						@bottom-right { content: ""; }
					}
					* { box-sizing: border-box; }
					html, body { margin: 0; padding: 0; background: #fff; color: #172033; }
					body {
						font-family: ${contractDocumentFont(input.design.font_family)};
						font-size: ${input.design.font_size}px;
						line-height: 1.9;
						text-align: ${input.design.alignment};
						-webkit-print-color-adjust: exact;
						print-color-adjust: exact;
					}
					main { width: auto; }
					header { margin-bottom: 8mm; padding-bottom: 5mm; border-bottom: 2px solid ${input.design.accent_color}; }
					header p { margin: 0; }
					.brand { color: ${input.design.accent_color}; font-weight: 700; }
					.contract-number { color: #64748b; font-size: 0.9em; direction: ltr; text-align: start; }
					h1, h2, h3, h4 { margin: 5mm 0 2mm; color: ${input.design.accent_color}; break-after: avoid-page; }
					h1 { font-size: 1.5em; }
					h2 { font-size: 1.3em; }
					h3, h4 { font-size: 1.15em; }
					p { margin: 0 0 3mm; orphans: 3; widows: 3; }
					.bullet { color: ${input.design.accent_color}; }
					code { padding: 0 1mm; border-radius: 1mm; background: #f1f5f9; font-family: monospace; }
					footer { margin-top: 12mm; padding-top: 3mm; border-top: 1px solid ${input.design.accent_color}; color: #64748b; font-size: 0.85em; }
				</style>
			</head>
			<body>
				<main>${input.headerHtml}${input.bodyHtml}${input.footerHtml}</main>
			</body>
		</html>`;
}

export function printHtmlDocument(html: string): Promise<void> {
	return new Promise((resolve, reject) => {
		const parsed = new DOMParser().parseFromString(html, 'text/html');
		const printRoot = document.createElement('div');
		printRoot.id = 'aqardesk-print-root';
		printRoot.replaceChildren(
			...Array.from(parsed.body.childNodes, (node) => node.cloneNode(true))
		);
		const printStyles = Array.from(parsed.head.querySelectorAll('style')).map((source) => {
			const style = document.createElement('style');
			style.textContent = source.textContent;
			return style;
		});
		const isolationStyles = document.createElement('style');
		isolationStyles.textContent = `
			#aqardesk-print-root { display: none; }
			@media print {
				html, body { display: block !important; width: 100% !important; height: auto !important; margin: 0 !important; padding: 0 !important; overflow: visible !important; }
				body > *:not(#aqardesk-print-root) { display: none !important; }
				#aqardesk-print-root, #aqardesk-print-root * { visibility: visible !important; }
				#aqardesk-print-root { display: block !important; position: static !important; width: 100% !important; height: auto !important; overflow: visible !important; }
			}`;
		const previousTitle = document.title;
		const cleanup = () => {
			window.removeEventListener('afterprint', onAfterPrint);
			printRoot.remove();
			printStyles.forEach((style) => style.remove());
			isolationStyles.remove();
			document.title = previousTitle;
		};
		const onAfterPrint = () => {
			cleanup();
			resolve();
		};
		try {
			document.title = parsed.title || previousTitle;
			document.head.append(...printStyles, isolationStyles);
			document.body.append(printRoot);
			window.addEventListener('afterprint', onAfterPrint, { once: true });
			window.print();
		} catch (cause) {
			cleanup();
			reject(cause);
		}
	});
}

function escapeHtml(value: string) {
	return value
		.replaceAll('&', '&amp;')
		.replaceAll('<', '&lt;')
		.replaceAll('>', '&gt;')
		.replaceAll('"', '&quot;')
		.replaceAll("'", '&#39;');
}
