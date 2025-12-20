# Contributing to The Council Of Dicks

First off, thanks for taking the time to contribute! 🎉

## Code of Conduct

Be excellent to each other. We're building a decentralized AI democracy, not a dictatorship.

## How Can I Contribute?

### Reporting Bugs

- **Check existing issues** first
- **Use a clear title** describing the problem
- **Describe steps to reproduce**
- **Include system information** (OS, Rust version, Node version)
- **Attach logs** if applicable

### Suggesting Features

- **Check discussions** to see if it's already proposed
- **Explain the use case** - why is this valuable?
- **Consider the philosophy** - does it align with decentralization and freedom?

### Pull Requests

1. **Fork the repo** and create a branch from `main`
2. **Make your changes**
3. **Add tests** if applicable
4. **Update documentation**
5. **Ensure tests pass**: `cargo test && pnpm test`
6. **Format your code**: `cargo fmt && pnpm format`
7. **Submit PR** with clear description

## 📚 Related Documentation

- **[DEVELOPMENT.md](DEVELOPMENT.md)**: Technical setup guide for contributors.
- **[CORE_VISION.md](docs/CORE_VISION.md)**: Understanding the project's philosophy is crucial for contributing.
- **[ROADMAP.md](docs/ROADMAP.md)**: Check what's planned before suggesting new features.

## Development Setup

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/TheCouncelOfDicks.git
cd TheCouncelOfDicks

# Install dependencies
pnpm install

# Run in dev mode
pnpm tauri dev
```

## Project Structure

- `src-tauri/src/` - Rust backend code
- `src/` - Svelte frontend code
- `docs/` - Documentation
- `tests/` - Tests

## Coding Standards

### Rust

- Follow `rustfmt` defaults
- Use `clippy` recommendations: `cargo clippy`
- Write doc comments for public APIs
- Add unit tests for new functionality

### TypeScript/Svelte

- Use TypeScript for type safety
- Follow Prettier defaults
- Component names in PascalCase
- Keep components focused and small

## Commit Messages

Use clear, descriptive commit messages:

```
feat: add P2P peer discovery via mDNS
fix: resolve heartbeat timeout issue
docs: update installation instructions
refactor: simplify voting algorithm
test: add knowledge bank integration tests
```

## Areas Needing Help

### High Priority
- [ ] P2P networking core (libp2p integration)
- [ ] Safety systems implementation
- [ ] Knowledge bank persistence layer

### Medium Priority
- [ ] Additional AI model integrations
- [ ] Web version (browser-only light node)
- [ ] Performance optimizations

### Always Welcome
- Documentation improvements
- Bug fixes
- UI/UX enhancements
- Test coverage

## Questions?

Open a **Discussion** on GitHub - we're happy to help!

---

*Remember: Every contribution makes the network stronger!* 💪
