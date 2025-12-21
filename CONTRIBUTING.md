# Contributing to Bangla Calendar

Thank you for your interest in contributing! 🙏

## Development Setup

### Prerequisites
- [Rust](https://rustup.rs/) (2024 edition)
- Windows 10/11 (required for Win32 APIs)
- Windows SDK (for MSIX packaging)

### Building Locally

```bash
# Clone the repository
git clone https://github.com/abusayed0206/bangla-calendar.git
cd bangla-calendar

# Build debug version
cargo build

# Build release version
cargo build --release

# Run the application
cargo run --release
```

## Branching Strategy

- `main` - Stable release branch
- `develop` - Development branch (if used)
- `feature/*` - Feature branches
- `fix/*` - Bug fix branches

## Commit Convention

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>: <description>

[optional body]
```

### Types
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `chore:` - Maintenance tasks
- `refactor:` - Code refactoring
- `ui:` - UI/UX changes
- `perf:` - Performance improvements

### Examples
```
feat: add Bangla month name display
fix: correct date calculation for leap years
docs: update installation instructions
chore: bump windows crate to v0.61
```

## Pull Request Process

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Run checks: `cargo fmt && cargo clippy`
5. Commit with conventional commit message
6. Push and create a Pull Request

## Release Process

Releases are automated via GitHub Actions:

1. **Manual Release:**
   - Go to Actions → "Bump Version" workflow
   - Click "Run workflow"
   - Select bump type (patch/minor/major)
   - Optionally add pre-release tag (alpha/beta/rc)

2. **Automatic:**
   - Version is updated in `Cargo.toml`
   - Git tag is created and pushed
   - Release workflow builds binaries and MSIX
   - GitHub Release is created with artifacts

## Labels for PRs

Use these labels for automatic changelog categorization:

| Label | Category |
|-------|----------|
| `feature`, `enhancement` | 🚀 Features |
| `bug`, `fix` | 🐛 Bug Fixes |
| `ui`, `ux` | 🎨 UI/UX |
| `dependencies` | 📦 Dependencies |
| `documentation` | 📚 Documentation |
| `chore`, `refactor` | 🔧 Maintenance |
| `i18n`, `bangla` | 🌍 Localization |

## Code Style

- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- No warnings in release builds
- Document public functions
- Use meaningful variable names
