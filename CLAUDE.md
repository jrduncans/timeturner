# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Build
cargo build --release

# Test
cargo test --verbose

# Format (run before committing — CI enforces this)
cargo fmt --all

# Lint (warnings and pedantic clippy treated as errors)
cargo clippy --all-targets --all-features -- -D warnings -D clippy::pedantic

# Run a single test
cargo test <test_name>
```

## Architecture

Timeturner is a CLI tool that parses date-time strings from multiple input formats and converts them into multiple output representations simultaneously.

**Data flow:**

```
CLI args (clap) → lib.rs::run()
  → parsing.rs::parse_input()   → DateTime<Utc>
  → converting.rs::convert()    → Vec<ConversionResult>
  → Output: ValuePerLine (one per line) or Alfred (JSON)
```

**Key modules:**

- `main.rs` — CLI entry point using `clap` derive macros; delegates entirely to `lib::run()`
- `lib.rs` — Orchestrates parsing and conversion; defines `OutputMode` and `DurationUnit` enums
- `parsing.rs` — Accepts 10+ input formats (RFC3339, epoch millis, natural language, etc.); uses the `dateparser` crate plus custom parsing for edge cases (commas, UTC suffixes)
- `converting.rs` — Generates multiple output formats from a single `DateTime<Utc>`: RFC3339 UTC, RFC3339 local, epoch milliseconds, human-readable duration, and optional specific duration units
- `alfred.rs` — Alternate output mode formatting results as Alfred workflow JSON

The `--duration-unit` flag (`-d`) changes how the human-readable duration is expressed (seconds, minutes, hours, days, weeks, or fortnights). Without it, `converting.rs` uses `humantime` to auto-format.

**CI runs tests with `TZ=America/Los_Angeles`** — timezone-sensitive tests depend on this.
