# GitHub Actions Workflows Guide

This document describes the CI/CD workflows configured for this project.

## Available Workflows

### 1. CI Workflow (`ci.yml`)

**Trigger**: Automatic on push/PR to main branches when `.rs`, `.toml`, or workflow files change.

**Jobs**:
- `check`: Code formatting and linting
  - Runs `cargo fmt --check`
  - Runs `cargo clippy`
  - Runs `cargo check`

- `test`: Run test suite
  - Executes `cargo test`

- `build`: Build release binary
  - Compiles optimized binary
  - Checks binary size (fails if >10MB)

- `security-audit`: Security checks
  - Runs `cargo audit` for vulnerable dependencies

**Caching**: Uses GitHub Actions cache for cargo registry, git index, and build artifacts.

### 2. Lint Workflow (`lint.yml`)

**Trigger**: Automatic on push/PR to main branches when `.rs` or `.toml` files change.

**Jobs**:
- `rustfmt`: Format validation
- `clippy`: Static analysis
- `toml-check`: TOML syntax validation using taplo

### 3. Release Workflow (`release.yml`)

**Trigger**: Manual via GitHub Actions UI

**Inputs**:
- `version` (required): Version tag (e.g., `v1.0.0`)
- `prerelease` (optional): Mark as pre-release (default: false)

**Process**:
1. Updates version in `Cargo.toml`
2. Builds optimized release binary
3. Runs tests to ensure quality
4. Creates artifacts:
   - Standalone `.exe` file
   - ZIP archive with README and checksums
   - SHA256 checksums file
5. Generates release notes with:
   - Feature list
   - Installation instructions
   - Binary size information
   - Verification instructions
6. Creates GitHub Release with all artifacts
7. Uploads build artifacts (retained for 90 days)

## Creating a Release

1. Navigate to the **Actions** tab in GitHub
2. Click on **Release** workflow
3. Click **Run workflow** button
4. Fill in the form:
   - **Version**: Enter version number (e.g., `v1.0.0`)
   - **Pre-release**: Check if this is a pre-release
5. Click **Run workflow**

The workflow will:
- Build and test the code
- Create a GitHub release
- Attach compiled binaries
- Generate release notes

## Local Testing

Before pushing, you can test locally:

```bash
# Format check
cargo fmt --all -- --check

# Linting
cargo clippy --all-targets --all-features -- -D warnings

# Tests
cargo test

# Build
cargo build --release

# Security audit
cargo install cargo-audit
cargo audit
```

## Workflow Configuration Files

- `.github/workflows/ci.yml` - Main CI pipeline
- `.github/workflows/lint.yml` - Code quality checks
- `.github/workflows/release.yml` - Release automation
- `rustfmt.toml` - Code formatting rules
- `clippy.toml` - Linter configuration

## Caching Strategy

All workflows use GitHub Actions caching to speed up builds:
- Cargo registry cache (dependencies)
- Cargo git index cache
- Target build cache (separate for different job types)

Cache keys are based on `Cargo.lock` hash, ensuring cache invalidation when dependencies change.

## Troubleshooting

### CI Fails on Format Check
Run locally: `cargo fmt --all`

### CI Fails on Clippy
Run locally: `cargo clippy --all-targets --all-features -- -D warnings`
Fix warnings before pushing.

### Release Workflow Fails
- Ensure version number is valid (e.g., `v1.0.0`)
- Check that all tests pass locally
- Verify you have write permissions to the repository

### Binary Size Exceeds Limit
The CI checks that the binary doesn't exceed 10MB. If it does:
- Review dependencies for bloat
- Check if debug symbols are being included
- Verify `strip = true` in `Cargo.toml` release profile

## Security

- Workflows run in isolated environments
- No secrets are required for CI/CD
- Release workflow uses `GITHUB_TOKEN` (automatically provided)
- Security audit runs on every CI check

## Future Improvements

- [ ] Add code coverage reporting
- [ ] Implement automatic changelog generation
- [ ] Add performance benchmarking
- [ ] Create nightly builds
- [ ] Add multi-architecture support (ARM64)
