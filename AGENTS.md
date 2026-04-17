# AGENTS.md

Project guidance and behavioral guidelines for Claude Code when working with this repository.

## General Guidelines

Behavioral guidelines to reduce common LLM coding mistakes. Merge with project-specific instructions as needed.

**Tradeoff:** These guidelines bias toward caution over speed. For trivial tasks, use judgment.

### 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:
- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them - don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

### 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

### 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:
- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it - don't delete it.

When your changes create orphans:
- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.

### 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:
- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make it pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:
```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.

---

**These guidelines are working if:** fewer unnecessary changes in diffs, fewer rewrites due to overcomplication, and clarifying questions come before implementation rather than after mistakes.

## Project Overview

Rust library implementing Content Extraction via Text Density (CETD) algorithm for extracting main content from web pages by analyzing text density patterns.

## Recent Progress

### ✅ Completed Features
- **Markdown Extraction**: Structured markdown output using CETD density analysis
- **HTTP Client**: Migrated to wreq for browser emulation and TLS fingerprinting  
- **Encoding Support**: Full non-UTF-8 encoding support using chardetng

### 🔧 Current Status (Updated April 2026)
- **Dependencies**: Updated to latest compatible versions via `cargo update`
- **Security Audit**: Completed with cargo-audit (found fxhash unmaintained, lru unsound advisories)
- **Documentation**: Fixed intra-doc links and HTML tag warnings
- **CLI Tool**: Fully functional with URL/file input, text/markdown output
- **Library API**: Stable with comprehensive feature set
- **Testing**: 32 passing unit tests, 2 integration tests

## Improvement Plan

See `specs/ctx.md` for detailed analysis and roadmap. Key areas:

### 1. Testing Coverage
- Increase unit test density for core algorithms
- Add integration tests for malformed HTML edge cases
- Implement performance regression tests

### 2. Edge-case Handling
- Robust handling of malformed HTML and encoding edge cases
- Error recovery strategies for NaN threshold calculations

### 3. Documentation
- API documentation for internal functions
- Examples for advanced use cases
- Troubleshooting guide

### 4. Performance Optimization
- Profile memory usage across document sizes
- Parallelize tree traversal where possible
- Cache density calculations

### 5. Feature Extensions
- Additional output formats (JSON, structured data)
- Configurable extraction thresholds via builder pattern
- Integration with web scraping frameworks

### 6. Security
- Monitor and address security advisories (fxhash, lru)
- Evaluate alternatives for affected dependencies

### Immediate Next Steps
1. Address TODOs in code (cetd.rs test, example optimization)
2. Add 5+ new unit tests for edge cases
3. Implement JSON output format
4. Profile and benchmark current performance
5. Evaluate security advisories

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
cargo tarpaulin          # Coverage report (requires installation)
cargo audit              # Security audit (requires cargo-audit installation)

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
