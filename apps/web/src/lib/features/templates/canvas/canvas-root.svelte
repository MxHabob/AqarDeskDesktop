<script lang="ts">
import * as m from "#lib/paraglide/messages";
/**
 * CanvasRoot — The main SVG viewport for the template designer.
 * Renders the document page at real mm dimensions with zoom/pan support.
 * This is the WYSIWYG design surface.
 */
import { onMount, tick } from "svelte";
import type { EditorStore } from "../stores/editor-store.svelte";
import { findNode } from "@docforge/core";
import { mmToPx, pxToMm, screenToDoc } from "@docforge/core/canvas";
import CanvasPage from "./canvas-page.svelte";
import SelectionOverlay from "./selection-overlay.svelte";
import GridOverlay from "./grid-overlay.svelte";
import RulerBar from "./ruler-bar.svelte";
import FloatingActionBar from "../layout/floating-action-bar.svelte";
import { Button } from "@aqardesk/ui/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@aqardesk/ui/components/ui/dropdown-menu";
import { ChevronDown, ChevronRight, Clipboard, Lock, Plus, Trash } from "../icons";

// ── Props ────────────────────────────────────────────────────

let {
  store,
}: {
  store: EditorStore;
} = $props();

// ── Local State ──────────────────────────────────────────────

let containerEl: HTMLDivElement | undefined = $state();
let viewportEl: HTMLDivElement | undefined = $state();
let svgEl: SVGSVGElement | undefined = $state();
let containerWidth = $state(800);
let containerHeight = $state(600);
let scrollTop = $state(0);
let pageLayouts = $state<
  { pageId: string; left: number; top: number; width: number; height: number }[]
>([]);
let previousActivePageIndex = $state(0);
let isPanning = $state(false);
let panStartX = 0;
let panStartY = 0;
let panStartPanX = 0;
let panStartPanY = 0;
let selectionOverlay: SelectionOverlay | undefined = $state();

// ── Derived ──────────────────────────────────────────────────

const vp = $derived(store.viewport);
const pageSetup = $derived(store.document.meta.pageSetup);

// Page dimensions in mm
const pageWidthMm = $derived(getPageDimensionMm("width"));
const pageHeightMm = $derived(getPageDimensionMm("height"));

// Page dimensions in px at current zoom
const pageWidthPx = $derived(mmToPx(pageWidthMm, vp.dpi, vp.zoom));
const pageHeightPx = $derived(mmToPx(pageHeightMm, vp.dpi, vp.zoom));
const pageGapPx = $derived(mmToPx(10, vp.dpi, vp.zoom));
const pageStridePx = $derived(pageHeightPx + pageGapPx);
const pageStackHeightPx = $derived(
  store.pageCount * pageHeightPx + Math.max(0, store.pageCount - 1) * pageGapPx,
);
const renderPages = $derived(
  store.pages
    .map((page, index) => ({ page, index }))
    .sort((a, b) => {
      if (a.index === store.activePageIndex) return 1;
      if (b.index === store.activePageIndex) return -1;
      return a.index - b.index;
    }),
);
const svgHeight = $derived(Math.max(containerHeight, pageStackHeightPx + 80));

// Center the document when it fits; otherwise start at the first page.
const offsetX = $derived(
  Math.max(0, (containerWidth - pageWidthPx) / 2) + vp.panX,
);
const offsetY = $derived(
  Math.max(40, (containerHeight - pageStackHeightPx - 80) / 2) + vp.panY,
);
const activePageOffsetY = $derived(
  offsetY + store.activePageIndex * pageStridePx,
);
const rulerOffsetY = $derived(activePageOffsetY - scrollTop);

$effect(() => {
  const activePageIndex = store.activePageIndex;
  const container = containerEl;
  if (!container || activePageIndex === previousActivePageIndex) return;
  previousActivePageIndex = activePageIndex;
  if (!store.consumeActivePageReveal()) return;

  const pageTop = offsetY + activePageIndex * pageStridePx;
  if (pageTop < scrollTop) {
    container.scrollTo({ top: Math.max(0, pageTop - 24), behavior: "smooth" });
  } else if (pageTop + pageHeightPx > scrollTop + containerHeight) {
    container.scrollTo({
      top: pageTop + pageHeightPx - containerHeight + 24,
      behavior: "smooth",
    });
  }
});

$effect(() => {
  const layoutKey = [
    containerWidth,
    containerHeight,
    pageWidthPx,
    pageHeightPx,
    offsetX,
    offsetY,
    store.activePageIndex,
    store.rulersVisible,
    store.pages.map((page) => page.id).join(","),
  ].join(":");
  if (!layoutKey) return;

  const frame = requestAnimationFrame(updatePageLayouts);
  return () => cancelAnimationFrame(frame);
});

// ── Lifeperiod ────────────────────────────────────────────────

onMount(() => {
  if (containerEl) {
    const obs = new ResizeObserver((entries) => {
      for (const entry of entries) {
        containerWidth = entry.contentRect.width;
        containerHeight = entry.contentRect.height;
      }
    });
    obs.observe(containerEl);

    // Initial zoom to fit
    setTimeout(() => {
      store.zoomToFit(containerWidth, containerHeight);
    }, 50);

    return () => obs.disconnect();
  }
});

// Attach wheel listener imperatively with {passive: false}
// so that preventDefault() works (Svelte's onwheel is passive by default)
$effect(() => {
  const svg = svgEl;
  if (!svg) return;
  svg.addEventListener("wheel", handleWheel, { passive: false });
  return () => svg.removeEventListener("wheel", handleWheel);
});

// ── Mouse Position ───────────────────────────────────────────

let cursorDocMm = $state({ x: 0, y: 0 });
let pointerMoveFrame: number | null = null;
let pendingPointerMove: MouseEvent | null = null;

function handleMouseMove(e: MouseEvent) {
  pendingPointerMove = e;
  if (pointerMoveFrame !== null) return;
  pointerMoveFrame = requestAnimationFrame(() => {
    pointerMoveFrame = null;
    const latestEvent = pendingPointerMove;
    pendingPointerMove = null;
    if (latestEvent) updatePointerPosition(latestEvent);
  });
}

function updatePointerPosition(e: MouseEvent) {
  if (!svgEl) return;
  const rect = svgEl.getBoundingClientRect();
  const screenX = e.clientX - rect.left - offsetX;
  const screenY = e.clientY - rect.top - activePageOffsetY;
  cursorDocMm = screenToDoc(screenX, screenY, { ...vp, panX: 0, panY: 0 });

  if (isPanning) {
    const dx = e.clientX - panStartX;
    const dy = e.clientY - panStartY;
    store.setPan(panStartPanX + dx, panStartPanY + dy);
  }
}

// ── Zoom (Ctrl+Wheel) ────────────────────────────────────────

function handleWheel(e: WheelEvent) {
  if (e.ctrlKey || e.metaKey) {
    e.preventDefault();
    const zoomFactor = e.deltaY > 0 ? 0.95 : 1.05;
    const newZoom = vp.zoom * zoomFactor;
    store.setZoom(newZoom);
  }
}

// ── Pan (Middle mouse or Space+drag) ─────────────────────────

function handleMouseDown(e: MouseEvent) {
  const target = e.target as Element;
  const pageGroup = target.closest<SVGGElement>("[data-page-index]");
  const pageIndex = pageGroup ? Number(pageGroup.dataset.pageIndex) : null;
  let pageChanged = false;

  // Middle mouse button or space+click for pan
  if (e.button === 1 || (e.button === 0 && store.activeTool === "pan")) {
    e.preventDefault();
    isPanning = true;
    panStartX = e.clientX;
    panStartY = e.clientY;
    panStartPanX = vp.panX;
    panStartPanY = vp.panY;
    return;
  }

  if (
    pageIndex !== null &&
    Number.isInteger(pageIndex) &&
    store.activePageIndex !== pageIndex
  ) {
    store.setActivePage(pageIndex, false);
    pageChanged = true;
  }

  // Left click on empty canvas area → rubber band selection or clear
  if (
    e.button === 0 &&
    (e.target === svgEl ||
      target?.classList?.contains("canvas-bg") ||
      target?.classList?.contains("page-bg"))
  ) {
    const startRubberBand = () => {
      if (!selectionOverlay) return;
      if (pageIndex !== null && store.pages[pageIndex]?.locked) {
        store.clearSelection();
        return;
      }
      if (!svgEl) return;
      const rect = svgEl.getBoundingClientRect();
      const canvasX = e.clientX - rect.left - offsetX;
      const activeOffsetY = offsetY + store.activePageIndex * pageStridePx;
      const canvasY = e.clientY - rect.top - activeOffsetY;
      selectionOverlay.startRubberBand(e, canvasX, canvasY);
    };

    if (pageChanged) {
      void tick().then(startRubberBand);
    } else if (selectionOverlay) {
      startRubberBand();
    } else {
      store.clearSelection();
    }
  }
}

function handleMouseUp(e: MouseEvent) {
  if (pointerMoveFrame !== null) cancelAnimationFrame(pointerMoveFrame);
  pointerMoveFrame = null;
  pendingPointerMove = null;
  if (isPanning) updatePointerPosition(e);
  isPanning = false;
}

function moveNodesAcrossPages(
  nodeIds: string[],
  sourcePageIndex: number,
  clientX: number,
  clientY: number,
) {
  if (!svgEl) return;

  const targetPage = [...svgEl.querySelectorAll<SVGGElement>("[data-page-index]")].find(
    (page) => {
      const bounds = page.querySelector(".page-bg")?.getBoundingClientRect();
      return (
        bounds !== undefined &&
        clientX >= bounds.left &&
        clientX <= bounds.right &&
        clientY >= bounds.top &&
        clientY <= bounds.bottom
      );
    },
  );
  const targetPageIndex = Number(targetPage?.dataset.pageIndex);
  if (!targetPage || !Number.isInteger(targetPageIndex) || targetPageIndex === sourcePageIndex) {
    return;
  }
  if (store.pages[targetPageIndex]?.locked) return;

  const sourcePage = svgEl.querySelector<SVGGElement>(
    `[data-page-index="${sourcePageIndex}"]`,
  );
  const sourceBounds = sourcePage?.querySelector(".page-bg")?.getBoundingClientRect();
  const targetBounds = targetPage.querySelector(".page-bg")?.getBoundingClientRect();
  if (!sourceBounds || !targetBounds) return;

  const updates = nodeIds.flatMap((id) => {
    const node = findNode(store.document, id);
    const position =
      (node as any)?.props?.position ?? (node as any)?.position;
    if (!position) return [];

    return [{
      id,
      x: pxToMm(
        mmToPx(Number(position.x) || 0, vp.dpi, vp.zoom) +
          sourceBounds.left -
          targetBounds.left,
        vp.dpi,
        vp.zoom,
      ),
      y: pxToMm(
        mmToPx(Number(position.y) || 0, vp.dpi, vp.zoom) +
          sourceBounds.top -
          targetBounds.top,
        vp.dpi,
        vp.zoom,
      ),
    }];
  });

  store.moveNodesToPage(updates, targetPageIndex);
}

function updatePageLayouts() {
  if (!viewportEl || !svgEl) return;
  const viewportBounds = viewportEl.getBoundingClientRect();

  pageLayouts = store.pages.flatMap((page, index) => {
    const pageBounds = svgEl
      ?.querySelector<SVGRectElement>(`[data-page-index="${index}"] .page-bg`)
      ?.getBoundingClientRect();
    if (!pageBounds) return [];

    return [{
      pageId: page.id,
      left: pageBounds.left - viewportBounds.left,
      top: pageBounds.top - viewportBounds.top,
      width: pageBounds.width,
      height: pageBounds.height,
    }];
  });
}

// ── Helpers ──────────────────────────────────────────────────

function getPageDimensionMm(dim: "width" | "height"): number {
  const sizes: Record<string, { w: number; h: number }> = {
    A4: { w: 210, h: 297 },
    A3: { w: 297, h: 420 },
    A5: { w: 148, h: 210 },
    Letter: { w: 216, h: 279 },
    Legal: { w: 216, h: 356 },
    ID_CARD: { w: 53.98, h: 85.6 },
    BADGE: { w: 54, h: 86 },
  };
  const size = sizes[pageSetup.size] ?? {
    w: pageSetup.width ?? 210,
    h: pageSetup.height ?? 297,
  };
  const isLandscape = pageSetup.orientation === "landscape";
  if (dim === "width") return isLandscape ? size.h : size.w;
  return isLandscape ? size.w : size.h;
}

// ── Drag & Drop ──────────────────────────────────────────────

let isDragOver = $state(false);

function handleDragOver(e: DragEvent) {
  e.preventDefault();
  if (e.dataTransfer) e.dataTransfer.dropEffect = "copy";
  isDragOver = true;
}

function handleDragLeave() {
  isDragOver = false;
}

function handleDrop(e: DragEvent) {
  e.preventDefault();
  isDragOver = false;
  if (!svgEl || !e.dataTransfer) return;

  // Calculate drop position in document mm
  const rect = svgEl.getBoundingClientRect();
  const screenX = e.clientX - rect.left - offsetX;
  const screenY = e.clientY - rect.top - offsetY;
  const pageIndex = Math.floor(screenY / pageStridePx);
  const pageLocalY = screenY - pageIndex * pageStridePx;
  if (
    pageIndex < 0 ||
    pageIndex >= store.pageCount ||
    pageLocalY < 0 ||
    pageLocalY > pageHeightPx
  ) {
    return;
  }
  store.setActivePage(pageIndex, false);
  const docPos = screenToDoc(screenX, pageLocalY, {
    ...vp,
    panX: 0,
    panY: 0,
  });

  // Check for element type drop (from toolbar)
  const elementType = e.dataTransfer.getData("application/x-docforge-element");
  if (elementType) {
    const props: Record<string, unknown> = {};
    if (elementType === "Text") {
      const content = e.dataTransfer.getData("application/x-docforge-text-content");
      if (content) props.content = content;
      const fontSize = Number(
        e.dataTransfer.getData("application/x-docforge-text-font-size"),
      );
      if (Number.isFinite(fontSize) && fontSize > 0) props.fontSize = fontSize;
      const fontWeight = e.dataTransfer.getData(
        "application/x-docforge-text-font-weight",
      );
      if (fontWeight) props.fontWeight = fontWeight;
      const fontFamily = e.dataTransfer.getData(
        "application/x-docforge-text-font-family",
      );
      if (fontFamily) props.fontFamily = fontFamily;
      const color = e.dataTransfer.getData("application/x-docforge-text-color");
      if (color) props.color = color;
      const lineHeight = Number(
        e.dataTransfer.getData("application/x-docforge-text-line-height"),
      );
      if (Number.isFinite(lineHeight) && lineHeight > 0) {
        props.lineHeight = lineHeight;
      }
    } else if (elementType === "Shape") {
      const shape = e.dataTransfer.getData("application/x-docforge-shape");
      if (shape) props.shape = shape;
    }
    store.addElement(elementType, props, docPos.x, docPos.y);
    return;
  }

  // Check for variable drop
  const varName = e.dataTransfer.getData("application/x-docforge-variable");
  if (varName) {
    const selectedNode = store.selectedNode;
    if (selectedNode && selectedNode.type === "Text") {
      const currentContent = (selectedNode as any).props?.content || "";
      store.updateSelectedProps({
        props: {
          ...(selectedNode as any).props,
          content: currentContent + `{{${varName}}}`,
        },
      } as any);
    } else {
      // Create a new text node with the variable
      store.addElement(
        "Text",
        {
          content: `{{${varName}}}`,
          fontSize: 12,
          fontFamily: "Rubik",
        },
        docPos.x,
        docPos.y,
      );
    }
    return;
  }

  // Check for asset drop
  const assetRef = e.dataTransfer.getData("application/x-docforge-asset");
  if (assetRef) {
    const asset = store.document.assets[assetRef];
    const assetUrl = e.dataTransfer.getData("text/uri-list");
    store.addAsset(assetRef, "image");
    store.addElement(
      "Image",
      {
        assetRef,
        src: assetUrl || (asset as any)?.url || "",
        width: 40,
        height: 30,
        fit: "contain",
      },
      docPos.x,
      docPos.y,
    );
    return;
  }
}
</script>

<!-- The application canvas uses pointer events here; keyboard shortcuts are handled by TemplateStudio. -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div bind:this={viewportEl} class="relative grow overflow-hidden bg-muted/45">
  <div
    bind:this={containerEl}
    class="absolute inset-0 overflow-auto scrollbar-none bg-muted/45 select-none"
    role="application"
    aria-label={m.docforge_canvas_label()}
    tabindex="0"
    onmousemove={handleMouseMove}
    onmousedown={handleMouseDown}
    onmouseup={handleMouseUp}
    onmouseleave={handleMouseUp}
    ondragover={handleDragOver}
    ondragleave={handleDragLeave}
    ondrop={handleDrop}
    onscroll={() => {
      scrollTop = containerEl?.scrollTop ?? 0;
      updatePageLayouts();
    }}
  >
  <!-- SVG Canvas -->
  <svg
    bind:this={svgEl}
    width={containerWidth}
    height={svgHeight}
    class="absolute inset-0 {isPanning ? 'cursor-grabbing' : ''}"
    style="top: {store.rulersVisible ? '24px' : '0'}; left: {store.rulersVisible ? '24px' : '0'};"
  >
    <!-- Background pattern -->
    <defs>
      <pattern id="canvas-dots" width="20" height="20" patternUnits="userSpaceOnUse">
        <circle cx="10" cy="10" r="0.5" fill="var(--color-muted-foreground)" opacity="0.15" />
      </pattern>
    </defs>
    <rect width="100%" height="100%" fill="url(#canvas-dots)" class="canvas-bg" />

    <!-- Stack every document page vertically like a multi-page design canvas. -->
    <g transform="translate({offsetX}, {offsetY})">
      {#each renderPages as item (item.page.id)}
        {@const page = item.page}
        {@const index = item.index}
        <g
          transform="translate(0, {index * pageStridePx})"
          class:page-transform={index === store.activePageIndex}
          data-page-index={index}
          data-page-id={page.id}
        >
          <rect
            x="3"
            y="3"
            width={pageWidthPx}
            height={pageHeightPx}
            rx="2"
            fill="black"
            opacity="0.08"
          />
          <rect
            class="page-bg"
            width={pageWidthPx}
            height={pageHeightPx}
            rx="1"
            fill={pageSetup.backgroundColor ?? "white"}
            stroke="var(--color-border)"
            stroke-width="1"
          />

          {#if store.gridVisible && index === store.activePageIndex}
            <GridOverlay
              widthPx={pageWidthPx}
              heightPx={pageHeightPx}
              gridSizeMm={store.gridSize}
              viewport={vp}
            />
          {/if}

          <CanvasPage
            {page}
            {store}
            {index}
            pageLocked={page.locked === true}
            onMoveAcrossPage={moveNodesAcrossPages}
            pageSizePx={{ width: pageWidthPx, height: pageHeightPx }}
          />

          {#if index === store.activePageIndex && !page.locked}
            <SelectionOverlay
              bind:this={selectionOverlay}
              {store}
              pageIndex={index}
              onMoveAcrossPage={moveNodesAcrossPages}
              pageSizePx={{ width: pageWidthPx, height: pageHeightPx }}
            />
          {/if}

          {#if isDragOver && index === store.activePageIndex}
            <rect
              x="0"
              y="0"
              width={pageWidthPx}
              height={pageHeightPx}
              fill="none"
              stroke="#3b82f6"
              stroke-width="3"
              stroke-dasharray="8,4"
              rx="4"
              pointer-events="none"
              opacity="0.5"
            />
          {/if}
        </g>
      {/each}
    </g>
  </svg>
  </div>

  {#each store.pages as page, index (page.id)}
    {@const layout = pageLayouts[index]}
    {#if layout?.pageId === page.id}
      <div
        class="absolute z-10 flex h-8 items-center justify-between border border-border/60 bg-card px-2"
        style="left: {layout.left}px; top: {layout.top - 32}px; width: {layout.width}px;"
      >
      <button
        type="button"
        class="min-w-0 truncate px-2 text-start text-xs font-medium text-muted-foreground hover:text-foreground"
        aria-current={store.activePageIndex === index ? "page" : undefined}
        onclick={() => store.setActivePage(index, false)}
      >
        {page.name || `${m.template_page_label()} ${index + 1}`}
      </button>
      <div class="flex shrink-0 items-center gap-1">
        <Button
          variant="ghost"
          size="icon-xs"
          disabled={index === 0}
          aria-label={m.template_page_move_up()}
          title={m.template_page_move_up()}
          onclick={() => store.movePage(index, -1)}
        >
          <ChevronRight class="-rotate-90" size={14} />
        </Button>
        <Button
          variant="ghost"
          size="icon-xs"
          disabled={index === store.pageCount - 1}
          aria-label={m.template_page_move_down()}
          title={m.template_page_move_down()}
          onclick={() => store.movePage(index, 1)}
        >
          <ChevronRight class="rotate-90" size={14} />
        </Button>
        <Button
          variant="ghost"
          size="icon-xs"
          class={page.locked ? "text-primary" : ""}
          aria-label={page.locked ? m.template_action_unlock() : m.template_action_lock()}
          title={page.locked ? m.template_action_unlock() : m.template_action_lock()}
          onclick={() => {
            store.updateNodeProps(page.id, { locked: !page.locked });
            if (!page.locked && store.activePageIndex === index) {
              store.clearSelection();
            }
          }}
        >
          <Lock size={14} />
        </Button>
        <Button
          variant="ghost"
          size="icon-xs"
          aria-label={m.template_page_duplicate()}
          title={m.template_page_duplicate()}
          onclick={() => store.duplicatePage(index)}
        >
          <Clipboard size={14} />
        </Button>
        <Button
          variant="ghost"
          size="icon-xs"
          class="text-destructive hover:text-destructive"
          disabled={store.pageCount <= 1}
          aria-label={m.template_page_remove()}
          title={m.template_page_remove()}
          onclick={() => store.removePage(index)}
        >
          <Trash size={14} />
        </Button>
      </div>
      </div>
    {/if}
  {/each}

  {#each store.pages as page, index (page.id)}
    {#if index === store.pageCount - 1}
      {@const layout = pageLayouts[index]}
      {#if layout?.pageId === page.id}
        <div
          class="absolute z-10 flex -translate-x-1/2 -translate-y-1/2 items-center"
          style="left: {layout.left + layout.width / 2}px; top: {layout.top + layout.height + pageGapPx}px;"
        >
          <Button
            variant="outline"
            size="lg"
            class="min-w-64 rounded-e-none"
            onclick={() => store.addPage()}
          >
            <Plus size={16} />
            {m.template_page_add()}
          </Button>
          <DropdownMenu>
            <DropdownMenuTrigger>
              {#snippet child({ props }: { props: Record<string, unknown> })}
                <Button
                  {...props}
                  variant="outline"
                  size="icon-lg"
                  class="rounded-s-none"
                  aria-label={m.template_page_duplicate()}
                  title={m.template_page_duplicate()}
                >
                  <ChevronDown size={16} />
                </Button>
              {/snippet}
            </DropdownMenuTrigger>
            <DropdownMenuContent align="center" side="bottom">
              <DropdownMenuItem onclick={() => store.duplicatePage(index)}>
                <Clipboard size={14} />
                {m.template_page_duplicate()}
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      {/if}
    {/if}
  {/each}
  {#if store.rulersVisible}
    <RulerBar
      axis="horizontal"
      pageSizeMm={pageWidthMm}
      viewport={vp}
      {offsetX}
      {offsetY}
      cursorMm={cursorDocMm.x}
      containerSizePx={containerWidth}
    />
    <RulerBar
      axis="vertical"
      pageSizeMm={pageHeightMm}
      viewport={vp}
      {offsetX}
      offsetY={rulerOffsetY}
      cursorMm={cursorDocMm.y}
      containerSizePx={containerHeight}
    />
    <div class="absolute top-0 inset-s-0 z-20 size-6 bg-card border-b border-e border-border/40"></div>
  {/if}

  <FloatingActionBar
    {store}
    {offsetX}
    offsetY={activePageOffsetY - scrollTop}
    pageSizePx={{ width: pageWidthPx, height: pageHeightPx }}
    containerSize={{ width: containerWidth, height: containerHeight }}
    rulersVisible={store.rulersVisible}
  />
</div>
