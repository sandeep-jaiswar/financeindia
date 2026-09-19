# Contributing to financeindia

First off, thank you for considering contributing to `financeindia`! It's people like you who make it a great tool for everyone.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How Can I Contribute?](#how-can-i-contribute)
- [Development Setup](#development-setup)
- [Workflow](#workflow)
- [Testing](#testing)
- [Commit Messages](#commit-messages)
- [Pull Request Process](#pull-request-process)
- [Reporting Security Issues](#reporting-security-issues)

## Code of Conduct

This project adheres to the Contributor Covenant [code of conduct](https://www.contributor-covenant.org/version/2/0/code_of_conduct.html). By participating, you are expected to uphold this code. Please report unacceptable behavior to jaiswarsandeep119@gmail.com.

## How Can I Contribute?

### 💡 Reporting Bugs
- **Check existing issues** first to avoid duplicates
- Use the GitHub issue tracker with the "bug" label
- Include:
  - Clear, descriptive title
  - Steps to reproduce
  - Expected vs. actual behavior
  - Python version, OS, and `financeindia` version
  - Any relevant logs or error messages (use code blocks)
  - Minimal reproducible example (MRE)

**Example**:
```markdown
**Description**: get_equity_list() returns 500 error on weekends

**Steps to Reproduce**:
1. Call `client.get_equity_list()` on Saturday 2026-09-20
2. See error: `HTTPError 500`

**Expected**: Should return empty list or appropriate weekend status

**Environment**:
- Python 3.11
- financeindia 0.2.3
- Linux 5.15
```

### 💻 Suggesting Enhancements
- Use GitHub issues with the "enhancement" label
- Clearly describe the desired behavior and why it's useful
- Provide examples of how the feature would be used
- Link related issues or discussions

**Example**:
```markdown
**Enhancement**: Add caching layer for frequently-requested data

**Use Case**: Repeated calls to `get_equity_list()` within the same session
could use a 5-minute cache to reduce network load.

**Proposed API**:
```python
client = financeindia.FinanceClient(cache_ttl_seconds=300)
```
```

### 🔧 Code Contributions (Bug Fixes & Features)
1. **Fork the repository** and create a feature branch:
   ```bash
   git checkout -b fix/issue-123-description
   # or
   git checkout -b feat/new-feature-name
   ```

2. **Make your changes** following the design principles (see below)

3. **Add tests** for your changes (see [Testing](#testing))

4. **Commit with conventional commits** (see [Commit Messages](#commit-messages))
   ```bash
   git commit -m "fix: resolve issue with equity list parsing"
   ```

5. **Push and create a pull request** (see [Pull Request Process](#pull-request-process))

### 📖 Documentation Contributions
- Typos in README, CHANGELOG, or docs? Submit a fix!
- Examples unclear? Suggest improvements
- Architecture documentation missing? Help us document it
- No PR needed for minor fixes; issues welcome

### 🎯 Good First Issues

Look for issues labeled `good-first-issue` — these are great entry points for new contributors:
- Self-contained fixes
- Clear expected outcome
- Mentorship available

---

## Development Setup

### Prerequisites
- **Rust**: [Install 1.88+](https://www.rust-lang.org/tools/install) (2024 edition)
- **Python**: 3.8 or newer
- **maturin**: Build PyO3 extensions
  ```bash
  pip install maturin
  ```

### Quick Start

```bash
# Clone the repository
git clone https://github.com/sandeep-jaiswar/financeindia.git
cd financeindia

# Create virtual environment (optional but recommended)
python -m venv .venv
source .venv/bin/activate  # On Windows: .venv\Scripts\activate

# Install development dependencies
pip install pytest pytest-asyncio

# Build the extension in develop mode (rebuilds on code changes)
maturin develop

# Verify installation
python -c "import financeindia; print(financeindia.__version__)"
```

### Troubleshooting

**Error**: `Rust compiler not found`
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustup default 1.88  # Pin to 1.88+
```

**Error**: `maturin not found`
```bash
pip install --upgrade pip setuptools wheel
pip install maturin
```

**Error**: Rebuild not working
```bash
# Clean and rebuild
maturin develop --release  # Use --release for faster runtime
cargo clean
maturin develop
```

---

## Workflow

### 1. Create a Feature Branch

Use descriptive branch names:
- `fix/issue-123-equity-quote-typo`
- `feat/option-chain-cache`
- `docs/architecture-guide`
- `refactor/simplify-csv-parsing`

```bash
git checkout -b fix/your-feature
```

### 2. Make Changes

**Rust Code** (`src/**/*.rs`):
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use meaningful variable names
- Add doc comments for public functions
- Keep functions small and focused

**Python Code** (`financeindia/**/*.py`, `tests/**/*.py`):
- Follow [PEP 8](https://www.python.org/dev/peps/pep-0008/) (use `black` formatter)
- Use type hints
- Meaningful test names

### 3. Update Type Stub (Important!)

If you add/modify Rust functions exposed to Python, **update** `financeindia/financeindia.pyi`:

```python
# financeindia/financeindia.pyi
class FinanceClient:
    def new_method(self, param: str) -> list[NewDataRow]: ...

class NewDataRow:
    symbol: str
    value: float
```

This enables IDE autocomplete and is tested by `pytest tests/test_stub.py`.

---

## Testing

### Run Tests Locally

```bash
# All tests (requires live NSE connectivity)
pytest tests/

# Only offline/stub tests (CI-safe, no network)
pytest tests/test_stub.py

# Specific test
pytest tests/test_client.py::test_market_status

# Verbose output
pytest -v tests/

# With coverage
pytest --cov=tests/
```

### Writing Tests

**Offline tests** (in `tests/test_stub.py`):
- No network required
- Fast (< 100ms)
- Validate type stubs and parsing logic

```python
def test_models_have_expected_fields():
    """Verify PyO3 models match type stubs"""
    assert hasattr(MarketStatus, 'name')
    assert hasattr(MarketStatus, 'status')
```

**Integration tests** (in `tests/test_client.py`):
- Use live NSE APIs
- Validate real data contracts
- Use `@pytest.mark.parametrize` for multiple cases

```python
def test_price_volume_data(client):
    """Fetch and validate historical OHLC data"""
    data = client.price_volume_data("RELIANCE", "01-01-2026", "19-09-2026")
    
    assert len(data) > 0
    assert hasattr(data[0], 'close_price')
    assert isinstance(data[0].close_price, float)
```

### Benchmarking

```bash
# Compare performance before/after changes
python -m timeit -n 100 -r 5 "client.get_equity_list()"
```

---

## Commit Messages

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>: <subject>

<body>

<footer>
```

### Type

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `refactor`: Code refactoring (no behavior change)
- `perf`: Performance improvement
- `test`: Test additions/updates
- `chore`: Build, CI, dependencies
- `style`: Code style (formatting, semicolons)
- `revert`: Revert previous commit

### Subject (max 50 chars)
- Imperative mood ("add" not "added" or "adds")
- Lowercase first letter
- No period at end

### Body (optional)
- Explain *what* and *why*, not *how*
- Wrap at 72 characters
- Reference related issues: `Closes #123`

### Footer (optional)
- Link to issues: `Closes #123`, `Fixes #456`
- Breaking changes: `BREAKING CHANGE: description`

### Examples

```bash
# Good
git commit -m "fix: handle NSE timeout in async_client"
git commit -m "feat: add option chain caching

Reduces latency for repeated calls to get_option_chain()
by caching responses for 5 minutes.

Closes #89"

# ❌ Bad
git commit -m "fixed stuff"
git commit -m "WIP: work in progress"
git commit -m "fixed: resolve issue"  # missing type
```

### Validation

Commits are validated by `commitlint` (see `.husky/commit-msg`). Invalid commits are rejected.

---

## Pull Request Process

### Before Submitting

1. **Rebase on main**:
   ```bash
   git fetch origin main
   git rebase origin/main
   ```

2. **Ensure tests pass**:
   ```bash
   pytest tests/test_stub.py  # Always works
   pytest tests/             # If you have NSE access
   ```

3. **Build succeeds**:
   ```bash
   maturin develop
   cargo test  # Rust unit tests
   ```

### PR Description Template

```markdown
## Description
Brief explanation of changes.

## Type of Change
- [ ] Bug fix (non-breaking)
- [ ] New feature (non-breaking)
- [ ] Breaking change
- [ ] Documentation update

## Related Issues
Closes #123

## How to Test
Steps to verify the fix:
1. Clone branch
2. Run `pytest tests/test_stub.py`
3. Verify: [expected behavior]

## Checklist
- [ ] Tests added/updated
- [ ] Type stub (`financeindia.pyi`) updated
- [ ] Documentation updated (README, ARCHITECTURE.md, etc.)
- [ ] Commit messages follow conventional commits
- [ ] No linting errors (`cargo fmt`, `cargo clippy`)
```

### Review & Merge

- Maintainers will review within 3-5 days
- Address feedback by adding commits (don't rebase)
- Once approved, PR is merged to `main`
- Automatic release-please handles versioning

---

## Reporting Security Issues

**Do not** open public GitHub issues for security vulnerabilities.

Email: `jaiswarsandeep119@gmail.com`

Include:
- Description of vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if available)

We'll acknowledge within 48 hours and work on a fix.

---

## Design Principles

- **Efficiency**: Minimize dependencies, keep binary size small
- **Safety**: Leverage Rust's safety guarantees (no UB, memory safety)
- **Performance**: Prioritize fast parsing, minimal I/O overhead
- **Type Safety**: Use PyO3 models, not dicts/Series
- **Compatibility**: Support Python 3.8+ via ABI3 wheels

---

## Useful Resources

- [financeindia CLAUDE.md](./CLAUDE.md) — Project-specific development notes
- [ARCHITECTURE.md](./ARCHITECTURE.md) — Technical design deep-dive
- [PyO3 Book](https://pyo3.rs/) — Rust ↔ Python FFI
- [Conventional Commits](https://www.conventionalcommits.org/) — Commit format
- [NSE Data Formats](https://www.nseindia.com/) — NSE API structure (reverse-engineered)

---

**Thank you for contributing! 🎉**
