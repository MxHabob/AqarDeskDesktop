<script lang="ts">
/**
 * CanvasPage — Renders a single page's child nodes recursively.
 */
import type { PageNode, TemplateNode } from "@docforge/core";
import type { EditorStore } from "../stores/editor-store.svelte";
import CanvasNode from "./canvas-node.svelte";

let {
  page,
  store,
  pageSizePx,
  pageLocked = false,
  onMoveAcrossPage,
  index = 0,
}: {
  page: PageNode;
  store: EditorStore;
  pageSizePx: { width: number; height: number };
  pageLocked?: boolean;
  onMoveAcrossPage?: (
    nodeIds: string[],
    sourcePageIndex: number,
    clientX: number,
    clientY: number,
  ) => void;
  index?: number;
} = $props();
</script>

<g class="canvas-page" data-page-id={page.id}>
  {#each page.children as child (child.id)}
    <CanvasNode
      node={child}
      {store}
      {pageSizePx}
      {pageLocked}
      {onMoveAcrossPage}
      {index}
    />
  {/each}
</g>
