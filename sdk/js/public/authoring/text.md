# Text

A paragraph is a node with `content.type: "text"`. Headings, body copy, quotes, and code listings are the same kind of node — they differ by `role`. Appearance comes from that role in `styles/theme.json`, not from fields on the node.

Copy catalog fragments into `content/root.json` `children`. Do not replace `root.json` with a fragment.

## Paragraphs and headings

| You want | Write |
|----------|--------|
| A paragraph | One text node (`role` usually `body`) |
| The next paragraph | A **sibling** text node. Space between them is the parent `layout.gap` (see [Layout](layout.md)) |
| A line break inside one paragraph | `\n` in `content.value` |
| A heading | A separate node with role `h1`–`h4` |

Copy [`ex_heading.json`](../../skills/k2f/catalog/content/ex_heading.json). It sets `keep_with_next` on the title so the heading does not sit alone at the bottom of a page.

A quotation is the same shape with `role: "quote"`. A code listing is `role: "code"` with `content.type: "text"` — copy [`ex_code.json`](../../skills/k2f/catalog/content/ex_code.json). Do not author `content.type: "code_block"`.

There is no HTML `<br>` node and no per-node margin. `preserve_whitespace` is for code-like text, not ordinary body copy.

## Emphasis, links, and math

Inline style is `modifiers` on the text node, not `bold` / `color` on the node, and not Markdown `**…**` in the string.

Each modifier needs `type`, `intent`, and a UTF-8 **byte** `range` (`[start, end)` into `content.value`). `\n` is 1 byte; CJK and emoji are multi-byte. Closed `type` set: `emphasis`, `link`, `underline`, `strikethrough`, `subscript`, `superscript`, `math`, `syntax_highlight`.

`intent` is a key under `theme.modifiers.styles[type]` for most types (`strong`, `default`, `keyword`). For `link`, `intent` is the URL (look comes from `link.default`). For `math`, `intent` is the TeX; put U+FFFC in the string and mark that character.

Always compute `range` with [`scripts/modifier_range.py`](../../skills/k2f/scripts/modifier_range.py) — do not hand-count:

```bash
python scripts/modifier_range.py --text "Do not sign." --find "Do not"
# [0, 6]
```

Copy [`ex_modifiers.json`](../../skills/k2f/catalog/content/ex_modifiers.json). Starter already defines the style keys that example uses. `$…$` is Markdown import only — see [Markdown conversion](../../skills/k2f/references/converting-markdown.md).

## Lists

A list is sibling nodes with `role: "list_item"`, a shared `list_id`, optional `depth` (nesting; default 0), and `marker_type` (`bullet` or `number`). Copy [`ex_list.json`](../../skills/k2f/catalog/content/ex_list.json). Marker look lives on the `list_item` role in the [theme](theme.md).

Do not fake bullets with `•` in a body string.

## See also (catalog, not this page)

| Need | File |
|------|------|
| Display equation | [`ex_math.json`](../../skills/k2f/catalog/content/ex_math.json) (`content.type: "math"`; add Noto Sans Math and `font_aliases`) |
| Numbered equation | [`ex_math_numbered.json`](../../skills/k2f/catalog/content/ex_math_numbered.json) (not `\\tag`) |
| Fillable blank or checkbox | [`ex_form.json`](../../skills/k2f/catalog/content/ex_form.json) (`form_field` — never `____` or `□`) |

## Allowed keys

[Allowed keys](../reference/keys.md) and [`nodes.schema.json`](../../skills/k2f/schema/nodes.schema.json).

## Common mistakes

- Putting `font_size`, `bold`, or `color` on the node — those belong on the **role**
- Using `\n\n` in one string to mean “new paragraph”
- Hand-counting modifier ranges across `\n` or non-ASCII
- Writing `$…$` or `^22^` in author JSON — use `math` / `superscript` modifiers
- Drawing a blank with `____` or `□` — copy `ex_form.json`
