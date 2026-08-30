# K2F Content Operations - Detailed Guide

This guide covers advanced content reading and writing operations for K2F files.

## Content includes (long documents)

Split large documents by **semantic subtree**, not by rendered page. In `content/root.json`, a container's `children` may mix normal nodes and include stubs:

```json
{ "include": "content/ch01.json" }
```

Each referenced file holds **one** `SemanticNode` (often a container wrapping a chapter). Nested includes are allowed. Rules:

- Path must be under `content/`, end in `.json`, and not be `content/root.json`
- Every file under `content/` must be referenced (directly or transitively) or pack fails with `UNEXPECTED_PATH`
- Compile and `content_hash` use the **expanded** tree; include stubs exist only on disk
- Do not scan or sort all JSON in `content/` — only explicit includes are loaded

`styles/theme.json` stays a single file (optional `styles/tokens.json` only).

## Node Structure Deep Dive

### Required Fields
Every node MUST have:
- `id`: String (hierarchical identifier)
- `role`: String (must exist in theme)
- `content`: Object (one of: `text`, `image`, `container`, `table`, `table_reference`, `code_block`, `math`)

### Optional Fields
- `variant`: String | null (must exist in theme for the role)
- `modifiers`: Array (default: [])
- `layout`: Object | null (`stack`, `grid`, `overlay`, or `columns`)
- `break_inside`: `"auto"` | `"avoid"` (default `"auto"`)
- `keep_with_next`: boolean (default `false`)
- `column_span`: `"none"` | `"all"` (default `"none"`)
- `preserve_whitespace`: boolean | null (for `code_block`, must be `true` or omitted)
- List fields when `role` is `list_item`: `list_id`, `depth`, `marker_type`

## Content Types

### 1. Text Node
```json
{
  "id": "root.text1",
  "role": "body",
  "content": {
    "type": "text",
    "value": "Text content here"
  },
  "modifiers": []
}
```

### 2. Image Node
```json
{
  "id": "root.image1",
  "role": "image",
  "content": {
    "type": "image",
    "value": {
      "src": "assets/images/photo.png",
      "width": 300000,   // 300pt in fixed-point
      "height": 200000  // 200pt in fixed-point
    }
  }
}
```

**Important:** Image sizes MUST be specified in fixed-point Pt (1/1000 pt units). The engine does not inspect image binaries.

### 3. Container Node
```json
{
  "id": "root.section1",
  "role": "section",
  "content": {
    "type": "container",
    "value": {
      "children": [
        // Array of SemanticNode objects
      ]
    }
  },
  "layout": {
    "type": "stack",
    "direction": "vertical",
    "gap": 12000
  }
}
```

### 4. Code Block Node
```json
{
  "id": "root.snippet",
  "role": "code_block",
  "content": {
    "type": "code_block",
    "value": ["fn main() {", "  println!(\"hi\");", "}"]
  },
  "modifiers": [
    { "range": [0, 2], "type": "syntax_highlight", "intent": "keyword" }
  ]
}
```

See [semantic_code_blocks.md](./semantic_code_blocks.md). `role` and `content.type` must both be `code_block`; no `layout`; only `syntax_highlight` modifiers.

### 5. Display Math Node
```json
{
  "id": "root.eq1",
  "role": "math",
  "break_inside": "avoid",
  "content": {
    "type": "math",
    "value": "\\frac{a}{b}"
  }
}
```

Inline math: put U+FFFC in a text node and add `{ "type": "math", "intent": "<tex>", "range": [i, i+1] }`.

### 6. Table Reference Node
```json
{
  "id": "root.table1",
  "role": "table",
  "content": {
    "type": "table_reference",
    "value": {
      "source": "assets/data/revenue.csv",
      "view_mode": "summary",
      "width": 500000,   // 500pt
      "height": 300000  // 300pt
    }
  }
}
```

**Important:** Table sizes MUST be specified. The engine does not load external datasets for `table_reference`.

### 7. Strict Table Node (Native)
```json
{
  "id": "root.table2",
  "role": "table",
  "content": {
    "type": "table",
    "value": {
      "column_widths": [{ "fr": 1 }, { "pt": 200000 }],
      "header_rows": 0,
      "gap": 0,
      "data": {
        "type": "inline",
        "rows": [
          [
            { "id": "root.table2.r0c0", "role": "body", "content": { "type": "text", "value": "A" } },
            { "id": "root.table2.r0c1", "role": "body", "content": { "type": "text", "value": "B" } }
          ]
        ]
      }
    }
  }
}
```

Asset-backed rows: `"data": { "type": "asset", "source": "assets/data/rows.json" }`.

Full attribute details: [attribute_reference.md](./attribute_reference.md).

## Reading Nodes by ID Path

### Simple Path
```
"root" → root node
```

### Lookup by full `id`

Each node's `id` is the **full** dotted path (e.g. `"root.section1.heading"`), not just the last segment.

### Algorithm (depth-first search)

1. Start at the root node (`content/root.json`).
2. If `node.id === target_id`, return the node.
3. If the node is a container, recurse into each child in `content.value.children`.
4. If no match, the id is missing (`UNKNOWN_ID` at SDK/CLI time).

Do not split the path and match segment names — two siblings could share a suffix under different parents.

## Writing Operations

### Adding a New Node
1. Determine parent container
2. Generate unique ID: `parent_id.child_name`
3. Create node with required fields
4. Append to parent's `content.value.children`

**Example:**
```json
// Parent container
{
  "id": "root.section1",
  "content": {
    "type": "container",
    "value": {
      "children": [
        // Add new node here
        {
          "id": "root.section1.new_paragraph",
          "role": "body",
          "content": {
            "type": "text",
            "value": "New paragraph"
          },
          "modifiers": []
        }
      ]
    }
  }
}
```

### Updating Existing Node
1. Find node by ID path
2. Update desired fields
3. Preserve structure (don't change `content.type`)

### Deleting a Node
1. Find parent container
2. Remove node from `content.value.children` array

## Modifiers

Modifiers apply styling to text ranges. Max 50 modifiers per node.

**Structure:**
```json
{
  "modifiers": [
    {
      "range": [0, 5],        // UTF-8 byte range [start, end)
      "type": "emphasis",     // Modifier type
      "intent": "critical"     // Intent (maps to theme)
    }
  ]
}
```

**Rules:**
- `range`: `[start, end)` — start inclusive, end exclusive
- Range is UTF-8 **byte** offsets on character boundaries (not character indices)
- Run `python scripts/modifier_range.py --text "…" --find "…"` instead of counting characters
- Modifiers are sorted by start index
- Overlapping modifiers resolved by theme precedence

**Closed modifier `type` enum:**
- `emphasis` - Bold/italic (intent: `strong`, `emphasis`, …)
- `link` - Underlined accent (intent usually a URL)
- `underline` / `strikethrough`
- `subscript` / `superscript`
- `math` / `syntax_highlight`

There is no `font-size`, `font-family`, or `color` modifier type.

## Layout Hints

### Stack Layout
```json
{
  "layout": {
    "type": "stack",
    "direction": "vertical",      // or "horizontal"
    "gap": 12000,                  // Gap between children (Pt)
    "align_items": "start",        // "start" | "center" | "end" | "stretch"
    "justify_content": "start",    // "start" | "center" | "end"
    "width": 500000,               // Optional fixed width (Pt)
    "height": 300000               // Optional fixed height (Pt)
  }
}
```

### Grid Layout
```json
{
  "layout": {
    "type": "grid",
    "columns": [
      { "pt": 200000 },   // Fixed width column (200pt)
      { "fr": 1 },        // Fractional column
      { "fr": 2 }         // 2x fractional column
    ],
    "rows": [
      { "pt": 100000 },   // Fixed height row (100pt)
      { "fr": 1 }         // Fractional row
    ],
    "gap": 12000,         // Gap between cells (Pt)
    "cell_align": {
      "x": "stretch",     // "start" | "center" | "end" | "stretch"
      "y": "stretch"      // "start" | "center" | "end" | "stretch"
    }
  }
}
```

**Grid rules:**
- Columns/rows: Array of `{pt: integer}`, `{fr: integer}`, or `{auto: true}`
- `auto` tracks take the max measured min-size of cells in that track; `fr` units then distribute remaining space proportionally (`auto` is not CSS `auto-fit`)
- Children placed in document order (no auto-placement)

### Overlay Layout
```json
{
  "layout": {
    "type": "overlay",
    "width": 500000,      // Optional fixed width (Pt)
    "height": 300000      // Optional fixed height (Pt)
  }
}
```

**Overlay rules:**
- Children stacked visually (first = back, last = front)
- No gap or alignment options
- Used for layering (e.g., glass overlay over image)

### Columns Layout
```json
{
  "layout": {
    "type": "columns",
    "count": 2,
    "gap": 12000
  }
}
```

Children pack left→right across equal-width columns (`count` 2..=4). Place title/abstract **outside** the columns container for full-width content. Child nodes may set `"column_span": "all"` to span the full content width (figures/tables). See [attribute_reference.md](./attribute_reference.md).

## Manifest Structure

Package `manifest.json` (no `root`; semantic tree is `content/root.json`):

```json
{
  "title": "Document Title",
  "canvas_mode": "paged",
  "page_config": {
    "width": 595000,
    "height": 842000,
    "margin": [72000, 72000, 72000, 72000]
  },
  "engine_version": "0.1.0",
  "running_blocks": [
    {
      "position": "footer",
      "node": {
        "id": "doc.footer",
        "role": "running_footer",
        "content": { "type": "text", "value": "Confidential" }
      }
    }
  ]
}
```

v0.1 `canvas_mode` is `"paged"` only. At compile time the engine may merge `root` with these fields into an in-memory `Manifest`.

## Best Practices

1. **ID Naming:** Use descriptive, hierarchical IDs: `root.section1.heading1`
2. **Preserve Structure:** Don't change `content.type` when updating
3. **Fixed-Point Math:** All sizes in 1/1000 pt units (integers only)
4. **Validate:** Check against `schema/nodes.schema.json` before writing
5. **Modifier Limits:** Max 50 modifiers per node
6. **Required Fields:** Always include `id`, `role`, `content`

## Common Errors to Avoid

1. **Decimal Pt values:** Use integers (12000 not 12.0)
2. **Missing required fields:** Always include id, role, content
3. **Invalid role:** Role must exist in theme.json
4. **Invalid variant:** Variant must exist in theme for that role
5. **Modifier overflow:** Max 50 modifiers per node
6. **Invalid range:** Modifier ranges must be valid UTF-8 byte offsets on character boundaries
