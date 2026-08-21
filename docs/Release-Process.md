# Release Process

This document describes how to publish a new release of BBKEmu.

## Prerequisites

- Push access to the `master` branch
- Permission to create tags and releases on GitHub

## Steps

### 1. Update version numbers

Version numbers must be updated in **three files**:

| File | Field | Current |
|------|-------|---------|
| `Cargo.toml` (workspace root) | `[workspace.package] version` | `"0.1.0"` |
| `crates/bbkemu-libretro/bbkemu_libretro.info` | `display_version` | `"0.1.0"` |
| `crates/bbkemu-libretro/src/lib.rs` | `library_version` | `"0.1.0"` |

All values must match. The `.info` file is copied directly into the release
artifacts — RetroArch reads `display_version` to display the core version to
users. The `library_version` in `lib.rs` is returned by the libretro API.

```bash
# Example: bumping to 0.2.0
# 1. Edit Cargo.toml
sed -i 's/^version = "0.1.0"/version = "0.2.0"/' Cargo.toml

# 2. Edit .info file
sed -i 's/^display_version = "0.1.0"/display_version = "0.2.0"/' \
  crates/bbkemu-libretro/bbkemu_libretro.info

# 3. Edit lib.rs
sed -i 's/library_version: c"0.1.0"/library_version: c"0.2.0"/' \
  crates/bbkemu-libretro/src/lib.rs
```

### 2. Commit the version bump

```bash
git add Cargo.toml Cargo.lock crates/bbkemu-libretro/bbkemu_libretro.info crates/bbkemu-libretro/src/lib.rs
git commit -m "chore: bump version to 0.2.0"
git push origin master
```

### 3. Create and push a tag

The release workflow triggers on tags matching `v*` (e.g. `v0.2.0`).

```bash
git tag v0.2.0
git push origin v0.2.0
```

### 4. CI builds and creates a draft release

Pushing the tag triggers `.github/workflows/release.yml`, which:

1. **Builds standalone binaries** for Linux (x86_64 + aarch64), macOS (x86_64 + aarch64), and Windows (x86_64)
2. **Builds libretro cores** for the same platforms, renaming the cdylib to
   `bbkemu_libretro.<ext>` and bundling `bbkemu_libretro.info`
3. **Creates a draft GitHub Release** with:
   - Auto-generated release notes (PRs and commits since the previous tag)
   - A download table for standalone and libretro artifacts
   - All build artifacts attached

### 5. Review and publish the release

1. Go to [Releases](https://github.com/AloysHF/BBKEmu/releases)
2. Find the draft release created by CI
3. Review the auto-generated changelog — edit if needed (可以参考之前正式发布版本的 changelog)
4. Verify all expected artifacts are attached:
   - `bbkemu-linux-x86_64.tar.gz`
   - `bbkemu-linux-aarch64.tar.gz`
   - `bbkemu-macos-x86_64.tar.gz`
   - `bbkemu-macos-aarch64.tar.gz`
   - `bbkemu-windows-x86_64.zip`
   - `*-libretro.*` (one per platform)
5. Click **Publish release**

### 6. Sync `.info` file to upstream libretro-super

RetroArch's **Online Updater > Core Downloader** reads the `.info` file from the
upstream [libretro-super](https://github.com/libretro/libretro-super) repository
(`dist/info/bbkemu_libretro.info`), not from this repo. If the `.info` file
was changed in this release (version, metadata, supported extensions, etc.), a
PR must be submitted to sync the changes upstream.

1. Fork [libretro/libretro-super](https://github.com/libretro/libretro-super)
2. Copy `crates/bbkemu-libretro/bbkemu_libretro.info` from this repo
   to `dist/info/bbkemu_libretro.info` in the fork
3. Submit a PR to `libretro/libretro-super` — reference the BBKEmu release
   tag and list the changed fields in the PR description

## Troubleshooting

### CI build fails

- Check the [Actions](https://github.com/AloysHF/BBKEmu/actions) tab
  for the failed run
- The most common failure is a missing Linux build dependency — the CI installs
  `libasound2-dev` automatically

### Re-triggering a release

The release workflow only runs on tag pushes. To re-trigger:

```bash
# 1. Delete the tag locally and remotely
git tag -d v0.2.0
git push origin --delete v0.2.0

# 2. Re-push the tag (CI will re-run)
git push origin v0.2.0
```

If a draft release was already created by the failed run, delete it from the
[Releases](https://github.com/AloysHF/BBKEmu/releases) page before
re-pushing the tag, otherwise the new run may conflict with the existing draft.

### Release artifacts missing

- Verify the tag name starts with `v` (e.g. `v0.2.0`, not `0.2.0`)
- The release workflow only triggers on tag pushes (`v*`); manually dispatching
  the workflow from the Actions tab will not create a release

### Version mismatch in RetroArch

- Ensure `Cargo.toml`, `bbkemu_libretro.info`, and `lib.rs` all have the same
  version string
- The `.info` file is bundled as-is into the release artifacts
