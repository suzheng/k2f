# K2F Style Operations - Detailed Guide

This guide covers advanced styling operations for K2F theme files.

## Theme Structure

The `theme.json` file contains all visual styling definitions:

```json
{
  "palette": {},
  "primitives": {},
  "roles": {},
  "font_aliases": {},
  "modifiers": {}
}
```

## Palette

Named color strings used throughout the theme:

```json
{
  "palette": {
    "black": "#000000",
    "white": "#FFFFFF",
    "gray_800": "#333333",
    "primary": "#0066CC"
  }
}
```

**Usage:** Reference by name in roles: `"color": "gray_800"`

## Roles

Role → base style mapping. Every role used in content MUST be defined here.

### Basic Role Structure
```json
{
  "roles": {
    "body": {
      "font_family": "default",
      "font_size": 12000,          // 12pt in fixed-point (1/1000 pt)
      "line_height_mult": 1200,    // 1.2x multiplier (1/1000 units)
      "color": "black",             // Palette key or hex
      "text_align": "start",        // "start" | "center" | "end" | "justify"
      "box_decoration": {},        // Optional box styling
      "variants": {}                // Optional variant definitions
    }
  }
}
```

### Required Fields
- `font_family`: String (must match font_aliases or be "default")
- `font_size`: Integer (fixed-point Pt, 1/1000 pt units)
- `line_height_mult`: Integer (multiplier in 1/1000 units, e.g., 1200 = 1.2x)
- `color`: String (palette key or hex color)

### Optional Fields
- `text_align`: "start" | "center" | "end" | "justify" (default: "start"; justify expands U+0020 gaps on non-final wrapped lines)
- `first_line_indent_pt`: Integer (non-negative millipt; first wrapped line only)
- `letter_spacing_pt`: Integer (extra glyph advance in 1/1000 pt; may be negative; official `h1` uses `-500`)
- `bold` / `italic`: Boolean (default `false`)
- `self_align`: "start" | "center" | "end" | "stretch" — overrides parent stack `align_items` for this item
- `list_style`: Optional list layout tokens (see below)
- `box_decoration`: Box decoration object (see Visual Primitives)
- `variants`: Object mapping variant names to style overrides

### Letter spacing, self_align, list_style

```json
{
  "roles": {
    "h1": {
      "font_family": "default",
      "font_size": 28000,
      "line_height_mult": 1200,
      "letter_spacing_pt": -500,
      "color": "ink"
    },
    "button": {
      "font_family": "default",
      "font_size": 12000,
      "line_height_mult": 1200,
      "color": "ink",
      "self_align": "start"
    },
    "list_item": {
      "font_family": "default",
      "font_size": 12000,
      "line_height_mult": 1400,
      "color": "ink",
      "list_style": {
        "marker_box_width_pt": 18000,
        "marker_gap_pt": 6000,
        "depth_indent_pt": 18000,
        "marker_align": "end",
        "bullet_glyph": "•",
        "number_suffix": "."
      }
    }
  }
}
```

`list_style` fields are optional; see `schema/styles.schema.json` `$defs/list_style`. Variants may override `self_align` or `list_style`.

## Variants

Variants switch visual appearance without changing semantic role:

```json
{
  "roles": {
    "card": {
      "font_family": "default",
      "font_size": 12000,
      "line_height_mult": 1200,
      "color": "black",
      "variants": {
        "glass": {
          "box_decoration": {
            "background": "glass_light",
            "blur": "background",
            "border": "subtle",
            "corner_radius": "medium"
          }
        },
        "raised": {
          "box_decoration": {
            "background": "paper",
            "shadow": "elevation.1",
            "corner_radius": "medium"
          }
        }
      }
    }
  }
}
```

**Usage in content:**
```json
{
  "id": "root.card1",
  "role": "card",
  "variant": "glass"  // Applies glass variant styling
}
```

## Visual Primitives

Named reusable visual atoms in `primitives`. Colors live only in `palette` (there is no `primitives.colors`).

### Surfaces (Fills)
```json
{
  "primitives": {
    "surfaces": {
      "glass_light": {
        "type": "solid",
        "color": "#FFFFFFCC"
      },
      "sunrise": {
        "type": "linear_gradient",
        "value": {
          "type": "linear",
          "angle_degrees": 90,
          "stops": [
            { "pos": 0, "color": "#FF6B6B" },
            { "pos": 1000, "color": "#FFE66D" }
          ]
        }
      }
    }
  }
}
```

### Gradients
```json
{
  "primitives": {
    "gradients": {
      "sunrise": {
        "type": "linear",
        "angle_degrees": 90,
        "stops": [
          { "pos": 0, "color": "#FF6B6B" },
          { "pos": 500, "color": "#FFE66D" },
          { "pos": 1000, "color": "#FFFFFF" }
        ]
      }
    }
  }
}
```

**Gradient rules:**
- `angle_degrees`: 0-360 (0 = horizontal right, 90 = vertical down)
- `stops`: Array of `{pos: 0-1000, color: string}`
- `pos`: 0 = start, 1000 = end

### Corners / Borders
```json
{
  "primitives": {
    "corners": { "none": 0, "small": 4000, "medium": 12000, "large": 24000, "full": 9999000 },
    "borders": {
      "subtle": { "width_pt": 500, "color": "#1111111A" },
      "contrast": { "width_pt": 1000, "color": "#1111114D" }
    }
  }
}
```

### Shadows
```json
{
  "primitives": {
    "shadows": {
      "elevation.1": {
        "layers": [
          {
            "offset_x_pt": 0,
            "offset_y_pt": 2000,
            "blur_radius_pt": 4000,
            "spread_radius_pt": 0,
            "color": "#0000001A"
          }
        ]
      },
      "elevation.3": {
        "layers": [
          {
            "offset_x_pt": 0,
            "offset_y_pt": 8000,
            "blur_radius_pt": 24000,
            "spread_radius_pt": 0,
            "color": "#00000033"
          },
          {
            "offset_x_pt": 0,
            "offset_y_pt": 4000,
            "blur_radius_pt": 4000,
            "spread_radius_pt": 0,
            "color": "#00000026"
          }
        ]
      }
    }
  }
}
```

**Shadow rules:**
- `layers`: Array of shadow layers (stacked)
- All values in fixed-point Pt (1/1000 pt units)
- `color`: Hex with alpha (e.g., "#00000020" = 20% opacity)

### Blurs
```json
{
  "primitives": {
    "blurs": {
      "background": {
        "radius_pt": 20000
      }
    }
  }
}
```

## Box Decoration

Theme-side decoration uses **named refs only** (inline fill/shadow/blur/border objects fail schema):

```json
{
  "box_decoration": {
    "background": "glass_light",
    "border": "subtle",
    "corner_radius": "medium",
    "padding_pt": 24000,
    "shadow": "elevation.1",
    "blur": "background"
  }
}
```

`padding_pt` may be a uniform integer or `{ top, right, bottom, left }`. Compile resolves names into an inlined lock `BoxDecoration`.

## Modifiers

Modifier styling configuration:

```json
{
  "modifiers": {
    "precedence": [
      "emphasis",
      "link",
      "underline",
      "strikethrough"
    ],
    "styles": {
      "emphasis": {
        "critical": {
          "bold": true,
          "color": "#CC0000"
        },
        "important": {
          "bold": true
        }
      },
      "syntax_highlight": {
        "warning": {
          "color": "#FFE5E5"
        }
      }
    }
  }
}
```

**Structure:**
- `precedence`: Array of modifier types (low → high priority)
- `styles`: Modifier type → intent → text style patch

**Text style patch fields:**
- `font_family`: String | null
- `font_size`: Integer | null
- `line_height_mult`: Integer | null
- `letter_spacing_pt`: Integer | null
- `color`: String | null
- `text_align`: "start" | "center" | "end" | "justify" | null
- `bold`: Boolean | null
- `italic`: Boolean | null
- `strikethrough`: Boolean | null
- `underline`: Boolean | null

## Font Aliases

Map human-readable font names to loaded font keys:

```json
{
  "font_aliases": {
    "Helvetica": "default",
    "Roboto": "default",
    "Times": "serif_font"
  }
}
```

**Usage:** Reference in roles: `"font_family": "Helvetica"`

## Style Update Operations

### Update Role Base Style
```json
// In theme.json
{
  "roles": {
    "body": {
      "font_size": 14000,        // Changed from 12000 to 14pt
      "color": "gray_800"        // Changed color
    }
  }
}
```

### Add New Role
```json
{
  "roles": {
    "h2": {
      "font_family": "default",
      "font_size": 24000,        // 24pt
      "line_height_mult": 1200,
      "color": "black",
      "bold": true
    }
  }
}
```

### Add Variant to Existing Role
```json
{
  "primitives": {
    "borders": {
      "bordered": { "width_pt": 2000, "color": "#CCCCCC" }
    }
  },
  "roles": {
    "card": {
      "variants": {
        "bordered": {
          "box_decoration": {
            "border": "bordered"
          }
        }
      }
    }
  }
}
```

### Add Primitive
```json
{
  "palette": {
    "accent.primary": "#0066CC"
  },
  "primitives": {
    "surfaces": {
      "surface.elevated": {
        "type": "solid",
        "color": "#FFFFFF"
      }
    },
    "shadows": {
      "elevation.2": {
        "layers": [
          {
            "offset_x_pt": 0,
            "offset_y_pt": 4000,
            "blur_radius_pt": 8000,
            "spread_radius_pt": 0,
            "color": "#00000020"
          }
        ]
      }
    }
  }
}
```

## Best Practices

1. **Use Primitives:** Theme `box_decoration` is named-only — no inline fill/shadow/blur/border objects
2. **Naming:** Prefer official atoms: `glass_light`, `elevation.1`, `corners.medium`, `blurs.background`
3. **Fixed-Point:** All Pt values in 1/1000 pt units (integers)
4. **Color Format:** Use hex with alpha: "#RRGGBBAA" or "#RRGGBB"
5. **Variants:** Use variants for visual skins, not new roles
6. **Validate:** Check against `schema/styles.schema.json` and `schema/visual_primitives.schema.json`

## Common Errors to Avoid

1. **Missing required role fields:** Always include font_family, font_size, line_height_mult, color
2. **Decimal Pt values:** Use integers (12000 not 12.0)
3. **Invalid color format:** Use hex strings or palette keys
4. **Undefined role:** Role must exist before use in content
5. **Undefined variant:** Variant must exist in role.variants
6. **Invalid primitive reference:** Primitive must exist before referencing
