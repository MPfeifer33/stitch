# PROJECT.md — stitch

**What:** Context rebuilder — gathers project state into a compact orientation brief for cold-starting agents.

**Status:** MVP complete, published to github.com/MPfeifer33/stitch

## Architecture
- `src/cli.rs` — Clap 4 CLI: `rebuild` (--depth, --contents), `sources`, `brief`
- `src/gather.rs` — ProjectContext with git state, file structure (WalkDir), key file detection (PROJECT.md, README, manifests), recent commits, evidence source detection. generate_brief() produces compact cold-start orientation.
- `src/report.rs` — Context display, sources checklist, brief output (text + JSON)
- `src/main.rs` — Standard error handling

## Usage
```bash
# Rebuild full project context
stitch rebuild

# Rebuild with file contents included (deeper)
stitch rebuild --contents

# Limit directory traversal depth
stitch rebuild --depth 3

# List available context sources
stitch sources

# Generate a compact cold-start brief
stitch brief
```

## Design Decisions
- Detects key files automatically (PROJECT.md, README, Cargo.toml, package.json, etc.)
- Brief format designed for injection into agent system prompts
- Lightweight — no database, just reads filesystem and git state
- Evidence source detection for integration with witness/latch/probe

## Last Updated
June 22, 2026 — Initial MVP
