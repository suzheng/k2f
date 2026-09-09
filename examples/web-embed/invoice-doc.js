/** Compile the invoice example through the SDK Editor API. Geometry stays in WASM. */

function textNode(id, role, text, extra = {}) {
  return { id, role, content: { type: "text", value: text }, ...extra };
}

function headingNode(id, level, text) {
  const role = level <= 4 ? `h${level}` : "h4";
  return textNode(id, role, text, { keep_with_next: true });
}

function warningNode(id, text) {
  return textNode(id, "warning", text, { break_inside: "avoid" });
}

function tableNode(tableId, columns, rows) {
  const cols = columns.length;
  const widths = Array.from({ length: cols }, () => ({ fr: 1 }));
  const tableRows = [
    columns.map((label, i) =>
      textNode(`${tableId}.h.c${i}`, "table_header_cell", label),
    ),
  ];
  for (const [ri, row] of rows.entries()) {
    const alt = ri % 2 === 1;
    tableRows.push(
      row.map((value, ci) => {
        const cell = textNode(`${tableId}.r${ri}.c${ci}`, "table_row_cell", value);
        if (alt) cell.variant = "alt";
        return cell;
      }),
    );
  }
  return {
    id: tableId,
    role: "table",
    content: {
      type: "table",
      value: {
        column_widths: widths,
        header_rows: 1,
        gap: 4000,
        data: { type: "inline", rows: tableRows },
      },
    },
  };
}

function childCount(ed, parentId = "root") {
  return ed.getNode(parentId).content.value.children.length;
}

function appendNode(ed, parentId, node) {
  ed.insertNode(parentId, childCount(ed, parentId), node);
}

export function invoiceDocument(k2f, data, shellBytes) {
  if (!shellBytes) {
    throw new Error("invoiceDocument requires shellBytes (a packed .K2F)");
  }
  const headers = data.rows[0].map((cell) => cell.content.value);
  const rows = data.rows.slice(1).map((row) =>
    row.map((cell) => cell.content.value),
  );
  const ed = k2f.Editor.open(shellBytes);
  appendNode(ed, "root", headingNode("invoice.header", 1, "STATEMENT #2025-001"));
  appendNode(
    ed,
    "root",
    textNode("invoice.details", "body", "Date: 15 Dec 2025\nBill To: TechInnovate Inc."),
  );
  appendNode(ed, "root", tableNode("invoice.table", headers, rows));
  appendNode(ed, "root", warningNode("invoice.note", "Net 14."));
  appendNode(ed, "root", textNode("invoice.total", "body", "Grand Total: $7,047.00"));
  return ed.save();
}
