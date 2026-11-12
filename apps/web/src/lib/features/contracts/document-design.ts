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
	footer_text: '',
	organization_logo_data_url: null
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
	const organizationLogo =
		values.organization_logo_data_url ?? DEFAULT_CONTRACT_DOCUMENT_STYLE.organization_logo_data_url;
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
		footerText.length > 400 ||
		!(
			organizationLogo === null ||
			(typeof organizationLogo === 'string' &&
				organizationLogo.length <= 1_500_000 &&
				/^data:image\/(?:png|jpeg|webp);base64,[A-Za-z0-9+/]+={0,2}$/.test(organizationLogo))
		)
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
		footer_text: footerText,
		organization_logo_data_url: organizationLogo
	};
}

export function contractTemplatePlaceholdersWithDesign(
	value: string | null | undefined,
	design: ContractDocumentStyle,
	docForgeDocument?: string
) {
	let parsed: Record<string, unknown> = {};
	if (value?.trim()) {
		const current: unknown = JSON.parse(value);
		if (!current || typeof current !== 'object' || Array.isArray(current)) {
			throw new Error('Contract template settings must be a JSON object.');
		}
		parsed = current as Record<string, unknown>;
	}
	return JSON.stringify({
		...parsed,
		design,
		...(docForgeDocument ? { docforge_document: docForgeDocument } : {})
	});
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
