import type {
  ExportOptions,
  GridRow,
  AqarDeskColumnDef,
  SelectionSnapshot,
} from "../types";
import { columnId, columnHeader, getCellValue } from "../types";
import { isRowSelected } from "./selection";
import { toTsv } from "./clipboard";

export function exportRows(
  rows: GridRow[],
  columns: AqarDeskColumnDef[],
  options: ExportOptions,
  selection?: SelectionSnapshot,
  visibleIds?: string[],
  rowIdKey = "id",
): string {
  const cols = (
    options.columns
      ? columns.filter((c) => options.columns!.includes(columnId(c)))
      : columns
  ).filter(
    (c) =>
      !c.hidden &&
      !columnId(c).startsWith("__") &&
      c.type !== "action" &&
      columnId(c) !== "actions",
  );

  let subset = rows;
  if (options.scope === "selected" && selection) {
    subset = rows.filter((row) =>
      isRowSelected(selection, String(row[rowIdKey] ?? ""), rows.length),
    );
  } else if (options.scope === "visible" && visibleIds) {
    const set = new Set(visibleIds);
    subset = rows.filter((row) => set.has(String(row[rowIdKey] ?? "")));
  }

  if (options.format === "json") {
    return JSON.stringify(
      subset.map((row) => {
        const obj: GridRow = {};
        for (const col of cols) obj[columnId(col)] = getCellValue(row, col);
        return obj;
      }),
    );
  }

  const header = cols.map(columnHeader);
  const matrix = [
    header,
    ...subset.map((row) => cols.map((col) => getCellValue(row, col))),
  ];
  return options.format === "csv"
    ? matrix
        .map((row) => row.map(toCsvCell).join(","))
        .join("\r\n")
    : toTsv(matrix);
}

function toCsvCell(value: unknown): string {
  let text = value == null ? "" : String(value);
  if (typeof value === "string" && /^\s*[=+\-@]/.test(value)) {
    text = `'${text}`;
  }
  return /[",\r\n]/.test(text) ? `"${text.replaceAll('"', '""')}"` : text;
}

export function downloadText(
  filename: string,
  text: string,
  mime = "text/plain;charset=utf-8",
): void {
  if (typeof document === "undefined") return;
  const blob = new Blob([text], { type: mime });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  a.style.display = "none";
  document.body.append(a);
  a.click();
  a.remove();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}
