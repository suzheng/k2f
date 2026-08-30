# Semantic code blocks

`code_block` is a format role, not part of the narrow [agent_v0](agent_v0.md) SDK dialect. Use `markdown_to_k2f`, hand-author a theme that defines `code_block`, or unpacked package authoring — see [writing workflow](../../skills/k2f/references/writing.md).

## Node shape

A semantic code block is represented as a leaf `SemanticNode` with:

- **`role`**: `"code_block"`
- **`content.type`**: `"code_block"`
- **`content.value`**: either
  - a single string (may include `\n`), or
  - a flat array of strings (each element is a line)

The engine requires `role` and `content.type` to agree. If a node uses `role: "code_block"` it must use `content.type: "code_block"`, and vice versa.

### Constraints

- **`preserve_whitespace`**: must be `true` or omitted. Explicit `false` is a validation error.
- **`layout`**: not allowed on code block nodes.
- Themes should define a `code_block` role (and optional variants such as `dark_mode`) for styling.

### Whitespace and newline behavior

Code blocks toggle a preformatted text layout path:

- **Leading and trailing whitespace per line is preserved** (no trimming).
- **Line breaks come only from hard newlines** (`\n`). The engine does not insert soft wraps.
- **Empty lines are preserved** (consecutive `\n` yield deterministic vertical spacing).

### `string[]` canonicalization and modifier ranges

When `content.value` is a line array (`string[]`), the engine canonicalizes it to a single text buffer:

- `layout_text = lines.join("\n")`
- No implicit trailing newline is added.

All modifier ranges are validated and applied against this canonical `layout_text` using UTF-8 byte offsets.

### Modifier restriction

Inside a code block, modifiers are intentionally restricted:

- Allowed modifier type: **`"syntax_highlight"`**
- Any other modifier type on a code block is a deterministic validation error.

The engine does not perform language parsing; it only applies the precomputed highlight ranges.
