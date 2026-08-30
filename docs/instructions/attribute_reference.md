# K2F attribute reference (v0.1)

Field reference for authoring K2F packages. **Normative contract:** `schema/*.json` and [k2f-v0.1.md](../spec/k2f-v0.1.md). This file is a convenience index — when in doubt, trust the schema.

## Contents

- [Node attributes](#node-attributes)
- [Content types](#content-types)
- [Layout attributes](#layout-attributes)
- [Modifier attributes](#modifier-attributes)
- [Theme attributes](#theme-attributes)
- [Primitive attributes](#primitive-attributes)
- [Manifest attributes](#manifest-attributes)
- [Schema validation](#schema-validation)

## Node Attributes

### Required Attributes

**`id`** (string)
- Hierarchical identifier
- Format: `parent_id.child_name`
- Example: `"root.section1.heading"`

**`role`** (string)
- Semantic role name
- Must exist in `theme.json` → `roles`
- Example: `"body"`, `"h1"`, `"card"`

**`content`** (object)
- Node content (see Content Types below)
- Required structure depends on content type

### Optional Attributes

**`variant`** (string | null)
- Visual variant name
- Must exist in `theme.json` → `roles[role].variants[variant]`
- Example: `"glass"`, `"warning"`
- Default: `null`

**`modifiers`** (array)
- Text modifiers array
- Max items: 50
- Default: `[]`
- `type` is a closed enum: `emphasis`, `link`, `underline`, `strikethrough`, `subscript`, `superscript`, `math`, `syntax_highlight`
- `intent` is a free string (URL, TeX, highlight token)
- Structure: `[{range: [start, end], type: <closed enum>, intent: string}]`

**`layout`** (object | null)
- Layout hint for containers
- Types: `stack`, `grid`, `overlay`, `columns`
- Default: `null`
- See Layout Attributes below

**`break_inside`** (`"auto"` | `"avoid"`)
- Page split policy
- `"auto"` (default): keep whole if it fits a page, else split at child or line boundaries
- `"avoid"`: never split; fail if taller than one page

**`keep_with_next`** (boolean)
- If `true`, this node and the next sibling must start on the same page when they fit together
- Default: `false`

**`preserve_whitespace`** (boolean | null)
- Whitespace preservation hint
- When `role` is `"code_block"`, must be `true` or omitted (engine rejects `false`)

**`column_span`** (`"none"` | `"all"`)
- Inside a `layout.type = "columns"` container: `"all"` spans full content width
- Default: `"none"`

**`list_id`** (string | null)
- List group identifier
- Used when `role` is `"list_item"`
- Required (enforced by engine validation) for `role="list_item"`
- Default: `null`

**`depth`** (integer | null)
- Nesting depth for list items (>= 0)
- Used when `role` is `"list_item"`
- If omitted/null, the engine treats it as `0`
- Default: `null`

**`marker_type`** (`"bullet"` | `"number"` | null)
- Marker style for list items
- Used when `role` is `"list_item"`
- If omitted/null, the engine applies a deterministic default
- Default: `null`

**List item example**
```json
{
  "id": "root.items.1",
  "role": "list_item",
  "list_id": "list_a",
  "depth": 0,
  "marker_type": "bullet",
  "content": { "type": "text", "value": "First item" }
}
```

## Content Types

### Text Content
```json
{
  "type": "text",
  "value": "Text string"
}
```

### Code Block Content
```json
{
  "type": "code_block",
  "value": "fn main() {}\\n"
}
```
Or line array: `"value": ["fn main() {", "}"]`. Engine joins with `\n`.

Constraints:
- `role` must be `"code_block"` (and vice versa)
- No `layout` on the node
- Modifiers only `"syntax_highlight"`
- See [semantic_code_blocks.md](./semantic_code_blocks.md)

### Math Content (display)
```json
{
  "type": "math",
  "value": "\\frac{a}{b}"
}
```
Use with `role: "math"`. Inline math stays on a text node as U+FFFC plus modifier `{ "type": "math", "intent": "<tex>" }`.

### Image Content
```json
{
  "type": "image",
  "value": {
    "src": "string",           // Required: Image path
    "width": integer,          // Required: Width in Pt (1/1000 pt)
    "height": integer          // Required: Height in Pt (1/1000 pt)
  }
}
```

### Container Content
```json
{
  "type": "container",
  "value": {
    "children": [              // Required: Array of SemanticNode
      // ... nodes ...
    ]
  }
}
```

### Table Reference Content
```json
{
  "type": "table_reference",
  "value": {
    "source": "string",        // Required: Data source path
    "view_mode": "string",     // Required: View mode identifier
    "width": integer,          // Required: Width in Pt (1/1000 pt)
    "height": integer          // Required: Height in Pt (1/1000 pt)
  }
}
```

### Strict Table Content (Native)
```json
{
  "type": "table",
  "value": {
    "column_widths": [         // Required: explicit columns (strict, deterministic)
      { "fr": 1 },
      { "pt": 200000 },
      { "fr": 2 }
    ],
    "header_rows": 0,          // Optional, default: 0 (repeat on page breaks in paged mode)
    "gap": 0,                  // Optional, default: 0 (Pt in 1/1000 units, used for both row/col gaps)
    "data": {
      "type": "inline",
      "rows": [
        [ /* SemanticNode */, /* SemanticNode */, /* SemanticNode */ ],
        [ /* SemanticNode */, /* SemanticNode */, /* SemanticNode */ ]
      ]
    }
  }
}
```

**Asset-backed table data** (alternative to inline rows):
```json
"data": {
  "type": "asset",
  "source": "assets/data/rows.json"
}
```

**Constraints (enforced by engine validation):**
- `column_widths` must be non-empty (`{pt}` / `{fr}` only — no `{auto:true}`)
- Every row must have exactly `column_widths.length` cells (after asset expand)
- `header_rows <= rows.length`

## Layout Attributes

### Stack Layout
```json
{
  "layout": {
    "type": "stack",           // Required
    "direction": "vertical" | "horizontal",  // Optional, default: "vertical"
    "gap": integer,            // Optional, default: 0 (Pt in 1/1000 units)
    "align_items": "start" | "center" | "end" | "stretch",  // Optional, default: "stretch"
    "justify_content": "start" | "center" | "end",           // Optional, default: "start"
    "width": integer,          // Optional: Fixed width (Pt in 1/1000 units)
    "height": integer          // Optional: Fixed height (Pt in 1/1000 units)
  }
}
```

### Grid Layout
```json
{
  "layout": {
    "type": "grid",            // Required
    "columns": [               // Required: Array of track definitions
      { "pt": integer } | { "fr": integer } | { "auto": true }
    ],
    "rows": [                  // Required: Array of track definitions
      { "pt": integer } | { "fr": integer } | { "auto": true }
    ],
    "gap": integer,            // Optional, default: 0 (Pt in 1/1000 units)
    "row_gap": integer,        // Optional: row axis gap (millipt); null/omit → use gap
    "column_gap": integer,     // Optional: column axis gap (millipt); null/omit → use gap
    "cell_align": {            // Optional, default: {x: "stretch", y: "stretch"}
      "x": "start" | "center" | "end" | "stretch",
      "y": "start" | "center" | "end" | "stretch"
    }
  }
}
```

**Grid Track:**
- `{pt: integer}` - Fixed size in Pt (1/1000 pt units)
- `{fr: integer}` - Fractional unit (distributes remaining space after `pt` and `auto`)
- `{auto: true}` - Content-sized: max measured min-size of cells in that track, then leftover goes to `fr`. Not CSS `auto-fit`. Layout grids only — table `column_widths` stay `{pt}` / `{fr}`.

### Overlay Layout
```json
{
  "layout": {
    "type": "overlay",         // Required
    "width": integer,          // Optional: Fixed width (Pt in 1/1000 units)
    "height": integer           // Optional: Fixed height (Pt in 1/1000 units)
  }
}
```

### Columns Layout (continuous multi-column flow)
```json
{
  "layout": {
    "type": "columns",         // Required
    "count": 2,                // Required: 2..=4 equal-width columns
    "gap": 12000               // Optional: gap between columns (Pt in 1/1000 units)
  }
}
```

Children pack left→right across columns, then to the next page. Place title/abstract **outside** the columns container for full-width (通栏) content.

**`column_span`** (node attribute, not inside layout):
- `"all"` — node spans the full content width inside a columns container (figures/tables)
- Omit or `"none"` — flows in a single column

```json
{
  "id": "paper.fig",
  "role": "body",
  "column_span": "all",
  "content": { "type": "image", "value": { "src": "assets/images/fig.png", "width": 400000, "height": 200000 } }
}
```

## Modifier Attributes

```json
{
  "modifiers": [
    {
      "range": [integer, integer],  // Required: [start, end) UTF-8 byte offsets
      "type": "emphasis",            // Required: closed enum (see modifiers above)
      "intent": "strong"             // Required: free string (URL / TeX / token)
    }
  ]
}
```

**Constraints:**
- Max 50 modifiers per node
- Range: `[start, end)` — start inclusive, end exclusive
- Range is UTF-8 **byte** offsets on character boundaries (not character indices)
- Use `python scripts/modifier_range.py --text "…" --find "…"` to compute ranges
- Modifiers sorted by start index

## Theme Attributes

### Role Attributes
```json
{
  "roles": {
    "role_name": {
      "font_family": "string",        // Required
      "font_size": integer,            // Required: Pt in 1/1000 units
      "line_height_mult": integer,     // Required: Multiplier in 1/1000 units
      "letter_spacing_pt": integer,    // Optional: extra glyph advance (may be negative)
      "first_line_indent_pt": integer, // Optional: first wrapped line only (non-negative millipt)
      "color": "string",                // Required: Palette key or hex
      "text_align": "start" | "center" | "end" | "justify",  // Optional, default: "start"
      "bold": boolean,                  // Optional, default: false
      "italic": boolean,                // Optional, default: false
      "self_align": "start" | "center" | "end" | "stretch",  // Optional: override parent stack align_items
      "list_style": object,             // Optional: marker box / indent tokens
      "box_decoration": object,        // Optional: named primitive refs + padding
      "variants": object               // Optional: Variant definitions
    }
  }
}
```

**`list_style` fields** (all optional): `marker_box_width_pt`, `marker_gap_pt`, `depth_indent_pt`, `marker_align` (`start`|`center`|`end`), `bullet_glyph`, `number_suffix`.

### Variant Attributes
```json
{
  "variants": {
    "variant_name": {
      "box_decoration": object,        // Optional: Box decoration override
      "self_align": "start" | "center" | "end" | "stretch",
      "list_style": object,            // Optional: list token overrides
      "text_overrides": object         // Optional: Text style patch
    }
  }
}
```

### Box Decoration Attributes

Theme JSON uses **named refs only**. Compile inlines them into lock `BoxDecoration`. Unknown names fail (`UNKNOWN_PRIMITIVE`).

```json
{
  "box_decoration": {
    "background": "paper",             // Optional: surfaces/gradients name
    "border": "subtle",                // Optional: primitives.borders name
    "corner_radius": "small",          // Optional: primitives.corners name
    "padding_pt": integer | object,    // Optional: Padding (uniform or edge-specific)
    "shadow": "elevation.1",           // Optional: primitives.shadows name
    "blur": "background"               // Optional: primitives.blurs name
  }
}
```

**Border primitive value** (under `primitives.borders`, not inline on the role):
```json
{
  "subtle": {
    "width_pt": integer,               // Required: Width (Pt in 1/1000 units)
    "color": "string",                 // Required: Palette key or hex
    "edges": ["top", "bottom"],        // Optional: which sides get the stroke; omit = all four
    "style": "solid"                   // Optional: solid | dashed | dotted
  }
}
```

**Padding:**
```json
// Uniform padding
"padding_pt": 24000

// Edge-specific padding
"padding_pt": {
  "top": integer,
  "right": integer,
  "bottom": integer,
  "left": integer
}
```

### Text Style Patch Attributes
```json
{
  "text_overrides": {
    "font_family": "string" | null,
    "font_size": integer | null,
    "line_height_mult": integer | null,
    "letter_spacing_pt": integer | null,
    "color": "string" | null,
    "text_align": "start" | "center" | "end" | "justify" | null,
    "bold": boolean | null,
    "italic": boolean | null,
    "strikethrough": boolean | null,
    "underline": boolean | null
  }
}
```

## Primitive Attributes

`palette` is the only color table. There is no `primitives.colors`.

### Palette
```json
{
  "palette": {
    "ink": "#111111",
    "paper": "#FFFFFF"
  }
}
```

### Surface (Fill) Primitive
```json
{
  "primitives": {
    "surfaces": {
      "surface_name": {
        "type": "solid",
        "color": "string"
      } | {
        "type": "linear_gradient",
        "value": {
          "type": "linear",
          "angle_degrees": integer,     // 0-360
          "stops": [
            {
              "pos": integer,           // 0-1000
              "color": "string"
            }
          ]
        }
      }
    }
  }
}
```

### Gradient Primitive
```json
{
  "primitives": {
    "gradients": {
      "gradient_name": {
        "type": "linear",
        "angle_degrees": integer,       // Required: 0-360
        "stops": [                      // Required: Array, min 2 items
          {
            "pos": integer,             // Required: 0-1000
            "color": "string"           // Required: Color
          }
        ]
      }
    }
  }
}
```

### Shadow Primitive
```json
{
  "primitives": {
    "shadows": {
      "shadow_name": {
        "layers": [                     // Required: Array, min 1 item
          {
            "offset_x_pt": integer,      // Required: X offset (Pt)
            "offset_y_pt": integer,      // Required: Y offset (Pt)
            "blur_radius_pt": integer,   // Required: Blur radius (Pt, >= 0)
            "spread_radius_pt": integer, // Required: Spread radius (Pt)
            "color": "string"           // Required: Color with alpha
          }
        ]
      }
    }
  }
}
```

### Blur Primitive
```json
{
  "primitives": {
    "blurs": {
      "blur_name": {
        "radius_pt": integer            // Required: Blur radius (Pt, >= 0)
      }
    }
  }
}
```

### Corner Primitive
```json
{
  "primitives": {
    "corners": {
      "small": 4000
    }
  }
}
```

### Border Primitive
```json
{
  "primitives": {
    "borders": {
      "subtle": { "width_pt": 500, "color": "#0000001A" },
      "underline": { "width_pt": 500, "color": "#111111", "edges": ["bottom"] }
    }
  }
}
```

## Manifest Attributes

Package `manifest.json` (on disk) does **not** contain `root`. The semantic tree lives in `content/root.json`. At compile time the SDK/engine may merge `root` with manifest fields into an in-memory `Manifest`.

```json
{
  "title": "string",                    // Required
  "author": "string" | null,            // Optional
  "created_at": integer | null,         // Optional: UTC unix seconds
  "canvas_mode": "paged",               // Required; v0.1 is paged only
  "page_config": {
    "width": integer,                   // Required: Page width (Pt in 1/1000 units)
    "height": integer,                  // Required: Page height (Pt in 1/1000 units)
    "margin": [integer, integer, integer, integer]  // Required: [top, right, bottom, left] (Pt)
  },
  "engine_version": "string",           // Required
  "generated_by": "string" | null,      // Optional: agent/model id
  "running_blocks": [                   // Optional: repeating header/footer nodes
    {
      "position": "header" | "footer",
      "node": { /* SemanticNode */ }
    }
  ]
}
```

`running_blocks` are only valid with `canvas_mode: "paged"`.

## Schema Validation

All attributes must conform to:
- `schema/nodes.schema.json` - Node structure
- `schema/styles.schema.json` - Theme structure
- `schema/visual_primitives.schema.json` - Visual primitives
- `schema/manifest.schema.json` - Manifest structure

**Key Constraints:**
- All Pt values: integers (1/1000 pt units)
- Max modifiers: 50 per node
- Required fields: Must be present
- Role/variant: Must exist in theme
- Range validation: Follows schema min/max rules
