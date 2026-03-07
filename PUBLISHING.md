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

The repository includes a GitHub Action `.github/workflows/publish.yml` that automates this process.

1.  **Add PyPI Token**: Go to your GitHub repository -> Settings -> Secrets and variables -> Actions -> New repository secret.
    - Name: `PYPI_API_TOKEN`
    - Value: `pypi-your-token-here`
2.  **Trigger Release**: To publish a new version:
    - Update the version in `Cargo.toml`.
    - Create a new tag: `git tag v0.1.0`.
    - Push the tag: `git push origin v0.1.0`.

The action will build wheels for multiple platforms (Linux, macOS, Windows) and upload them to PyPI.

## Versioning

Follow [Semantic Versioning](https://semver.org/). Update `version` in `Cargo.toml` before every release.
