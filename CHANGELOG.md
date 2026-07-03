# Changelog

## 0.4.4

### Added

- `DensityTree::extract_article` — plain-text article extractor that anchors at the densest subtree and walks up to its nearest `<article>`/`<main>`/`<section>`/`<div>`/`<content>` container. Avoids the contiguous-block heuristic in `extract_content` that pulls in high-density sidebar/ticker elements appearing before the article body in DOM order. Mirrors the anchor-and-walk-up strategy of `extract_content_as_markdown`, but renders the container as plain text. Works without the `markdown` feature.
- `get_article(&document)` — convenience wrapper around `DensityTree::extract_article`, parallel to `get_content`.
- `tests/e2e_article.rs` — regression test against a real theblock.co fixture (ticker excluded, article body present).
- `test_extract_article_excludes_ticker` unit test in `src/cetd.rs`.

### Changed

- Dependency bumps: `scraper` 0.26 → 0.27, `thiserror` 2.0 → 2, `zip` (dev) 8.5 → 8.6.
- `html/pages.zip` fixture corpus cleaned and compressed (~45% smaller, 4.6 MB → 2.7 MB); added the theblock.co Bessent/CBDC violator fixture. Leak-regression fixtures retain their `<script>`/`<style>`/`<svg>` so markdown-filtering coverage stays intact.
- `examples/check_pages.rs` — new `--article`/`-a` mode for ticker-clean article extraction.

### Notes

- `extract_content` and `get_content` are unchanged and remain for backward compatibility. Prefer `extract_article`/`get_article` when you need just the article body without sidebar/ticker noise.

## 0.4.3

### Fixed

- Markdown extraction (`extract_content_as_markdown`) no longer leaks raw HTML into the output:
  - `data:` URI images and `srcset` placeholders are dropped from `<img>`/`<source>`/`<picture>` before htmd conversion
  - TinyMCE editor bookmark `<span data-mce-type="...">` elements are pruned
  - `<div class="hidden">` (Tailwind/Bootstrap utility-class wrappers) and their entity-escaped contents no longer leak through
- `should_skip_element` now recognizes Tailwind/Bootstrap hiding utilities (`hidden`, `invisible`, `sr-only`) as `class` attribute tokens, in addition to the existing `hidden` HTML attribute, `aria-hidden`, and inline `style` checks

### Added

- `filtered_inner_html` helper (`src/utils.rs`, feature-gated behind `markdown`) — the markdown-path analog of `collect_text_filtered`. Prunes non-content subtrees before handing HTML to htmd so the markdown path uses the same skip rules as the text path
- `tests/e2e_leaks.rs` — regression tests against four real-world fixtures (theblock.co, bitcoinmagazine.com, cryptoslate.com, decrypt.co) loaded from `html/pages.zip`
- `examples/check_pages.rs` — manual inspection helper that runs text or markdown extraction over every page in `html/pages.zip`
- Four leak-pattern fixtures added to `html/pages.zip`

## 0.4.1

### Breaking Changes

- Removed `dce` CLI binary and `cli` feature. This is now a library-only crate. CLI extraction lives in [`pginf`](https://github.com/oiwn/pageinfo-rs).
- Default features changed from `["cli", "markdown"]` to `[]`.

### Changed

- Hardened text extraction against CSS/JS/SVG/config pollution in parsed DOM text
- Structural element filtering: skip `script`, `noscript`, `style`, `svg`, `template`, `canvas`, hidden elements, and common ad/share boilerplate containers
- Conservative text-fragment classifier for machine-like blobs (CSS blocks, JS/config patterns, encoded tokens) without broad keyword filtering
- Shared filtering applied in `get_node_text`, `DensityTree::build_density_tree`, and `HtmlTreeBuilder`
- Fixed clippy warnings in `benches/simple.rs` (deprecated `black_box`, unnecessary `let`) and `examples/debug_density.rs` (collapsible `if`)
- Added `prek.toml` for pre-commit hooks (`fmt`, `clippy`, `typos`, `gitleaks`)
- Added `.gitleaks.toml` allowlist for HTML test fixtures

## 0.4.0

- Fixed compilation with `chardetng` 1.0.0 (API breaking change: `EncodingDetector::new()` and `guess()` now take enum args)
- Relaxed dependency versions to minor-only ranges (e.g. `"1.0.102"` → `"1.0"`) for better semver compatibility
- Added `strsim` dev-dependency for string similarity metrics
- Enhanced `ce_score` evaluation example: added Sorensen-Dice similarity metric alongside existing LCS-based Precision/Recall/F1, refactored to `ScoringResult` struct
- Updated CI workflows: removed deprecated `actions-rs` actions, replaced with `dtolnay/rust-toolchain` + `Swatinem/rust-cache`, updated `codecov/codecov-action` to v5
