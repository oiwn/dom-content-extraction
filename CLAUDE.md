# CLAUDE.md

Project guidance for Claude Code when working with this repository.

## Project Overview

Rust library implementing Content Extraction via Text Density (CETD) algorithm for extracting main content from web pages by analyzing text density patterns.

## Recent Progress

### ✅ Completed Features
- **Markdown Extraction**: Structured markdown output using CETD density analysis
- **HTTP Client**: Migrated to wreq for browser emulation and TLS fingerprinting  
- **Encoding Support**: Full non-UTF-8 encoding support using chardetng

### 🔧 Current Status
- **CLI Tool**: Fully functional with URL/file input, text/markdown output
- **Library API**: Stable with comprehensive feature set
- **Testing**: Comprehensive test suite

## Architecture

### Core Components
- **`DensityTree`** (`src/cetd.rs`): Main structure for text density analysis
- **`DensityNode`** (`src/cetd.rs`): Individual nodes with text density metrics
- **Tree operations** (`src/tree.rs`): HTML traversal and metrics calculation
- **Unicode handling** (`src/unicode.rs`): Proper character counting
- **Utilities** (`src/utils.rs`): Text extraction and link analysis

### Algorithm Flow
1. Parse HTML with `scraper::Html`
2. Build density tree mirroring HTML structure
3. Calculate text density metrics per node
4. Compute composite density scores
5. Extract high-density regions as main content

### Binary Tool
`dce` CLI provides file/URL input with text/markdown output options.

## Development Commands

```bash
# Build and test
cargo build              # Build library
cargo build --release    # Optimized build  
cargo test               # Run tests
cargo bench              # Run benchmarks

# Code quality
cargo fmt                # Format code
cargo clippy             # Lint code
cargo tarpaulin          # Coverage report

# Examples
cargo run --example check -- lorem-ipsum    # Test extraction
cargo run --example check -- test4          # Show density nodes

# CLI usage
cargo run -- --url "https://example.com"        # Extract from URL
cargo run -- --file input.html --output out.txt # Extract from file
cargo run -- --file input.html --format markdown # Markdown output
```

## Project Structure
- `src/lib.rs` - Library interface and API
- `src/cetd.rs` - Core CETD algorithm
- `src/tree.rs` - HTML traversal
- `src/unicode.rs` - Unicode handling
- `src/utils.rs` - Text utilities
- `src/main.rs` - CLI implementation
- `examples/` - Usage examples

## Key Dependencies
- `scraper` - HTML parsing
- `ego-tree` - Tree structure
- `unicode-segmentation` - Unicode handling
- `chardetng` - Encoding detection

## Features

### Available Features
- **`cli`** (default): Command-line interface with URL fetching
- **`markdown`** (default): HTML to markdown conversion

### Feature Usage
```bash
cargo build --no-default-features              # Library only
cargo build --no-default-features --features cli # CLI only
cargo build --no-default-features --features markdown # Markdown only
cargo build                                    # Default (cli + markdown)
```

## Markdown Extraction
- Extracts high-density content as structured markdown
- Uses `htmd` for HTML to markdown conversion
- Feature-gated behind `markdown` flag

## CLI Tool
- `--format text` (default): Plain text extraction
- `--format markdown`: Structured markdown output
- Supports file/URL input with proper error handling

## HTTP Client Migration (Completed ✅)
**Migrated to wreq for browser emulation and TLS fingerprinting:**
- Async runtime with `tokio`
- Chrome 120 browser emulation
- TLS fingerprinting avoidance
- HTTP/2 support with advanced features

## Encoding Support (Enhanced ✅)
**Fixed non-UTF-8 encoding handling:**
- Replaced custom detection with `chardetng`
- Fixed NaN threshold bug in extraction algorithm
- Verified with Windows-1251 Russian content
