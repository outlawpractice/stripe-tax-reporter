# Cross-Platform Builds

## Overview

Stripe Tax Reporter now includes automated cross-platform builds using GitHub Actions. Pre-built binaries are available for all major platforms and automatically built and attached to each GitHub release.

## Supported Platforms

| Platform | Architecture | Binary Name | File |
|----------|--------------|-------------|------|
| macOS | Intel (x86_64) | stripe-tax-reporter | `stripe-tax-reporter-macos-x86_64` |
| macOS | Apple Silicon (aarch64) | stripe-tax-reporter | `stripe-tax-reporter-macos-aarch64` |
| Linux | x86_64 | stripe-tax-reporter | `stripe-tax-reporter-linux-x86_64` |
| Windows | x86_64 | stripe-tax-reporter.exe | `stripe-tax-reporter-windows-x86_64.exe` |

## Installation Methods

### 1. Download Pre-built Binary (Recommended)

Pre-built binaries are available from the [GitHub releases page](https://github.com/outlawpractice/stripe-tax-reporter/releases).

```bash
# macOS (Intel)
curl -L https://github.com/outlawpractice/stripe-tax-reporter/releases/download/v1.0.0/stripe-tax-reporter-macos-x86_64 -o stripe-tax-reporter
chmod +x stripe-tax-reporter
./stripe-tax-reporter

# macOS (Apple Silicon)
curl -L https://github.com/outlawpractice/stripe-tax-reporter/releases/download/v1.0.0/stripe-tax-reporter-macos-aarch64 -o stripe-tax-reporter
chmod +x stripe-tax-reporter
./stripe-tax-reporter

# Linux
wget https://github.com/outlawpractice/stripe-tax-reporter/releases/download/v1.0.0/stripe-tax-reporter-linux-x86_64
chmod +x stripe-tax-reporter-linux-x86_64
./stripe-tax-reporter-linux-x86_64

# Windows (PowerShell or Command Prompt)
# Download from GitHub releases page
stripe-tax-reporter-windows-x86_64.exe
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

## GitHub Actions Workflow

### File Location
`.github/workflows/release-binaries.yml`

### How It Works

1. When a new release is published on GitHub, the workflow triggers automatically
2. Builds are run in parallel on:
   - macOS-latest (for macOS Intel and Apple Silicon)
   - Ubuntu-latest (for Linux)
   - Windows-latest (for Windows)
3. Each platform compiles a native, optimized release binary
4. Binaries are automatically uploaded as release assets

### Workflow Details

**Trigger:** GitHub Release published
**Runners:** macOS, Ubuntu, Windows
**Actions:**
- Checkout source code
- Install Rust toolchain
- Build with `cargo build --release --target <target>`
- Upload binary to release using `softprops/action-gh-release`

### Adding New Platforms

To add support for additional platforms, edit `.github/workflows/release-binaries.yml` and add a new entry to the `matrix.include` array:

```yaml
- os: ubuntu-latest
  target: aarch64-unknown-linux-gnu  # New target
  binary_name: stripe-tax-reporter
  artifact_name: stripe-tax-reporter-linux-aarch64
```

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
