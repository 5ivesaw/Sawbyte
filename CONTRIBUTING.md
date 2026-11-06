# Contributing to Sawbyte

Sawbyte is a pure-Rust Minecraft client and launcher. We welcome contributions from anyone — whether you're fixing a typo, writing docs, or building new features.

## How to contribute

1. **Fork the repo** and create a branch from `main`
2. **Pick an issue** labeled `good first issue` or `help wanted`, or open a new one
3. **Make your changes** and test them (`cargo test --workspace`)
4. **Open a pull request** — describe what you changed and why

## Where help is needed most

| Area | What | Labels |
|------|------|--------|
| Native GUI | Replace localhost HTTP server with real OS window (X11/Win32), no Chromium/Electron | `native-gui` |
| Packaging | Automate `.deb`, `.AppImage`, `.exe` builds in CI | `packaging` |
| CI | Fix flaky cancellations, re-enable macOS safely | `help wanted` |
| Docs | Improve README, add examples, write tutorials | `documentation` |

## Development setup

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh

# Clone and build
git clone https://github.com/5ivesaw/Sawbyte.git
cd Sawbyte
cargo test --workspace
```

## Rules

- No secrets or keys in code or commits
- MIT licensed — your contributions are too
- Be respectful in issues and PRs
- Test before you push

## Questions?

Open a discussion in [GitHub Discussions](https://github.com/5ivesaw/Sawbyte/discussions) or ask in any issue thread.
