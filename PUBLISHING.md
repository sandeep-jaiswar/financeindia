# Publishing financeindia to PyPI

This guide explains how to build and publish `financeindia` to PyPI using `maturin`.

## Prerequisites

1.  **PyPI Account**: Register at [pypi.org](https://pypi.org/account/register/).
2.  **API Token**: Create an API token in your PyPI account settings.
3.  **maturin**: Ensure you have `maturin` installed: `pip install maturin`.

## Local Build & Publish

### 1. Build Wheels
Build the extension for your current platform:
```bash
maturin build --release
```

### 2. Publish
You can publish directly using `maturin`:
```bash
maturin publish
```
It will prompt for your PyPI username (use `__token__`) and password (your API token).

## Automated Publishing with GitHub Actions

Two workflows cooperate to release `financeindia`:

1.  **`.github/workflows/release-please.yml`** — Drives semantic versioning.
    It watches [Conventional Commits](https://www.conventionalcommits.org/) on
    `main`. When commit types warrant a release, it opens (and keeps up to date)
    a "Release PR" that bumps the version in `Cargo.toml` / `Cargo.lock`, updates
    `CHANGELOG.md`, and (once merged) creates a `vX.Y.Z` git tag and a GitHub
    Release.

2.  **`.github/workflows/publish.yml`** — Builds & publishes.
    Triggered by any `v*` tag push, it cross-compiles wheels for Linux (incl.
    musl), Windows, and macOS, generates build provenance attestations, and
    uploads them to PyPI with `uv publish`.

### Prerequisites
- **PyPI Token**: Go to your GitHub repository → Settings → Secrets and variables → Actions → New repository secret.
  - Name: `PYPI_API_TOKEN`
  - Value: `pypi-your-token-here`

(Trusted publishing via OIDC can be enabled later by swapping the `UV_PUBLISH_TOKEN`
step for `uv publish --trusted-publishing always`.)

### Release flow (automated)
1. Merge `feat` / `fix` / etc. PRs to `main` using conventional commit messages.
2. `release-please` opens a `chore: release X.Y.Z` PR. Review and merge it.
3. The merge creates the `vX.Y.Z` tag + GitHub Release, which triggers
   `publish.yml` to build and publish the wheels to PyPI.

To cut the current `0.1.0` baseline manually (e.g. for the very first stable
upload), you can push a tag yourself:

```bash
git tag v0.1.0
git push origin v0.1.0
```

### Versioning

This project follows [Semantic Versioning](https://semver.org/). The version
lives in `Cargo.toml` and is read dynamically by `pyproject.toml`
(`dynamic = ["version"]`). **Do not bump it by hand** — `release-please`
manages it from commit history. The current released version is tracked in
`.release-please-manifest.json`.

Commit type → version bump:
- `fix:` → patch (0.1.0 → 0.1.1)
- `feat:` → minor (0.1.0 → 0.2.0)
- `feat!:` / `fix!:` / `BREAKING CHANGE:` → major (0.1.0 → 1.0.0)
