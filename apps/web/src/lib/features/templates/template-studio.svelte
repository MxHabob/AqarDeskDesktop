<script lang="ts">
import type { ContractTemplate, ContractTemplateInput } from "@aqardesk/sdk";
import * as m from "#lib/paraglide/messages";
import {
  docForgeDocumentFromPlaceholders,
  docForgeDocumentToTemplateContent,
  serializeDocForgeTemplate,
} from "#lib/features/contracts/docforge-template";
import {
  contractDocumentStyleFromPlaceholders,
  contractTemplatePlaceholdersWithDesign,
} from "#lib/features/contracts/document-design";
import type { ContractDocumentStyle } from "#lib/api/desktop";
import { createDocument } from "@docforge/core";
import { Button, toast } from "@aqardesk/ui";
import { onMount, untrack } from "svelte";
import CanvasContextMenu from "./canvas/canvas-context-menu.svelte";
import CanvasRoot from "./canvas/canvas-root.svelte";
import DocForgePreview from "./docforge-preview.svelte";
import ContextToolbar from "./layout/context-toolbar.svelte";
import FlyoutPanel from "./layout/flyout-panel.svelte";
import SideRail from "./layout/side-rail.svelte";
import StatusBar from "./layout/status-bar.svelte";
import PropertiesInspector from "./panels/properties-inspector.svelte";
import { copyNodes, cutNodes, pasteNodes } from "./stores/clipboard-manager";
import { createEditorStore } from "./stores/editor-store.svelte";

let {
  template,
  onBack,
  onSave,
}: {
  template: ContractTemplate;
  onBack: () => void;
  onSave: (input: ContractTemplateInput) => Promise<ContractTemplate>;
} = $props();

// State
let templateRecord = $state(untrack(() => template));
let templateName = $state(untrack(() => template.name));
let isSaving = $state(false);
let isPropertiesOpen = $state(false);
let contextMenuVisible = $state(false);
let contextMenuX = $state(0);
let contextMenuY = $state(0);

// Editor Store
let store = createEditorStore({
  version: "2.0",
  meta: {
    name: "",
    documentType: "",
    entityType: "",
    locale: "ar",
    pageSetup: {
      size: "A4",
      orientation: "portrait",
      margins: { top: 10, bottom: 10, left: 10, right: 10 },
      unit: "mm",
    },
  },
  variables: {},
  assets: {},
  pages: [{ type: "Page", id: "page-root", children: [] }],
});

let propertiesWidth = $state(300);
let isLoading = $state(true);
let error = $state<string | null>(null);

const previewAllocationRows = [
  {
    contract_number: "CTR-001",
    property_name: "Sample property",
    unit_number: "A-1",
    amount: "50,000 YER",
    amount_minor: 50000,
    display: "CTR-001 · Sample property (A-1): 50,000 YER",
  },
];

const previewValues: Record<string, unknown> = {
  page: 1,
  items: previewAllocationRows,
  payments: previewAllocationRows,
  payment: {
    number: "RCT-0001",
    receipt_number: "RCT-0001",
    date: "2025-01-15",
    status: "Posted",
    tenant: "Sample Tenant",
    amount: "50,000 YER",
    currency: "YER",
    method: "Cash",
    reference: "—",
    description: "Sample payment",
    notes: "Sample payment",
    allocations: previewAllocationRows,
  },
  tenant: {
    name: "Sample Tenant",
    full_name: "Sample Tenant",
    phone: "+967 700 000 000",
    email: "tenant@example.com",
    address: "Sana'a, Yemen",
    national_id: "",
  },
  contract: {
    number: "CTR-001",
    date: "2025-01-15",
    start_date: "2025-01-01",
    end_date: "2025-12-31",
    rent: "50,000",
    deposit: "10,000",
  },
  property: {
    name: "Sample property",
    code: "PROP-001",
    address: "Al-Zubairi Street",
    city: "Sana'a",
  },
  unit: {
    number: "A-1",
    type: "Apartment",
    floor: "2",
    area: "120 m²",
    bedrooms: "3",
    bathrooms: "2",
    rent: "50,000",
    currency: "YER",
  },
  organization: {
    name: m.app_name(),
    name_ar: m.app_name(),
    phone: "",
    website: "",
    address: "",
    logo_url: "",
  },
};

$effect(() => {
  if (store.selectedIds.length > 0) {
    isPropertiesOpen = true;
    store.activeRailPanel = null;
  }
});

function toggleProperties() {
  const shouldOpen = !isPropertiesOpen;
  isPropertiesOpen = shouldOpen;
  if (shouldOpen) store.activeRailPanel = null;
}

// Initialize
async function initEditor() {
  isLoading = true;
  error = null;
  try {
    const version = templateRecord.versions[0];
    const placeholders = version?.placeholders_json;
    const isReceiptTemplate = templateRecord.lease_type === "receipt";
    const design = contractDocumentStyleFromPlaceholders(
      placeholders,
      isReceiptTemplate ? "a5" : "a4",
      isReceiptTemplate ? 8 : 12,
    );
    const doc = docForgeDocumentFromPlaceholders(
      placeholders,
      version?.content_body ?? "",
      templateRecord.locale,
      isReceiptTemplate ? "a5" : design.paper_size,
    );
    doc.meta.id = templateRecord.id;
    doc.meta.name = templateName;
    doc.meta.documentType = templateRecord.lease_type;
    doc.meta.entityType = templateRecord.lease_type;

    store.setSystemSchemas({
      payment: {
        type: "object",
        label: m.template_schema_payment_group(),
        properties: {
          number: { type: "string", label: m.template_schema_payment_number() },
          receipt_number: { type: "string", label: m.payments_receipt_number() },
          date: { type: "string", label: m.template_schema_payment_date() },
          amount: { type: "number", label: m.template_schema_payment_amount() },
          currency: { type: "string", label: m.template_schema_currency() },
          method: { type: "string", label: m.template_schema_payment_method() },
          reference: { type: "string", label: m.template_schema_payment_reference() },
          description: { type: "string", label: m.template_schema_payment_description() },
          status: { type: "string", label: m.payments_status() },
          tenant: { type: "string", label: m.template_schema_tenant_name() },
          allocations: { type: "array", label: m.payments_allocation_column() },
          notes: { type: "string", label: m.template_schema_payment_description() },
        },
      },
      tenant: {
        type: "object",
        label: m.template_schema_tenant_group(),
        properties: {
          name: { type: "string", label: m.template_schema_tenant_name() },
          phone: { type: "string", label: m.template_schema_tenant_phone() },
          email: { type: "string", label: m.template_schema_tenant_email() },
          national_id: { type: "string", label: m.template_schema_tenant_national_id() },
        },
      },
      contract: {
        type: "object",
        label: m.template_schema_contract_group(),
        properties: {
          number: { type: "string", label: m.template_schema_contract_number() },
          start_date: { type: "string", label: m.template_schema_contract_start_date() },
          end_date: { type: "string", label: m.template_schema_contract_end_date() },
          rent: { type: "number", label: m.template_schema_contract_rent() },
          deposit: { type: "number", label: m.template_schema_contract_deposit() },
        },
      },
      property: {
        type: "object",
        label: m.template_schema_property_group(),
        properties: {
          name: { type: "string", label: m.template_schema_property_name() },
          address: { type: "string", label: m.template_schema_property_address() },
        },
      },
      unit: {
        type: "object",
        label: m.template_schema_unit_group(),
        properties: {
          number: { type: "string", label: m.template_schema_unit_number() },
        },
      },
      organization: {
        type: "object",
        label: m.template_schema_organization_group(),
        properties: {
          name: { type: "string", label: m.template_schema_organization_name_en() },
          name_ar: { type: "string", label: m.template_schema_organization_name_ar() },
          phone: { type: "string", label: m.settings_general_phone() },
          website: { type: "string", label: m.settings_general_website() },
          address: { type: "string", label: m.template_schema_organization_address() },
          logo_url: { type: "image", label: m.template_schema_organization_logo() },
        },
      },
    });

    store.loadDocument(doc);
    isLoading = false;
  } catch (cause: unknown) {
    const message = cause instanceof Error ? cause.message : String(cause);
    toast.error(message);
    error = message;
    isLoading = false;
  }
}

// Rename
async function renameTemplate(newName: string) {
  const trimmedName = newName.trim();
  if (!trimmedName || trimmedName === templateName) return;
  templateName = trimmedName;
  await saveTemplate();
}

// Save
async function saveTemplate() {
  if (isSaving || (!store.isDirty && templateName === templateRecord.name)) return;
  isSaving = true;
  try {
    store.document.meta.name = templateName;
    const isReceiptTemplate = templateRecord.lease_type === "receipt";
    if (isReceiptTemplate && store.document.meta.pageSetup.size !== "A5") {
      store.updatePageSetup({ size: "A5" });
    }
    const setup = store.document.meta.pageSetup;
    const paperSize: ContractDocumentStyle["paper_size"] =
      isReceiptTemplate
        ? "a5"
        : setup.size === "A5"
          ? "a5"
          : setup.size === "Letter"
            ? "letter"
            : "a4";
    const version = templateRecord.versions[0];
    const design = contractDocumentStyleFromPlaceholders(
      version?.placeholders_json,
      isReceiptTemplate ? "a5" : "a4",
      isReceiptTemplate ? 8 : 12,
    );
    const updateRequest: ContractTemplateInput = {
      name: templateName,
      description: templateRecord.description ?? null,
      template_key: templateRecord.template_key,
      lease_type: templateRecord.lease_type,
      locale: templateRecord.locale,
      content_format: version?.content_format ?? "markdown",
      content_body: docForgeDocumentToTemplateContent(store.document),
      placeholders_json: contractTemplatePlaceholdersWithDesign(
        version?.placeholders_json,
        { ...design, paper_size: paperSize },
        serializeDocForgeTemplate(store.document),
      ),
    };

    templateRecord = await onSave(updateRequest);
    templateName = templateRecord.name;
    store.markSaved();
    toast.success(m.ui_copy_template_saved_successfully_299be978());
  } catch (cause: unknown) {
    toast.error(cause instanceof Error ? cause.message : String(cause));
  } finally {
    isSaving = false;
  }
}

function generatePreview() {
  store.clearSelection();
  store.activeRailPanel = null;
  store.editorMode = "preview";
}

// Keyboard Shortcuts
function handleKeydown(e: KeyboardEvent) {
  const isCtrl = e.ctrlKey || e.metaKey;
  const isInputFocused =
    e.target instanceof HTMLInputElement ||
    e.target instanceof HTMLTextAreaElement;

  if (contextMenuVisible) {
    contextMenuVisible = false;
  }

  // Close flyout on Escape
  if (e.code === "Escape" && store.activeRailPanel) {
    e.preventDefault();
    store.activeRailPanel = null;
    return;
  }

  if (isCtrl && e.code === "KeyZ" && !isInputFocused) {
    e.preventDefault();
    if (e.shiftKey) {
      store.redo();
    } else {
      store.undo();
    }
  } else if (isCtrl && e.code === "KeyY" && !isInputFocused) {
    e.preventDefault();
    store.redo();
  } else if (isCtrl && e.code === "KeyS") {
    e.preventDefault();
    saveTemplate();
  } else if (isCtrl && e.code === "KeyD") {
    e.preventDefault();
    store.duplicateSelected();
  } else if (isCtrl && e.code === "KeyA" && !isInputFocused) {
    e.preventDefault();
    store.selectAll();
  } else if (isCtrl && e.code === "KeyC" && !isInputFocused) {
    e.preventDefault();
    copyNodes(store.document, store.selectedIds);
  } else if (isCtrl && e.code === "KeyX" && !isInputFocused) {
    e.preventDefault();
    cutNodes(store.document, store.selectedIds).then((cmd) => {
      if (cmd) {
        store.executeCommand(cmd);
        store.clearSelection();
      }
    });
  } else if (isCtrl && e.code === "KeyV" && !isInputFocused) {
    e.preventDefault();
    const parentId = store.activePage?.id;
    if (parentId) {
      pasteNodes(parentId).then((cmd) => {
        if (cmd) store.executeCommand(cmd);
      });
    }
  } else if (e.code === "Delete" || e.code === "Backspace") {
    if (store.selectedIds.length > 0 && !isInputFocused) {
      e.preventDefault();
      store.deleteSelected();
    }
  } else if (e.code === "Escape") {
    store.clearSelection();
  } else if (
    ["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"].includes(e.code) &&
    !isInputFocused
  ) {
    if (store.selectedIds.length > 0 && store.selectedNode) {
      e.preventDefault();
      const nudge = e.shiftKey ? 5 : 0.5;
      const pos =
        (store.selectedNode as any).props?.position ??
        (store.selectedNode as any).position;
      if (pos) {
        const dx =
          e.code === "ArrowLeft" ? -nudge : e.code === "ArrowRight" ? nudge : 0;
        const dy =
          e.code === "ArrowUp" ? -nudge : e.code === "ArrowDown" ? nudge : 0;
        store.moveNodes([
          {
            id: store.selectedNode.id,
            x: Math.round((pos.x + dx) * 10) / 10,
            y: Math.round((pos.y + dy) * 10) / 10,
          },
        ]);
      }
    }
  }
}

function handleContextMenu(e: MouseEvent) {
  e.preventDefault();
  contextMenuX = e.clientX;
  contextMenuY = e.clientY;
  contextMenuVisible = true;
}

onMount(() => {
  store.setAutoSaveCallback(() => {
    void saveTemplate();
  });
  void initEditor();
  window.addEventListener("keydown", handleKeydown);
  return () => {
    window.removeEventListener("keydown", handleKeydown);
    store.destroy();
  };
});
</script>

<div class="editor-root" role="application" oncontextmenu={handleContextMenu}>
  {#if isLoading}
    <div class="grid h-full w-full place-content-center bg-card text-sm text-muted-foreground">
      {m.common_loading()}
    </div>
  {:else if error}
    <div class="flex h-full w-full flex-col items-center justify-center gap-3 bg-card">
      <p class="max-w-xl text-center text-sm text-destructive" role="alert">{error}</p>
      <Button variant="outline" onclick={onBack}>{m.settings_template_back_to_list()}</Button>
    </div>
  {:else}
    <div class="editor-main">
      {#if store.editorMode === "preview"}
        <section class="preview-mode">
          <div class="preview-toolbar">
            <Button variant="outline" onclick={() => store.editorMode = "design"}>
              {m.settings_template_back_to_editor()}
            </Button>
            <Button onclick={() => window.print()}>{m.template_preview_print()}</Button>
          </div>
          <div class="preview-scroll">
            <DocForgePreview
              document={store.document}
              values={previewValues}
              id="template-print-preview"
            />
          </div>
        </section>
      {:else}
        <SideRail {store} onopen={() => isPropertiesOpen = false} />

        <div class="canvas-area">
          <ContextToolbar
            {store}
            isPropertiesVisible={isPropertiesOpen}
            ontoggleproperties={toggleProperties}
            templateName={templateName}
            onback={onBack}
            onsave={saveTemplate}
            onpreview={generatePreview}
            onrename={renameTemplate}
          />

          <CanvasRoot {store} />

          <FlyoutPanel {store} />
        </div>
        {#if isPropertiesOpen}
          <PropertiesInspector
          {store}
          bind:width={propertiesWidth}
          onclose={() => isPropertiesOpen = false}
        />
        {/if}
      {/if}
    </div>
    {#if store.editorMode === "design"}
      <StatusBar {store} />
    {/if}
  {/if}

  <CanvasContextMenu
    {store}
    x={contextMenuX}
    y={contextMenuY}
    visible={contextMenuVisible}
    onclose={() => contextMenuVisible = false}
  />
</div>

<style>
  .editor-root {
    height: 100%;
    width: 100%;
    display: flex;
    flex-direction: column;
    background: var(--color-muted);
    overflow: hidden;
  }

  .editor-main {
    flex-grow: 1;
    display: flex;
    overflow: hidden;
    position: relative;
    min-width: 0;
    min-height: 0;
    background: var(--color-muted);
  }

  .canvas-area {
    flex: 1 1 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    position: relative;
    min-width: 0;
    background: var(--color-muted);
  }

  .preview-mode {
    display: flex;
    flex: 1 1 0;
    min-width: 0;
    min-height: 0;
    flex-direction: column;
    overflow: hidden;
    background: var(--color-muted);
  }

  .preview-toolbar {
    display: flex;
    flex: none;
    justify-content: space-between;
    gap: 0.75rem;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-card);
    padding: 0.75rem 1rem;
  }

  .preview-scroll {
    flex: 1 1 0;
    min-height: 0;
    overflow: auto;
  }

  @media print {
    .editor-root,
    .editor-main,
    .preview-mode,
    .preview-scroll {
      position: static !important;
      display: block !important;
      height: auto !important;
      min-height: 0 !important;
      overflow: visible !important;
    }

    :global(body *) {
      visibility: hidden !important;
    }

    :global(#template-print-preview),
    :global(#template-print-preview *) {
      visibility: visible !important;
    }

    :global(#template-print-preview) {
      position: absolute !important;
      inset: 0 !important;
      width: 100% !important;
    }

    .preview-toolbar {
      display: none !important;
    }
  }
</style>
