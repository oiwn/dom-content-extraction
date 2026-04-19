# dom-content-extraction — Project Overview

Rust library implementing Content Extraction via Text Density (CETD) algorithm.
Extracts main content from web pages by analyzing text density patterns in the HTML DOM.

Based on: Sun, Song & Liao (2011) "DOM Based Content Extraction via Text Density".

For command-line usage, see [`pginf`](https://github.com/oiwn/pageinfo-rs).

## Architecture

### Core Algorithm Flow

1. Parse HTML with `scraper::Html`
2. Build a `DensityTree` (ego-tree) mirroring the DOM structure
3. Calculate per-node text density metrics (char count, tag count, link density)
4. Compute composite text density scores
5. Extract high-density regions as main content

### Source Files

| File | Purpose |
|---|---|
| `src/lib.rs` | Library API: public types, re-exports, `get_content()` convenience function |
| `src/cetd.rs` | Core CETD algorithm: `DensityTree`, `DensityNode`, density calculations, content extraction |
| `src/tree.rs` | HTML tree traversal: `NodeMetrics`, `TreeBuilder` trait, `BODY_SELECTOR`, metrics aggregation |
| `src/unicode.rs` | Unicode handling: grapheme/codepoint counting, text normalization, primary script detection |
| `src/utils.rs` | Utilities: node text extraction, link analysis, DOM construction helpers |
| `src/markdown.rs` | HTML-to-markdown extraction (feature-gated behind `markdown`) |

### Key Types

- **`DensityTree`** — Main entry point. Wraps an `ego_tree::Tree<DensityNode>`. Built from `Html::parse_document()`.
- **`DensityNode`** — Per-element metrics: `node_id`, `metrics` (char/tag/link counts), `density`, `density_sum`.
- **`NodeMetrics`** — Raw counts: `char_count`, `tag_count`, `link_char_count`, `link_tag_count`.
- **`DomExtractionError`** — Error type (currently only `NodeAccessError`).

### Extraction Hardening

The library filters out non-content elements that would pollute extracted text or skew density scoring:

- **Structural skips**: `script`, `noscript`, `style`, `svg`, `template`, `canvas` subtrees
- **Hidden containers**: `hidden`, `aria-hidden="true"`, inline `display:none` / `visibility:hidden`
- **Boilerplate containers**: `robots-nocontent`, `sharedaddy`, `sd-sharing`, `jetpack-likes-widget`, `jp-relatedposts`, `ads__`, `adfox`, `yatag`, `data-content="webR"`
- **Text-fragment classifier**: detects CSS blocks and machine-like JS/config blobs using shape evidence (punctuation density, assignment/call syntax, encoded tokens) without broad keyword filtering

### Examples

| File | Purpose |
|---|---|
| `examples/ce_score.rs` | Evaluation tool: scores extraction against CleanEval dataset using LCS (Precision/Recall/F1) + Sorensen-Dice |
| `examples/check.rs` | Quick extraction test on HTML files |
| `examples/basic.rs` | Minimal usage example |
| `examples/debug_density.rs` | Print density tree for inspection |
| `examples/debug_interfax.rs` | Debug extraction on non-UTF-8 encoded content (Windows-1251) |

### Test Data

| Path | Purpose |
|---|---|
| `html/test_{1..4}.html` | Unit test fixtures with known content structure |
| `data/GoldStandard/` | CleanEval gold-standard `.txt` files (~681 files) |
| `data/finalrun-input/` | Corresponding source `.html` files (~740 files) |

### Features

- **`markdown`**: HTML-to-markdown output via `htmd` (disabled by default)

### Key Dependencies

- `scraper` — HTML parsing (wraps html5ever)
- `ego-tree` — Tree data structure for density tree
- `thiserror` — Error type derives
- `unicode-segmentation` + `unicode-normalization` — Text processing
- `htmd` — HTML to markdown conversion (optional)
- `chardetng` + `encoding_rs` — Encoding detection (dev-only, used in examples)
- `strsim` — String similarity metrics (dev-only, used in evaluation)

### Quality Tools

- `prek` — Pre-commit hooks (`fmt`, `clippy`, `typos`, `gitleaks`)
- `.gitleaks.toml` — Allowlist for HTML test fixtures
