<script lang="ts">
  import { onDestroy, onMount, untrack } from "svelte";
  import type { TemplateDocument } from "@docforge/core";
  import { mmToPx } from "@docforge/core/canvas";
  import { resolveDocForgeDocument } from "#lib/features/contracts/docforge-template";
  import CanvasPage from "./canvas/canvas-page.svelte";
  import { createEditorStore } from "./stores/editor-store.svelte";

  let {
    document,
    values = {},
    id = "docforge-preview",
  }: {
    document: TemplateDocument;
    values?: Record<string, unknown>;
    id?: string;
  } = $props();

  const store = untrack(() =>
    createEditorStore(resolveDocForgeDocument(document, values).document),
  );

  const dimensions = $derived.by(() => {
    const setup = document.meta.pageSetup;
    const customDimension = (value: number | undefined, fallback: number) =>
      typeof value === "number" && Number.isFinite(value) && value >= 20 && value <= 1000
        ? value
        : fallback;
    const presets: Record<string, { width: number; height: number }> = {
      A4: { width: 210, height: 297 },
      A3: { width: 297, height: 420 },
      A5: { width: 148, height: 210 },
      Letter: { width: 215.9, height: 279.4 },
      Legal: { width: 215.9, height: 355.6 },
      ID_CARD: { width: 53.98, height: 85.6 },
      BADGE: { width: 54, height: 86 },
    };
    const preset = presets[setup.size];
    let width =
      setup.size === "Custom" ? customDimension(setup.width, 210) : preset?.width ?? 210;
    let height =
      setup.size === "Custom" ? customDimension(setup.height, 297) : preset?.height ?? 297;
    if (setup.orientation === "landscape") [width, height] = [height, width];
    return { width, height };
  });
  const pageSizePx = $derived({
    width: mmToPx(dimensions.width, store.viewport.dpi, 1),
    height: mmToPx(dimensions.height, store.viewport.dpi, 1),
  });

  onMount(() => {
    const printStyle = window.document.createElement("style");
    printStyle.textContent = `@media print { @page { size: ${dimensions.width}mm ${dimensions.height}mm; margin: 0; } }`;
    window.document.head.append(printStyle);
    return () => printStyle.remove();
  });

  onDestroy(() => store.destroy());
</script>

<div id={id} class="docforge-preview">
  {#each store.pages as page (page.id)}
    <section
      class="docforge-preview-page"
      style={`width: ${dimensions.width}mm; height: ${dimensions.height}mm;`}
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        viewBox={`0 0 ${pageSizePx.width} ${pageSizePx.height}`}
        role="img"
        aria-label={document.meta.name}
      >
        <rect width={pageSizePx.width} height={pageSizePx.height} fill="white" />
        <CanvasPage {page} {store} {pageSizePx} />
      </svg>
    </section>
  {/each}
</div>

<style>
  .docforge-preview {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1.5rem;
    min-width: 0;
    padding: 1.5rem;
  }

  .docforge-preview-page {
    flex: none;
    overflow: hidden;
    background: white;
    box-shadow: 0 8px 24px rgb(15 23 42 / 16%);
  }

  svg {
    display: block;
    width: 100%;
    height: 100%;
    pointer-events: none;
  }

  @media print {
    .docforge-preview {
      display: block;
      padding: 0;
    }

    .docforge-preview-page {
      margin: 0;
      break-after: page;
      box-shadow: none;
    }

    .docforge-preview-page:last-child {
      break-after: auto;
    }
  }
</style>
