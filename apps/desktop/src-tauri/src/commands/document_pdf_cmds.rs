use serde_json::Value;

#[tauri::command]
pub async fn document_preview_pdf(
    document_json: String,
    context_data: Value,
) -> Result<Vec<u8>, String> {
    render_document_pdf(&document_json, &context_data)
}

fn render_document_pdf(document_json: &str, context_data: &Value) -> Result<Vec<u8>, String> {
    use docforge_engine::layout::engine::LayoutEngine;
    use docforge_engine::model::document::TemplateDocument;
    use docforge_engine::render::font_cache::FontCache;
    use docforge_engine::render::pdf_renderer::{PdfRenderConfig, render_to_pdf};

    let document_value: Value =
        serde_json::from_str(document_json).map_err(|error| error.to_string())?;
    let document_value = normalize_pages(document_value)?;
    let document: TemplateDocument =
        serde_json::from_value(document_value).map_err(|error| error.to_string())?;
    let fonts = FontCache::with_system_fonts();
    let layout = LayoutEngine::new(&fonts)
        .layout(&document, context_data)
        .map_err(|error| error.to_string())?;
    let pdf = render_to_pdf(
        &layout,
        &fonts,
        &PdfRenderConfig {
            title: Some(document.meta.name),
            creator: "AqarDesk DocForge".to_string(),
            ..Default::default()
        },
    )
    .map_err(|error| error.to_string())?;

    if !pdf.bytes.starts_with(b"%PDF-") {
        return Err("DocForge returned an invalid PDF document.".to_string());
    }

    Ok(pdf.bytes)
}

fn normalize_pages(mut document: Value) -> Result<Value, String> {
    if document.get("root").is_some_and(|root| !root.is_null()) {
        return Ok(document);
    }

    let pages = document
        .get("pages")
        .and_then(Value::as_array)
        .ok_or_else(|| "DocForge document must contain a pages array.".to_string())?;
    let mut children = Vec::new();

    for (index, page) in pages.iter().enumerate() {
        let page_children = page
            .get("children")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("DocForge page {index} must contain a children array."))?;
        if index > 0 {
            children.push(serde_json::json!({
                "type": "PageBreak",
                "id": format!("aqar-page-break-{index}")
            }));
        }
        children.extend(page_children.iter().cloned());
    }

    let root = serde_json::json!({
        "type": "Page",
        "id": "aqar-document-pages",
        "children": children
    });
    document
        .as_object_mut()
        .ok_or_else(|| "DocForge document must be a JSON object.".to_string())?
        .insert("root".to_string(), root);

    Ok(document)
}

#[cfg(test)]
mod tests {
    use super::{normalize_pages, render_document_pdf};
    use serde_json::json;

    #[test]
    fn normalize_pages_preserves_each_page_with_page_breaks() {
        let normalized = normalize_pages(json!({
            "version": "2.0",
            "pages": [
                { "type": "Page", "children": [{ "type": "Text", "id": "first" }] },
                { "type": "Page", "children": [{ "type": "Text", "id": "second" }] }
            ]
        }))
        .expect("normalize multi-page document");

        let children = normalized["root"]["children"]
            .as_array()
            .expect("read root children");
        assert_eq!(children.len(), 3);
        assert_eq!(children[0]["id"], "first");
        assert_eq!(children[1]["type"], "PageBreak");
        assert_eq!(children[2]["id"], "second");
    }

    #[test]
    fn normalize_pages_keeps_legacy_root_documents() {
        let document = json!({ "root": { "type": "Page", "children": [] } });
        assert_eq!(
            normalize_pages(document.clone()).expect("keep root"),
            document
        );
    }

    #[test]
    fn render_document_pdf_returns_a_pdf_for_a_multipage_docforge_document() {
        let document = json!({
            "version": "2.0",
            "meta": {
                "name": "Receipt",
                "documentType": "receipt",
                "entityType": "receipt",
                "locale": "ar",
                "pageSetup": {
                    "size": "A5",
                    "orientation": "portrait",
                    "margins": { "top": 10, "bottom": 10, "left": 10, "right": 10 },
                    "unit": "mm"
                }
            },
            "pages": [
                { "type": "Page", "id": "page-one", "children": [] },
                { "type": "Page", "id": "page-two", "children": [] }
            ]
        });

        let pdf = render_document_pdf(&document.to_string(), &json!({}))
            .expect("render a multipage DocForge document");

        assert!(pdf.starts_with(b"%PDF-"));
    }
}
