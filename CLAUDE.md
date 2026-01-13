# Development Guidelines for Stripe Tax Reporter

## Building Release Binaries

### Important: Use Toolchains, NOT `cross`

When building binaries for multiple platforms, **always use Rust toolchains directly**, never use the `cross` tool.

**Why:**
- `cross` adds unnecessary complexity and dependencies
- Direct toolchain builds are simpler and more reliable
- Rust toolchains are already installed on the build system

**How to build for each platform:**

```bash
# macOS Intel x86_64 (native on Intel Mac)
cargo build --release --target x86_64-apple-darwin

# macOS Apple Silicon aarch64 (native on Apple Silicon Mac)
cargo build --release --target aarch64-apple-darwin

# Linux x86_64 (requires Linux build environment or appropriate OpenSSL setup)
cargo build --release --target x86_64-unknown-linux-gnu

# Windows x86_64 (requires Windows build environment or appropriate OpenSSL setup)
cargo build --release --target x86_64-pc-windows-msvc
```

### Cross-Compilation Notes

For Linux and Windows cross-compilation from macOS:
- OpenSSL cross-compilation requires careful configuration
- Consider providing binaries built on native Linux/Windows runners instead
- For now, macOS binaries are provided; Linux/Windows users can build from source or use `cargo install`

### Available Targets

Check installed Rust targets with:
```bash
rustup target list | grep installed
```

All necessary targets should already be installed on the build system.

## Release Process

1. **Build binaries**: Use `cargo build --release --target <target>` for each platform
2. **Upload to GitHub**: Use `gh release upload <tag>` to attach binaries
3. **Update documentation**: Ensure README.md has download links for all available binaries

## See Also

- `CROSS_PLATFORM_BUILDS.md` - Architecture and build strategy
- `README.md` - User-facing documentation with installation instructions
