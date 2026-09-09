# K2F File Operations Guide for AI Agents

> **Agent entry point:** Prefer the repo skill [`skills/k2f/`](../../skills/k2f/SKILL.md) (workflows: writing, converting-markdown, exporting-pdf, embedding-viewer, publishing). Loose JSON editing is fine for authoring; the product format is a packaged `.K2F` ZIP with embedded fonts and lock.

This guide enables AI agents to read, write, and modify K2F document files efficiently. K2F is a semantic document format that separates content (semantic meaning) from presentation (visual styling).

## Quick Reference: `.K2F` ZIP Package

Canonical paths (see [k2f-v0.1.md](../spec/k2f-v0.1.md)):

| Path | Required | Purpose |
|------|----------|---------|
| `manifest.json` | yes | Package metadata (`canvas_mode: "paged"`, `page_config`, optional `running_blocks`) |
| `content/root.json` | yes | State A semantic tree entry |
| `content/**/*.json` | when referenced | Subtree fragments via explicit `{ "include": "content/...." }` stubs |
| `styles/theme.json` | yes | Role-based appearance (single file; do not split theme) |
| `styles/tokens.json` | no | Design tokens |
| `changelog.json` | yes | Edit history |
| `document.K2F.lock` | after compile | State C geometry + render plan |
| `schema/*.json` | yes | Five format schemas only |
| `assets/fonts/*` | yes | Embedded fonts (pack coverage-subsets large CJK faces to GB2312 ∪ Big5 level 1 ∪ JIS X 0208 Han + all non-Han glyphs) |
| `assets/images/*` / `assets/data/*` | no | Images / table data |

**CLI workflow:**

```bash
k2f pack <source_dir> -o doc.K2F
k2f compile doc.K2F
k2f verify doc.K2F
k2f export-pdf doc.K2F -o doc.pdf
```

Agent writing dialect (narrower than the format): `engine/k2f_sdk/profiles/agent_v0.schema.json` — not embedded in the package.

### Loose JSON during authoring

While editing, you may work with unpacked files:

1. **`content/root.json`** — Semantic content tree with hierarchical IDs (may `{ "include": "content/...." }` subtrees)
2. **`content/*.json`** — Optional referenced fragments (one `SemanticNode` per file; must be referenced from root)
3. **`styles/theme.json`** — Visual styling (roles, variants, primitives, palette)
4. **`manifest.json`** — Document metadata and page configuration (no `root` field)

Unreferenced files under `content/` cause `UNEXPECTED_PATH` at pack time. Do not glob or auto-load all JSON in `content/`.

The final deliverable must be a packed `.K2F`.

## Core Concepts (95% of Tasks)

### 1. Reading Content

K2F content is a tree of semantic nodes. Each node has:
- `id`: Hierarchical identifier (e.g., `"root.section1.heading"`)
- `role`: Semantic role (e.g., `"h1"`, `"body"`, `"card"`, `"code_block"`, `"math"`)
- `variant`: Optional visual variant (e.g., `"glass"`, `"warning"`)
- `content`: Node content (`text`, `image`, `container`, `table`, `table_reference`, `code_block`, `math`)
- `modifiers`: Array of text modifiers. Struct: `{"type": "<closed enum>", "intent": "str", "range": [start, end]}`.
- `layout`: Optional layout hint (`stack`, `grid`, `overlay`, `columns`)
- Optional: `break_inside`, `keep_with_next`, `column_span`, list fields (`list_id`, `depth`, `marker_type`)

**Example:**
```json
{
  "id": "root",
  "role": "body",
  "content": {
    "type": "text",
    "value": "Hello World"
  },
  "modifiers": []
}
```

**To read a node by ID path:**
- Navigate the tree using dot-separated IDs: `root.section1.heading`
- For containers, traverse `content.value.children` array

### 2. Writing/Updating Content

**Updating text content:**
```json
{
  "id": "root",
  "role": "body",
  "content": {
    "type": "text",
    "value": "Updated text"
  }
}
```

**Adding a new node:**
- Add to parent's `content.value.children` array
- Assign unique hierarchical ID: `parent_id.child_name`
- Set required fields: `id`, `role`, `content`

**Updating node attributes:**
- Modify any property directly (id, role, variant, layout)
- Preserve structure: keep `content.type` consistent

### 3. Reading Styles

Styles are defined in `theme.json` under:
- `roles`: Role → base style mapping
- `primitives`: Named visual atoms (surfaces, gradients, corners, borders, shadows, blurs)
- `palette`: Sole color table (palette keys or `#RRGGBB` / `#RRGGBBAA`)
- `modifiers`: Modifier type → intent → style patches

**Example:**
```json
{
  "palette": {
    "black": "#000000"
  },
  "roles": {
    "body": {
      "font_family": "default",
      "font_size": 12000,
      "line_height_mult": 1200,
      "color": "black"
    }
  }
}
```

### 4. Updating Styles

**Update role style:**
- Modify properties in `theme.json` → `roles[role_name]`
- Top-level properties: `font_family`, `font_size`, `line_height_mult`, `color`, `text_align`, `letter_spacing_pt`, `self_align`
- Decoration properties (MUST be nested in `box_decoration`): `padding_pt`, `background`, `border`, `shadow`, `blur`, `corner_radius` (all visual atoms are **named string refs**)

**Add new role:**
- Add entry to `roles` object
- Must include: `font_family`, `font_size`, `line_height_mult`, `color`

**Update/add variant:**
- Add to `roles[role_name].variants[variant_name]`
- Can override `box_decoration` or `text_overrides`

**Add primitive:**
- Add to `primitives.surfaces`, `primitives.gradients`, `primitives.corners`, `primitives.borders`, `primitives.shadows`, or `primitives.blurs` (never `primitives.colors` — colors live only in `palette`)

### Font aliases (important)
If using custom font names (e.g. "Roboto") in roles, you must map them to loaded font families (usually "default") in `theme.json`:
```json
{
  "font_aliases": {
    "Roboto": "default",
    "Helvetica": "default"
  }
}
```

### 5. Adding New Attributes

**To add a new attribute to a node:**
1. Check if attribute is supported in schema: `schema/nodes.schema.json`
2. If supported, add directly to node JSON
3. If not supported, you cannot add it (schema enforces structure)

**Common attributes you can add:**
- `variant`: String (must exist in theme for that role)
- `layout`: Object (`stack` / `grid` / `overlay` / `columns`)
- `break_inside`, `keep_with_next`, `column_span`
- Properties within `layout` object (gap, direction, align_items, count, etc.)

**Important:** K2F uses strict schema validation. Only attributes defined in the schema are allowed.

## Fixed-Point Units

All size values use **fixed-point Pt** in 1/1000 pt units:
- `12000` = 12.0 pt
- `595000` = 595.0 pt (A4 width)
- Always use integers, never decimals

## Common Patterns

### Pattern 1: Update Text Content
```json
// Find node by ID path, update content.value
{
  "id": "root.section1.paragraph1",
  "content": {
    "type": "text",
    "value": "New text here"
  }
}
```

### Pattern 2: Add Child Node
```json
// Add to parent's children array
{
  "id": "root.section1.new_heading",
  "role": "h2",
  "content": {
    "type": "text",
    "value": "New Heading"
  },
  "modifiers": []
}
```

### Pattern 3: Change Role/Variant
```json
// Update role or variant to change visual appearance
{
  "id": "root.card1",
  "role": "card",
  "variant": "glass"  // Must exist in theme.roles.card.variants
}
```

### Pattern 4: Update Style
```json
// In theme.json
{
  "roles": {
    "body": {
      "font_size": 14000,  // Changed from 12000 to 14pt
      "color": "gray"      // Changed color
    }
  }
}
```

## Schema Validation

**Always validate against schemas:**
- Content nodes: `schema/nodes.schema.json`
- Theme/styles: `schema/styles.schema.json`
- Visual primitives: `schema/visual_primitives.schema.json`
- Manifest: `schema/manifest.schema.json`

**Key constraints:**
- Max 50 modifiers per node
- All Pt values must be integers (1/1000 pt units)
- Required fields must be present
- `role` must exist in theme
- `variant` must exist in theme for that role

## Detailed Guides

For advanced operations, see:
- [Content Operations Details](./content_operations.md) - Deep dive on reading/writing content nodes
- [Style Operations Details](./style_operations.md) - Advanced styling, primitives, variants
- [Attribute Reference](./attribute_reference.md) - Complete attribute reference

## Schema Files (Read These for Full Details)

- `schema/nodes.schema.json` - Complete node structure and validation rules
- `schema/styles.schema.json` - Complete theme structure and validation rules
- `schema/visual_primitives.schema.json` - Visual primitive definitions
- `schema/manifest.schema.json` - Manifest structure
- `schema/signatures.schema.json` - Signature record
- `engine/k2f_sdk/profiles/agent_v0.schema.json` - Agent authoring dialect (not packed)

## Architecture Reference

For understanding the K2F system architecture:
- [docs/architecture/](../architecture/README.md) — design, codebase map, layout engine
- [layout-engine.md](../architecture/layout-engine.md) — compile pipeline and paint plan

## Elegant Styling Principles

**Use primitives:** Reference named primitives (`"shadow": "elevation.1"`, `"corner_radius": "medium"`) — never inline fill/shadow/blur/border objects in theme `box_decoration`. Define reusable atoms in `theme.json` → `primitives`.

**Use variants, not roles:** Create visual variations via `variants` on existing roles. Only create new roles for semantic differences.

**Use palette:** Reference palette keys (`"color": "gray_800"`) instead of hex strings. Define colors in `palette` for consistency.

**Nest box_decoration:** Decoration properties (padding, background, border, shadow, blur, corner_radius) must be inside `box_decoration`, not top-level in roles.

## Technical Constraints

**Borders:** Define named entries under `primitives.borders`. By default a border applies to all four sides. Optional `edges` (`top` | `right` | `bottom` | `left`) limits which sides receive the stroke; optional `style` is `solid` | `dashed` | `dotted`. Reference by name from `box_decoration.border` — do not inline border objects on roles.

## Elegant Styling Requirements

**Card structure:** Cards can be containers (`"content": {"type": "container", "value": {"children": [...]}}`) with `box_decoration` on the card role, or leaf nodes with text content. For multi-child cards, add `layout.type="stack"` with `gap` to space children.

**Page backgrounds:** Use `document` role variant with `box_decoration.background` referencing a gradient/solid primitive. Apply variant to root: `"role": "document", "variant": "material"`.

**Box decoration essentials:** Prefer named refs: `background`, `corner_radius`, `padding_pt`. Add `shadow` / `blur` only when you intentionally accept raster paint (e.g. `card` + `raised` / `glass`). Example: `"shadow": "elevation.1"`, `"corner_radius": "medium"`.

**Shadows:** Define in `primitives.shadows` with `layers` array (official: `elevation.1`–`3`). Use `blur_radius_pt` 4000-24000 and alpha colors (`#0000001A` to `#00000026`).

**Gradients:** Define in `primitives.surfaces` or `primitives.gradients` as `linear_gradient` with `angle_degrees` and `stops` array (`pos` 0-1000, `color`).

**Glass effects:** Named surface (`glass_light`) + named `blur: "background"` in `box_decoration`.

## Preview Rendering (CLI)

- Pack + compile a directory, then render a page PNG: `k2f pack <dir> -o /tmp/doc.K2F && k2f compile /tmp/doc.K2F && k2f render /tmp/doc.K2F -o /tmp/preview.png`
- `box_decoration` can apply to leaf nodes too (cards don’t have to be containers).

## Tables

- Prefer native `content.type: "table"` with explicit `column_widths` and inline or asset-backed `data`.
- For ad-hoc table-like layouts, use `layout.type="grid"` and put **cell nodes directly** in `children` (row-major order).
- Style headers/cells with roles like `table_header_cell` / `table_row_cell`. Borders create double borders between cells; use spacing/backgrounds instead.

## Layout Notes (Theme)

- `theme.json` roles/variants may set `self_align` (`start|center|end|stretch`) to override a stack parent’s cross-axis `align_items` for that node (e.g., buttons/icons can opt out of `stretch`).
- `text_align` properties in roles must be one of `start`, `center`, `end`, or `justify` (not `left`/`right`).
- Containers may use `layout.type: "columns"` (`count` 2..=4); children may set `column_span: "all"` for full-width figures.

## Gotchas

- **Theme Modifiers**: Must be nested as `modifiers: { "precedence": [], "styles": { "type": { "intent": { patch... } } } }`. Direct mapping fails.
- **Padding**: `padding_pt` may be a uniform integer or `{top, right, bottom, left}` (not an array).
- **Gradients**: `linear_gradient` primitives must include `"type": "linear"`.
- **Roles**: `line_height_mult` is required for all roles.
