# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust library implementing the Content Extraction via Text Density (CETD) algorithm for extracting main content from web pages. The core concept analyzes text density patterns to distinguish content-rich sections from navigational elements.

## Architecture

### Core Components

- **`DensityTree`** (`src/cetd.rs`): Main structure representing text density analysis of HTML documents. Contains methods for building density trees, calculating metrics, and extracting content.
- **`DensityNode`** (`src/cetd.rs`): Individual nodes containing text density metrics (character count, tag count, link density).
- **Tree operations** (`src/tree.rs`): HTML document traversal and node metrics calculation.
- **Unicode handling** (`src/unicode.rs`): Proper character counting using grapheme clusters and Unicode normalization.
- **Utilities** (`src/utils.rs`): Helper functions for text extraction and link analysis.

### Algorithm Flow

1. Parse HTML document using `scraper::Html`
2. Build density tree mirroring HTML structure (`DensityTree::from_document`)
3. Calculate text density metrics for each node
4. Compute composite density scores (`calculate_density_sum`)
5. Extract high-density regions as main content

### Binary Tool

The `dce` binary (`src/main.rs`) provides CLI access to the library functionality, supporting both local files and URL fetching.

## Development Commands

### Build and Test
```bash
cargo build              # Build library
cargo build --release    # Optimized build
cargo test               # Run tests
cargo bench              # Run benchmarks
```

### Code Quality
```bash
cargo fmt                # Format code (max_width = 84, see rustfmt.toml)
cargo clippy             # Lint code
cargo tarpaulin          # Generate coverage report (target: 80%+, see .tarpaulin.toml)
just coverage            # Alternative coverage command (requires just)
```

### Examples
```bash
cargo run --example check -- lorem-ipsum    # Extract from generated lorem ipsum
cargo run --example check -- test4          # Show highest density node
cargo run --example ce_score                # Benchmark against CleanEval dataset
```

### Binary Usage
```bash
cargo run --bin dce -- --url "https://example.com"        # Extract from URL
cargo run --bin dce -- --file input.html --output out.txt # Extract from file
```

## Project Structure

- `src/lib.rs` - Main library interface and public API
- `src/cetd.rs` - Core CETD algorithm implementation
- `src/tree.rs` - HTML tree traversal and metrics
- `src/unicode.rs` - Unicode-aware text processing
- `src/utils.rs` - Text extraction utilities
- `src/main.rs` - CLI binary implementation
- `examples/` - Usage examples and benchmarking tools

## Key Dependencies

- `scraper` - HTML parsing and CSS selector support
- `ego-tree` - Tree data structure for density calculations
- `unicode-segmentation` - Proper Unicode grapheme handling
- `unicode-normalization` - Text normalization for consistent processing

## Features

- Default features include CLI functionality (`cli` feature)
- Library can be used without CLI dependencies by disabling default features
- Optional `markdown` feature for structured markdown extraction using density analysis

## Markdown Extraction Implementation

**Goal**: Add markdown extraction capability that leverages CETD density analysis to extract main content as structured markdown.

**Approach**:
- Create completely separate `src/markdown.rs` module (do not modify CETD algorithm)
- Use existing density analysis to identify high-density content nodes
- Extract HTML subtrees for those nodes using their NodeIDs
- Convert HTML to markdown using `htmd` library
- Add as optional `markdown` feature flag

**Implementation Steps**:
1. ✅ Add `htmd` dependency with `markdown` feature flag to Cargo.toml
2. ✅ Create `src/markdown.rs` with main API: `extract_content_as_markdown()`
3. ✅ Add markdown module to `src/lib.rs` with feature gating
4. ✅ Mirror logic from `DensityTree::extract_content()` but collect NodeIDs instead of text
5. ✅ Implement HTML container extraction using scraper's NodeID→HTML mapping
6. ✅ Integrate `htmd` for HTML→Markdown conversion
7. ✅ Add error handling and basic tests

**Current Status**: ✅ Implementation complete and working

**Resolution**:
- Simplified approach: Use `get_max_density_sum_node()` to find highest density content
- Handle text nodes by walking up the tree to find parent elements
- Extract HTML using `ElementRef::inner_html()` method
- Convert to markdown using `htmd::HtmlToMarkdown` with script/style tags skipped
- Proper error handling following existing patterns

**Key Implementation Details**:
- Uses `ElementRef::wrap()` to convert scraper nodes to elements
- Walks up parent tree when max density node is text (whitespace)
- Returns empty string when no content found (consistent with existing behavior)
- Trims markdown output for clean results

**Test Results**:
- ✅ Test `test_extract_content_as_markdown` passes
- ✅ All existing tests continue to pass
- ✅ Generated markdown includes proper formatting (headers, paragraphs)
- ✅ Works with both markdown feature enabled and disabled
