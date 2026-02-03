# Mecalin Project Context

## Project Overview
Mecalin is a typing tutor application built with GTK4, Rust, and Adwaita. It follows modern GNOME application patterns with GTK Builder XML UI templates and GResource embedding.

## Technology Stack
- **Language**: Rust
- **UI Framework**: GTK4 4.14+
- **Design**: libadwaita 1.5+ (Adwaita design system)
- **Build System**: Meson (production), Cargo (development)
- **Architecture**: GTK Builder with XML UI templates

## Development Workflow

### Quick Commands
```bash
# Development/testing
cargo run

# Production build
meson setup builddir
meson compile -C builddir
./builddir/mecalin
```

### Pre-commit Requirements
- **ALWAYS run `cargo fmt` before committing**
- Run `cargo clippy` to check for warnings
- Ensure code compiles without errors

## Release Process

### Version Bump
1. Update version in `Cargo.toml`
2. Update version in `meson.build`
3. Add release entry in `data/io.github.nacho.mecalin.metainfo.xml` with:
   - New version number
   - Release date (YYYY-MM-DD format, use current date)
   - List of changes since last release

### Generating Changelog
Review commits since last release:
```bash
git log vPREVIOUS..HEAD --oneline --no-merges
```

Organize changes into categories:
- **Added**: New features (Added hand widget, Added French translation)
- **Improved**: Enhancements (Improved keyboard highlighting, Improved performance)
- **Fixed**: Bug fixes (Fixed cursor position, Fixed WPM calculation)
- **Updated**: Translations and dependencies (Updated Italian translation, Updated dependencies)

### Creating Release
```bash
# Format code
cargo fmt

# Update Cargo.lock with new version
cargo update -p mecalin

# Commit version bump
git commit -am "Release X.Y.Z"

# Tag release
git tag vX.Y.Z

# Push changes and tag
git push && git push --tags
```

### Code Conventions
- Follow Rust standard conventions (rustfmt, clippy)
- Use GTK4/Adwaita patterns for UI components
- Embed UI resources using GResource
- Separate UI templates (XML) from logic (Rust)

## Project Structure
- UI templates should be in XML format for GTK Builder
- Follow GNOME application structure guidelines
- Use Adwaita design patterns for consistent UX

## Icon Design
- Application icon: `io.github.nacho.mecalin.svg`
- **MUST follow GNOME HIG palette**: https://developer.gnome.org/hig/reference/palette.html
- Use only colors from the official GNOME palette (Light 1-5, Dark 1-5, Blue, Green, Yellow, Orange, Red, Purple, Brown)

## Dependencies Management
- Core dependencies: GTK4, libadwaita, Rust toolchain, Meson
- Keep dependencies minimal and well-justified
- Prefer stable, well-maintained crates

## AI Assistant Guidelines
- Prioritize GTK4/Adwaita best practices
- Suggest modern Rust patterns appropriate for GUI applications
- Consider both development (Cargo) and production (Meson) build workflows
- Focus on GNOME HIG compliance for UI suggestions
- Keep code concise and maintainable
- Always remind about running `cargo fmt` before commits
