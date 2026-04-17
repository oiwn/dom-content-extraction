# Changelog

## 0.4.0

- Fixed compilation with `chardetng` 1.0.0 (API breaking change: `EncodingDetector::new()` and `guess()` now take enum args)
- Relaxed dependency versions to minor-only ranges (e.g. `"1.0.102"` → `"1.0"`) for better semver compatibility
- Added `strsim` dev-dependency for string similarity metrics
- Enhanced `ce_score` evaluation example: added Sorensen-Dice similarity metric alongside existing LCS-based Precision/Recall/F1, refactored to `ScoringResult` struct
- Updated CI workflows: removed deprecated `actions-rs` actions, replaced with `dtolnay/rust-toolchain` + `Swatinem/rust-cache`, updated `codecov/codecov-action` to v5
