use k2f_paint::{OpenedDocument, OFFICIAL_PNG_SCALE};
use k2f_sdk::{markdown_to_k2f, MarkdownOptions, PageSize};
use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn generate_showcase_artifacts() {
    let root = repo_root();
    let artifact_dir = root.join("target/showcase-artifacts");
    fs::create_dir_all(&artifact_dir).unwrap();

    let sample_md = r#"# K2F Typography & Visual Style Specification

K2F (Key-to-Flow) is a modern document format engineered for AI Agents and deterministic layouts. With the **Official Typography** design system, K2F provides a refined, publication-grade reading experience with Markdown-scale hierarchy, generous line spacing, and crisp contrast.

## 1. Type Scale & Hierarchy

The body text uses deep slate ink (`#24292f`) rather than faded gray, paired with **1.60x line-height multiplier** for optimal reading rhythm:

- **Heading 1 (H1)**: 24pt Bold, accent brand blue (`#0969da`), 1.25x line height, 10pt bottom margin
- **Heading 2 (H2)**: 18pt Bold, slate ink (`#24292f`), 1.30x line height, 16pt top / 8pt bottom margin
- **Heading 3 (H3)**: 14pt Bold, slate ink (`#24292f`), 1.35x line height, 12pt top / 6pt bottom margin
- **Body Paragraph**: 12pt Regular, slate ink (`#24292f`), 1.60x line height, 10pt bottom margin

### 1.1 Inline Formatting & Modifiers

K2F supports rich inline markup seamlessly: **Strong Bold Text**, *Emphasis Italic Text*, inline code `Editor::open_template("report")`, and interactive hyperlinks such as [K2F Repository Link](https://github.com/suzheng/k2f).

## 2. Structured Layout Elements

### 2.1 Blockquotes & Notes

> "Typography is the foundation of information hierarchy. K2F locks exact geometry coordinates while retaining full semantic fidelity for search, indexing, and AI agent manipulation."

### 2.2 Standard Data Tables (GFM Table)

| Element | Font Size | Line Height | Weight | Default Spacing |
| :--- | :--- | :--- | :--- | :--- |
| **Heading 1 (H1)** | 24 pt | 1.25x | Bold | Bottom: 10 pt |
| **Heading 2 (H2)** | 18 pt | 1.30x | Bold | Top: 16 pt, Bottom: 8 pt |
| **Heading 3 (H3)** | 14 pt | 1.35x | Bold | Top: 12 pt, Bottom: 6 pt |
| **Body (Paragraph)** | 12 pt | 1.60x | Regular | Bottom: 10 pt |
| **Table Header** | 11 pt | 1.40x | Regular | Cell padding: 6 pt |
| **Code Block** | 10.5 pt | 1.40x | Monospace | Container padding: 8 pt |

### 2.3 Semantic Code Blocks

```rust
// Initialize document with default Report theme
let mut ed = Editor::open_template("report")?;
ed.insert_node("root", 0, "{\"id\":\"doc.h1\",\"role\":\"h1\",\"keep_with_next\":true,\"content\":{\"type\":\"text\",\"value\":\"Architecture Overview\"}}")?;
ed.insert_node("root", 1, "{\"id\":\"doc.p1\",\"role\":\"body\",\"content\":{\"type\":\"text\",\"value\":\"Markdown-scale typography with deterministic layout lock.\"}}")?;
let package_bytes = ed.save_bytes()?;
```

### 2.4 Multi-Level Lists

1. **Deterministic Geometry Lock**: Computed once by the layout engine and rendered with sub-pixel precision.
2. **Four Official Themes**: Includes `Report` (default), `Legal`, `Invoice`, and `ClinicalSummary`.
3. **Lossless Roundtrip**: Bidirectional conversion between Markdown and K2F packages.
"#;

    let res = markdown_to_k2f(sample_md, {
        let mut opts =
            MarkdownOptions::new("K2F Typography & Visual Style Specification", "report").unwrap();
        opts.page_size = PageSize::A4;
        opts
    })
    .unwrap();

    let k2f_path = artifact_dir.join("showcase_report.K2F");
    fs::write(&k2f_path, &res.bytes).unwrap();

    let doc = OpenedDocument::open(&res.bytes).unwrap();
    let page_count = doc.page_count();
    println!("Report document page count: {}", page_count);

    for p in 0..page_count {
        let png_bytes = doc.render_page(p, OFFICIAL_PNG_SCALE).unwrap();
        let png_path = artifact_dir.join(format!("showcase_report_page_{}.png", p + 1));
        fs::write(&png_path, png_bytes).unwrap();
        println!(
            "Wrote rendered report page {} to {}",
            p + 1,
            png_path.display()
        );
    }

    let contract_md =
        fs::read_to_string(root.join("tests/fixtures/markdown/contract_cn.md")).unwrap();
    let contract_res = markdown_to_k2f(&contract_md, {
        let mut opts = MarkdownOptions::new("独立顾问协议", "legal").unwrap();
        opts.page_size = PageSize::A4;
        opts
    })
    .unwrap();

    let contract_doc = OpenedDocument::open(&contract_res.bytes).unwrap();
    for p in 0..contract_doc.page_count() {
        let png_bytes = contract_doc.render_page(p, OFFICIAL_PNG_SCALE).unwrap();
        let png_path = artifact_dir.join(format!("showcase_contract_page_{}.png", p + 1));
        fs::write(&png_path, png_bytes).unwrap();
        println!("Wrote contract page {} to {}", p + 1, png_path.display());
    }

    let published = root.join("examples/published");
    for (slug, prefix) in [
        ("invoice", "example_invoice"),
        ("contract", "example_contract"),
    ] {
        let bytes = fs::read(published.join(format!("{slug}.K2F"))).unwrap();
        let ex_doc = OpenedDocument::open(&bytes).unwrap();
        for p in 0..ex_doc.page_count() {
            let png_bytes = ex_doc.render_page(p, OFFICIAL_PNG_SCALE).unwrap();
            let png_path = artifact_dir.join(format!("{prefix}_page_{}.png", p + 1));
            fs::write(&png_path, png_bytes).unwrap();
            println!("Wrote {slug} page {} to {}", p + 1, png_path.display());
        }
    }
}
