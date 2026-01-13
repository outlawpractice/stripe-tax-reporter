# Cross-Platform Support

## Overview

Stripe Tax Reporter supports building and running on all major platforms: macOS, Linux, and Windows.

**Pre-built binaries are available for macOS.** Linux and Windows users can build from source or use `cargo install` (see installation options in README).

## Available Platforms

| Platform | Architecture | Installation Method | Notes |
|----------|--------------|---------------------|-------|
| macOS | Intel (x86_64) | Pre-built binary | Download from releases |
| macOS | Apple Silicon (aarch64) | Pre-built binary | Download from releases |
| Linux | x86_64 | Build from source or cargo install | Pre-built not available due to OpenSSL cross-compilation complexity |
| Windows | x86_64 | Build from source or cargo install | Pre-built not available due to compilation complexity |

## Installation Methods

### 1. Download Pre-built Binary (macOS only)

Pre-built binaries for macOS are available from the [GitHub releases page](https://github.com/outlawpractice/stripe-tax-reporter/releases).

```bash
# macOS (Intel)
curl -L https://github.com/outlawpractice/stripe-tax-reporter/releases/download/v1.0.1/stripe-tax-reporter-macos-x86_64 -o stripe-tax-reporter
chmod +x stripe-tax-reporter
export STRIPE_PROD_API_KEY="sk_live_..."
./stripe-tax-reporter

# macOS (Apple Silicon)
curl -L https://github.com/outlawpractice/stripe-tax-reporter/releases/download/v1.0.1/stripe-tax-reporter-macos-aarch64 -o stripe-tax-reporter
chmod +x stripe-tax-reporter
export STRIPE_PROD_API_KEY="sk_live_..."
./stripe-tax-reporter
```

### 2. Install with Cargo

```bash
cargo install --git https://github.com/outlawpractice/stripe-tax-reporter.git
```

### 3. Build from Source

```bash
git clone https://github.com/outlawpractice/stripe-tax-reporter.git
cd stripe-tax-reporter
cargo build --release
./target/release/stripe-tax-reporter
```

## Local Build Process

Binaries are built locally using Rust toolchains, not GitHub Actions (for cost efficiency and simplicity).

### Build Commands

Use the Rust toolchain directly for each platform:

```bash
# macOS Intel (x86_64)
cargo build --release --target x86_64-apple-darwin

# macOS Apple Silicon (aarch64)
cargo build --release --target aarch64-apple-darwin

# Linux (requires OpenSSL dev setup)
cargo build --release --target x86_64-unknown-linux-gnu

# Windows (requires OpenSSL dev setup)
cargo build --release --target x86_64-pc-windows-msvc
```

### Important: Use Toolchains, Not `cross`

Always use Rust toolchains directly. Do NOT use the `cross` tool, as it adds unnecessary complexity. See CLAUDE.md for more details.

## Build Optimization

### Cargo Configuration

File: `.cargo/config.toml`

Optimizations applied to all release builds:
- `opt-level = 3`: Full optimization for speed
- `lto = true`: Link-time optimization for smaller, faster binaries
- `codegen-units = 1`: Better optimization at the cost of longer compile time
- `strip = true`: Remove debug symbols to reduce binary size

### Binary Size Impact

These optimizations typically reduce binary size by 30-50% compared to standard release builds.

## Verification

### Test the Workflow

The workflow can be tested by:
1. Creating a new release tag: `git tag v1.0.1 && git push origin v1.0.1`
2. Publishing the release on GitHub
3. Checking the release page for attached binaries

**Note:** Only published releases trigger the workflow. Draft releases do not.

### Manual Platform Testing

To test a specific platform build locally:

```bash
# Test macOS Intel build
cargo build --release --target x86_64-apple-darwin

# Test Linux build (requires cross-compilation setup)
cargo build --release --target x86_64-unknown-linux-gnu
```

## Performance

### Native Binaries

Each binary is compiled natively on its target platform:
- **Macros & optimizations** are platform-specific
- **OpenSSL linking** uses the platform's native implementation
- **Performance** is optimized for the specific CPU architecture

This approach is superior to cross-compilation because:
- No cross-compilation complexity or toolchain setup
- True native optimization for each platform
- Guaranteed compatibility and reliability
- Binaries tested in their native environment

### Binary Sizes (Approximate)

- macOS binary: ~15-20 MB
- Linux binary: ~12-18 MB
- Windows binary: ~12-18 MB

## Future Enhancements

### Possible Additions

1. **Automated version updates**: Update download links in README on release
2. **checksum files**: Generate SHA256 checksums for binary verification
3. **Signed binaries**: GPG sign binaries for security
4. **Additional platforms**: aarch64-unknown-linux-gnu (ARM64 Linux), aarch64-apple-darwin (already included)
5. **Homebrew formula**: Enable `brew install stripe-tax-reporter`
6. **Compressed releases**: Compress binaries with gzip or bzip2

## Troubleshooting

### Workflow Not Running

- Ensure the release is **published**, not in draft status
- Check the "Actions" tab on GitHub for workflow logs
- Verify `.github/workflows/release-binaries.yml` is committed to the main branch

### Binary Won't Run on macOS

- Make sure the binary has execute permissions: `chmod +x stripe-tax-reporter`
- Check if it's a code-signing issue (Apple Silicon): `codesign -v stripe-tax-reporter`
- For Apple Silicon, use the `macos-aarch64` binary, not the Intel version

### Binary Not Compatible with Linux

- Ensure you downloaded the correct architecture (typically `x86_64` for modern systems)
- Check kernel compatibility: `uname -m` should output `x86_64`
- If using a different architecture (ARM, etc.), build from source

## References

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Rust Platform Support](https://doc.rust-lang.org/nightly/rustc/platform-support.html)
- [Cargo Build Configuration](https://doc.rust-lang.org/cargo/reference/config.html)
- [softprops/action-gh-release](https://github.com/softprops/action-gh-release)
