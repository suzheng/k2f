pub fn default_theme() -> String {
    r##"{
            "palette": {
                "black": "#000000",
                "white": "#FFFFFF",
                "gray": "#888888"
            },
            "primitives": {
                "surfaces": {
                    "code_bg": { "type": "solid", "color": "#F6F8FA" },
                    "code_dark": { "type": "solid", "color": "#0B0F14" }
                },
                "corners": { "code": 6000 },
                "borders": {
                    "code": { "width_pt": 1000, "color": "#00000014" },
                    "code_dark": { "width_pt": 1000, "color": "#FFFFFF14" }
                }
            },
            "roles": {
                "default": {
                    "font_family": "default",
                    "font_size": 12000,
                    "line_height_mult": 1200,
                    "color": "black"
                },
                "body": {
                     "font_family": "default",
                     "font_size": 12000,
                     "line_height_mult": 1200,
                     "color": "black"
                },
                "code_block": {
                     "font_family": "default",
                     "font_size": 12000,
                     "line_height_mult": 1200,
                     "color": "black",
                     "box_decoration": {
                          "padding_pt": 6000,
                          "corner_radius": "code",
                          "border": "code",
                          "background": "code_bg"
                     },
                     "variants": {
                          "dark_mode": {
                               "box_decoration": {
                                    "padding_pt": 6000,
                                    "corner_radius": "code",
                                    "border": "code_dark",
                                    "background": "code_dark"
                               },
                               "text_overrides": { "color": "#E6EDF3" }
                          }
                     }
                },
                "document": {
                     "font_family": "default",
                     "font_size": 12000,
                     "line_height_mult": 1200,
                     "color": "black"
                },
                "text": {
                     "font_family": "default",
                     "font_size": 12000,
                     "line_height_mult": 1200,
                     "color": "black"
                },
                "row": {
                     "font_family": "default",
                     "font_size": 12000,
                     "line_height_mult": 1200,
                     "color": "black"
                },
                "cell": {
                     "font_family": "default",
                     "font_size": 12000,
                     "line_height_mult": 1200,
                     "color": "black"
                },
                "box": {
                     "font_family": "default",
                     "font_size": 12000,
                     "line_height_mult": 1200,
                     "color": "black"
                },
                "canvas": {
                     "font_family": "default",
                     "font_size": 12000,
                     "line_height_mult": 1200,
                     "color": "black"
                },
                "table": {
                     "font_family": "default",
                     "font_size": 12000,
                     "line_height_mult": 1200,
                     "color": "black"
                },
                "header": {
                     "font_family": "default",
                     "font_size": 18000,
                     "line_height_mult": 1200,
                     "color": "black"
                }
            },
            "modifiers": {
                 "styles": {
                      "syntax_highlight": {
                           "keyword": { "color": "#0550AE" },
                           "string": { "color": "#0A3069" },
                           "comment": { "color": "#6E7781" },
                           "type": { "color": "#8250DF" }
                      },
                      "emphasis": {
                           "Helvetica": { "font_family": "Helvetica" },
                           "Times New Roman": { "font_family": "Times New Roman" },
                           "Courier": { "font_family": "Courier" },
                           "size_10": { "font_size": 10000 },
                           "size_12": { "font_size": 12000 },
                           "size_14": { "font_size": 14000 },
                           "size_18": { "font_size": 18000 },
                           "size_24": { "font_size": 24000 },
                           "size_36": { "font_size": 36000 },
                           "size_48": { "font_size": 48000 },
                           "size_72": { "font_size": 72000 }
                      }
                 }
            },
            "font_aliases": {
                "Helvetica": "default",
                "Times New Roman": "default",
                "Courier": "default"
            }
        }"##
    .to_string()
}

pub fn math_theme() -> String {
    r##"{
            "palette": {
                "black": "#000000",
                "white": "#FFFFFF",
                "gray": "#888888"
            },
            "roles": {
                "default": {
                    "font_family": "default",
                    "font_size": 12000,
                    "line_height_mult": 1200,
                    "color": "black"
                },
                "document": {
                     "font_family": "default",
                     "font_size": 12000,
                     "line_height_mult": 1200,
                     "color": "black"
                },
                "math": {
                     "font_family": "default",
                     "font_size": 12000,
                     "line_height_mult": 1200,
                     "color": "black",
                     "text_align": "center"
                }
            }
        }"##
    .to_string()
}

pub fn line_height_modifier_theme() -> String {
    // Theme that exposes line-height changes as deterministic geometry differences.
    r##"{
  "palette": { "black": "#000000" },
  "roles": {
    "default": { "font_family": "default", "font_size": 12000, "line_height_mult": 1200, "color": "black" },
    "body": { "font_family": "default", "font_size": 12000, "line_height_mult": 1200, "color": "black" }
  },
  "modifiers": {
    "styles": {
      "emphasis": {
        "tall": { "line_height_mult": 3000 },
        "tight": { "line_height_mult": 800 }
      }
    }
  }
}"##
        .to_string()
}

pub fn modifier_overlap_matrix_theme() -> String {
    // A small, explicit theme that makes precedence visible via deterministic font-size patches.
    r##"{
  "palette": { "black": "#000000" },
  "roles": {
    "default": { "font_family": "default", "font_size": 12000, "line_height_mult": 1200, "color": "black" },
    "body": { "font_family": "default", "font_size": 12000, "line_height_mult": 1200, "color": "black" }
  },
  "modifiers": {
    "precedence": ["emphasis", "underline"],
    "styles": {
      "emphasis": { "x": { "font_size": 10000 } },
      "underline": { "x": { "font_size": 20000 } }
    }
  }
}"##
        .to_string()
}

pub fn modifier_fontsize_theme_default_precedence() -> String {
    // Uses the engine's default modifier type precedence.
    r##"{
  "palette": { "black": "#000000" },
  "roles": {
    "default": { "font_family": "default", "font_size": 12000, "line_height_mult": 1200, "color": "black" },
    "body": { "font_family": "default", "font_size": 12000, "line_height_mult": 1200, "color": "black" },
    "header": { "font_family": "default", "font_size": 18000, "line_height_mult": 1200, "color": "black" }
  },
  "modifiers": {
    "styles": {
      "emphasis": {
        "critical": { "font_size": 30000, "line_height_mult": 1200 },
        "strong": { "font_size": 50000, "line_height_mult": 1100 }
      },
      "underline": {
        "on": { "font_size": 10000, "line_height_mult": 1200 }
      }
    }
  }
}"##
        .to_string()
}

pub fn modifier_fontsize_theme_custom_precedence() -> String {
    // Explicit precedence: underline applies first, emphasis applies later and therefore wins.
    r##"{
  "palette": { "black": "#000000" },
  "roles": {
    "default": { "font_family": "default", "font_size": 12000, "line_height_mult": 1200, "color": "black" },
    "body": { "font_family": "default", "font_size": 12000, "line_height_mult": 1200, "color": "black" },
    "header": { "font_family": "default", "font_size": 18000, "line_height_mult": 1200, "color": "black" }
  },
  "modifiers": {
    "precedence": ["underline", "emphasis"],
    "styles": {
      "emphasis": {
        "critical": { "font_size": 30000, "line_height_mult": 1200 },
        "strong": { "font_size": 50000, "line_height_mult": 1100 }
      },
      "underline": {
        "on": { "font_size": 10000, "line_height_mult": 1200 }
      }
    }
  }
}"##
        .to_string()
}

pub fn modifier_role_override_theme() -> String {
    r##"{
  "palette": { "black": "#000000" },
  "roles": {
    "default": { "font_family": "default", "font_size": 12000, "line_height_mult": 1200, "color": "black" },
    "body": { "font_family": "default", "font_size": 12000, "line_height_mult": 1200, "color": "black" }
  },
  "modifiers": {
    "styles": {
      "emphasis": { "strong": { "bold": true } }
    }
  }
}"##
        .to_string()
}

pub fn role_variant_primitives_theme() -> String {
    r##"{
  "palette": {
    "black": "#000000",
    "white": "#FFFFFF"
  },
    "primitives": {
    "surfaces": {
      "card_solid": { "type": "solid", "color": "white" },
      "glass_surface": { "type": "solid", "color": "#FFFFFFCC" }
    },
    "shadows": {
      "elevation_low": {
        "layers": [
          {
            "offset_x_pt": 0,
            "offset_y_pt": 4000,
            "blur_radius_pt": 12000,
            "spread_radius_pt": 0,
            "color": "#0000001A"
          }
        ]
      },
      "elevation_high": {
        "layers": [
          {
            "offset_x_pt": 0,
            "offset_y_pt": 10000,
            "blur_radius_pt": 24000,
            "spread_radius_pt": 2000,
            "color": "#00000026"
          }
        ]
      }
    },
    "blurs": {
      "heavy": { "radius_pt": 8000 }
    },
    "corners": { "medium": 12000 },
    "borders": {
      "glass": { "width_pt": 1000, "color": "#FFFFFF80" }
    }
  },
  "roles": {
    "default": { "font_family": "default", "font_size": 12000, "line_height_mult": 1200, "color": "black" },
    "body": { "font_family": "default", "font_size": 12000, "line_height_mult": 1200, "color": "black" },
    "document": { "font_family": "default", "font_size": 12000, "line_height_mult": 1200, "color": "black" },
    "card": {
      "font_family": "default",
      "font_size": 12000,
      "line_height_mult": 1200,
      "color": "black",
      "variants": {
        "glass": {
          "box_decoration": {
            "background": "glass_surface",
            "corner_radius": "medium",
            "padding_pt": 12000,
            "border": "glass",
            "shadow": "elevation_low",
            "blur": "heavy"
          },
          "text_overrides": {
            "color": "black"
          }
        }
      }
    }
  }
}"##
        .to_string()
}

pub fn canvas_background_gradient_theme() -> String {
    r##"{
  "palette": {
    "black": "#000000",
    "white": "#FFFFFF"
  },
  "primitives": {
    "surfaces": {
      "global_background": {
        "type": "linear_gradient",
        "value": {
          "type": "linear",
          "angle_degrees": 180,
          "stops": [
            { "pos": 0, "color": "#F7F8FA" },
            { "pos": 1000, "color": "#EEF1F6" }
          ]
        }
      }
    }
  },
  "roles": {
    "default": { "font_family": "default", "font_size": 12000, "line_height_mult": 1200, "color": "black" },
    "body": { "font_family": "default", "font_size": 12000, "line_height_mult": 1200, "color": "black" },
    "document": {
      "font_family": "default",
      "font_size": 12000,
      "line_height_mult": 1200,
      "color": "black",
      "variants": {
        "notes": {
          "box_decoration": { "background": "global_background" }
        }
      }
    }
  }
}"##
        .to_string()
}

pub fn card_variants_theme() -> String {
    r##"{
  "palette": {
    "black": "#000000",
    "white": "#FFFFFF"
  },
  "primitives": {
    "surfaces": {
      "card_solid": { "type": "solid", "color": "#FFFFFF" },
      "warning_surface": { "type": "solid", "color": "#FFF7DB" },
      "glass_surface": { "type": "solid", "color": "#FFFFFFCC" }
    },
    "shadows": {
      "elevation_low": {
        "layers": [
          {
            "offset_x_pt": 0,
            "offset_y_pt": 4000,
            "blur_radius_pt": 12000,
            "spread_radius_pt": 0,
            "color": "#0000001A"
          }
        ]
      },
      "elevation_high": {
        "layers": [
          {
            "offset_x_pt": 0,
            "offset_y_pt": 10000,
            "blur_radius_pt": 24000,
            "spread_radius_pt": 2000,
            "color": "#00000026"
          }
        ]
      }
    },
    "blurs": {
      "heavy": { "radius_pt": 8000 }
    },
    "corners": { "medium": 12000 },
    "borders": {
      "subtle": { "width_pt": 1000, "color": "#00000014" },
      "warning": { "width_pt": 1000, "color": "#00000026" },
      "glass": { "width_pt": 1000, "color": "#FFFFFF80" }
    }
  },
  "roles": {
    "default": { "font_family": "default", "font_size": 12000, "line_height_mult": 1200, "color": "black" },
    "body": { "font_family": "default", "font_size": 12000, "line_height_mult": 1200, "color": "black" },
    "document": { "font_family": "default", "font_size": 12000, "line_height_mult": 1200, "color": "black" },
    "card": {
      "font_family": "default",
      "font_size": 12000,
      "line_height_mult": 1200,
      "color": "black",
      "box_decoration": {
        "background": "card_solid",
        "corner_radius": "medium",
        "padding_pt": 12000,
        "border": "subtle",
        "shadow": "elevation_low"
      },
      "variants": {
        "warning": {
          "box_decoration": {
            "background": "warning_surface",
            "border": "warning",
            "shadow": "elevation_low"
          }
        },
        "glass": {
          "box_decoration": {
            "background": "glass_surface",
            "border": "glass",
            "shadow": "elevation_low",
            "blur": "heavy"
          }
        },
        "elevation_high": {
          "box_decoration": { "shadow": "elevation_high" }
        }
      }
    }
  }
}"##
        .to_string()
}

pub fn semantic_lists_theme() -> String {
    // Minimal theme that defines list marker geometry for role "list_item".
    r##"{
  "palette": { "black": "#000000" },
  "roles": {
    "default": { "font_family": "default", "font_size": 12000, "line_height_mult": 1200, "color": "black" },
    "document": { "font_family": "default", "font_size": 12000, "line_height_mult": 1200, "color": "black" },
    "list_item": {
      "font_family": "default",
      "font_size": 12000,
      "line_height_mult": 1200,
      "color": "black",
      "list_style": {
        "marker_box_width_pt": 18000,
        "marker_gap_pt": 4000,
        "depth_indent_pt": 18000,
        "bullet_glyph": "•",
        "number_suffix": "."
      },
      "variants": {
        "compact": {
          "list_style": {
            "marker_gap_pt": 2000
          }
        }
      }
    }
  }
}"##
        .to_string()
}

pub fn running_footer_theme() -> String {
    let mut v: serde_json::Value =
        serde_json::from_str(&default_theme()).expect("default theme json");
    v["roles"]["footer"] = serde_json::json!({
        "font_family": "default",
        "font_size": 10000,
        "line_height_mult": 1200,
        "color": "black",
        "text_align": "center",
        "self_align": "center"
    });
    serde_json::to_string_pretty(&v).expect("footer theme json")
}
