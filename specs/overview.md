# dom-content-extraction — Project Overview

Rust library implementing Content Extraction via Text Density (CETD) algorithm.
Extracts main content from web pages by analyzing text density patterns in the HTML DOM.

Based on: Sun, Song & Liao (2011) "DOM Based Content Extraction via Text Density".

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
| `src/main.rs` | CLI binary (`dce`): URL/file input, text/markdown output, encoding detection via chardetng |
| `src/markdown.rs` | HTML-to-markdown extraction (feature-gated behind `markdown`) |

### Key Types

- **`DensityTree`** — Main entry point. Wraps an `ego_tree::Tree<DensityNode>`. Built from `Html::parse_document()`.
- **`DensityNode`** — Per-element metrics: `node_id`, `metrics` (char/tag/link counts), `density`, `density_sum`.
- **`NodeMetrics`** — Raw counts: `char_count`, `tag_count`, `link_char_count`, `link_tag_count`.
- **`DomExtractionError`** — Error type (currently only `NodeAccessError`).

### Examples

| File | Purpose |
|---|---|
| `examples/ce_score.rs` | **Evaluation tool**: scores extraction against CleanEval gold-standard dataset using LCS (Precision/Recall/F1) + Sorensen-Dice similarity |
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

- **`cli`** (default): Binary with URL fetching via `wreq`, encoding detection via `chardetng`
- **`markdown`** (default): HTML-to-markdown output via `htmd`

### Key Dependencies

- `scraper` — HTML parsing (wraps html5ever)
- `ego-tree` — Tree data structure for density tree
- `thiserror` — Error type derives
- `unicode-segmentation` + `unicode-normalization` — Text processing
- `htmd` — HTML to markdown conversion (optional)
- `wreq` — HTTP client with browser emulation (optional, CLI only)
- `chardetng` — Encoding detection (optional, CLI only)
- `strsim` — String similarity metrics (dev-only, used in evaluation)
