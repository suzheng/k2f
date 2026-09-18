# Text

A paragraph is a `content.type: "text"` node. Appearance comes from the node's `role` in `styles/theme.json`, not from fields on the node.

Copy [`ex_heading.json`](../../skills/k2f/catalog/content/ex_heading.json) into `content/root.json` `children` (do not replace `root.json` with the fragment).

## Paragraph vs line break

| You want | Do this |
|----------|---------|
| A paragraph | One text node (`role` usually `body`) |
| The next paragraph | A **sibling** text node. Space between them is the parent `layout.gap` (see [Layout](layout.md)) |
| A line break inside one paragraph | `\n` in `content.value` |
| A heading | A separate node with role `h1`–`h4` |

There is no HTML `<br>` node and no per-node margin. `preserve_whitespace` is for code-like text, not ordinary body copy.

`ex_heading.json` also sets `keep_with_next` on the title so the heading does not sit alone at the bottom of a page.

## Emphasis, links, math

Inline style is `modifiers` on the text node, not `bold` / `color` on the node. Each modifier needs `type`, `intent`, and a UTF-8 **byte** `range` (`\n` is 1 byte; CJK and emoji are multi-byte).

Copy [`ex_modifiers.json`](../../skills/k2f/catalog/content/ex_modifiers.json). Always compute ranges with [`scripts/modifier_range.py`](../../skills/k2f/scripts/modifier_range.py) — do not hand-count.

`intent` must exist under `theme.modifiers.styles[type]` (for example `emphasis` → `strong`). Starter already defines the intents that example uses.

## Lists

A list is sibling nodes with `role: "list_item"`, a shared `list_id`, `depth`, and `marker_type` (`bullet` or `number`). Copy [`ex_list.json`](../../skills/k2f/catalog/content/ex_list.json). Do not fake bullets with `•` in a body string.

## Allowed keys

Node fields: [fields.md](../../skills/k2f/references/writing/fields.md). Exact contract: [`nodes.schema.json`](../../skills/k2f/schema/nodes.schema.json).

## Common mistakes

- Putting `font_size`, `bold`, or `color` on the node — those belong on the **role**
- Using two `\n\n` in one string to mean “new paragraph”
- Hand-counting modifier ranges across `\n` or non-ASCII
