# Changelog

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
