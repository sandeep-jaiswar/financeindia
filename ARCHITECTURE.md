# Architecture: financeindia Technical Design

This document describes the architecture and implementation of `financeindia`.

## Table of Contents

1. [Overview](#overview)
2. [Core Components](#core-components)
3. [Data Flow](#data-flow)
4. [Key Design Decisions](#key-design-decisions)
5. [Performance Optimizations](#performance-optimizations)
6. [Extending financeindia](#extending-financeindia)

---

## Overview

`financeindia` is a hybrid Rust + Python library that bridges high-performance data fetching with Python's ease of use.

### Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    Python Application                       │
└────────────────────┬────────────────────────────────────────┘
                     │ (PyO3 bindings)
┌────────────────────▼────────────────────────────────────────┐
│              financeindia (Rust Core)                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │ FinanceClient│  │ Models (PyO3)│  │ Async Runtime    │  │
│  │ (lib.rs)     │  │ (models.rs)  │  │ (Tokio)          │  │
│  └──────┬───────┘  └──────────────┘  └──────────────────┘  │
│         │                                                    │
│  ┌──────▼──────────────────────────────────────────────┐   │
│  │  Async HTTP Client (async_client.rs)               │   │
│  │  • Connection pooling (reqwest)                    │   │
│  │  • Cookie management (Akamai session)             │   │
│  │  • Automatic retries & backoff                    │   │
│  └──────┬───────────────────────────────────────────┬─┘   │
│         │                                            │     │
│  ┌──────▼─────────────┐         ┌──────────────────▼──┐   │
│  │ Domain Modules     │         │ Data Processing    │   │
│  │ • equities.rs      │         │ • CSV parsing      │   │
│  │ • derivatives.rs   │         │ • XML/XBRL parse   │   │
│  │ • indices.rs       │         │ • Zip handling     │   │
│  │ • corporate.rs     │         │ • Type conversion  │   │
│  │ • slb.rs           │         └────────────────────┘   │
│  │ • commodities.rs   │                                   │
│  │ • archive.rs       │                                   │
│  └────────────────────┘                                   │
│         │                                                 │
└─────────┼──────────────────────────────────────────────┬──┘
          │ (HTTP REST)                                   │
          │                                               │
    ┌─────▼──────────────────────────────────────────────▼────┐
    │    NSE/MCX Public APIs (Public Web Pages)               │
    │    • https://www.nseindia.com/                          │
    │    • https://www.mcxindia.com/                          │
    └────────────────────────────────────────────────────────┘
```

---

## Core Components

### 1. FinanceClient (lib.rs)

The main entry point. Provides the public Python API.

```rust
pub struct FinanceClient {
    client: Arc<AsyncClient>,
    runtime: Handle,
}

impl FinanceClient {
    pub fn new() -> Self { ... }
    
    // Public methods exposed to Python via PyO3
    pub fn get_market_status(&self) -> Result<MarketStatusResponse> { ... }
    pub fn price_volume_data(&self, symbol: &str, from: &str, to: &str) -> Result<Vec<PriceVolumeRow>> { ... }
    // ... 42+ more endpoints
}
```

**Key Features**:
- Wraps `AsyncClient` for convenient sync Python API
- Uses `fetch_py!` macro to bridge async Rust → sync Python
- Manages Tokio runtime lifecycle

### 2. AsyncClient (async_client.rs)

Core HTTP client with request/response handling.

```rust
pub struct AsyncClient {
    client: reqwest::Client,
    session: RwLock<SessionState>,
}

pub struct SessionState {
    cookies: Vec<Cookie>,
    last_refresh: Instant,
    base_url: String,
}
```

**Key Features**:
- **Connection Pooling**: Reuses TCP connections across requests
- **Cookie Management**: Warm-up every 15 minutes (Akamai requirement)
- **Automatic Retries**: Exponential backoff for 429/500 errors
- **Concurrent-Safe**: `RwLock` for thread-safe session access

### 3. Models (models.rs)

PyO3 data structures for type-safe Python access.

```rust
#[pyclass]
pub struct MarketStatus {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub status: String,
    #[pyo3(get)]
    pub open_time: String,
}

#[pyclass]
pub struct PriceVolumeRow {
    #[pyo3(get)]
    pub symbol: String,
    #[pyo3(get)]
    pub date: String,
    #[pyo3(get)]
    pub open_price: f64,
    #[pyo3(get)]
    pub close_price: f64,
    // ... more fields
}
```

**Benefits**:
- Type-safe access from Python (IDE autocomplete)
- Zero-copy attribute access
- Validation at construction time

### 4. Domain Modules

Each API domain is in its own file:

| File | Coverage |
|------|----------|
| `equities.rs` | 15+ endpoints (quotes, historical, bulk deals, etc.) |
| `derivatives.rs` | 8+ F&O endpoints (option chains, ban lists, margins) |
| `indices.rs` | 6+ index endpoints (constituents, history, yields) |
| `corporate.rs` | 4+ corporate action endpoints (dividends, financials) |
| `slb.rs` | 4+ securities lending endpoints |
| `commodities.rs` | NSE Commodities + MCX bhavcopy |
| `streaming.rs` | WebSocket for real-time data (beta) |

Each implements:
- **Request building**: Construct NSE-compliant URLs
- **Response parsing**: CSV/JSON/XBRL parsing
- **Error handling**: Map HTTP errors to Rust Result types

### 5. Archive Module (archive.rs)

Handles zip/bhavcopy downloads with security.

```rust
pub fn extract_safe(zip_data: &[u8], target_dir: &Path) -> Result<()> {
    let mut archive = ZipArchive::new(Cursor::new(zip_data))?;
    
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = target_dir.join(file.name());
        
        // Zip Slip prevention: ensure outpath is under target_dir
        if !outpath.starts_with(target_dir) {
            return Err("Path traversal attempt detected".into());
        }
        
        // Extract safely
        // ...
    }
    Ok(())
}
```

**Security**: Prevents directory traversal attacks (CVE-2021-21315).

---

## Data Flow

### Typical Request Lifecycle

1. **Python calls method** (e.g., `client.get_market_status()`)
2. **PyO3 dispatches** to Rust implementation
3. **Async wrapper** (`fetch_py!` macro) spawns on Tokio runtime
4. **AsyncClient** constructs URL and executes HTTP request
5. **Response parsing**:
   - Check HTTP status and session validity
   - Parse CSV/JSON/XML to typed objects
   - Validate data (e.g., date ranges, numeric ranges)
6. **Return to Python** as typed PyO3 object (or Vec of objects)
7. **Python** uses type-safe attribute access

### Example: Fetching Historical Data

```
User:              client.price_volume_data("RELIANCE", "01-01-2026", "19-09-2026")
                          │
                          ▼
PyO3 binding:    Dispatch to Rust
                          │
                          ▼
AsyncClient:     GET /live_analysis/query?key=RELIANCE...
                          │
                          ▼
NSE Server:      Returns CSV (symbol,date,open,high,low,close,volume)
                          │
                          ▼
CSV Parser:      Tokenize CSV rows
                 Validate numeric fields
                 Construct Vec<PriceVolumeRow>
                          │
                          ▼
Type Conversion: Convert to Python list of PyO3 objects
                          │
                          ▼
Python:          [PriceVolumeRow(...), PriceVolumeRow(...), ...]
                 Access: data[0].close_price (type-safe)
```

---

## Key Design Decisions

### 1. Rust + PyO3 (Not Pure Python)

**Why**: Performance via direct parsing (3-5x speedup) without serialization overhead.

**Tradeoff**: Requires Rust build chain. Mitigated via pre-built wheels (cp38-abi3).

### 2. Sync Python API with Async Core

**Why**: Python simplicity + Rust performance. Most users don't need explicit async.

```python
# Simple synchronous API (Python)
data = client.price_volume_data("RELIANCE", "01-01-2026", "19-09-2026")

# Behind the scenes (Rust)
fetch_py!(
    async { /* perform async HTTP request */ }
)
```

**Mechanism**: `fetch_py!` macro spawns blocking task on Tokio runtime.

### 3. Cookie Warm-up Strategy

**Why**: NSE's Akamai firewall resets sessions every 15 minutes. Keeping cookies fresh reduces latency.

```rust
pub async fn ensure_session_fresh(&self) -> Result<()> {
    let session = self.session.read().await;
    if session.last_refresh.elapsed() > Duration::from_secs(15 * 60) {
        drop(session); // Release read lock
        self.refresh_session().await?;
    }
    Ok(())
}
```

**Not a rate limiter**: Still respects NSE's actual rate limits. Application code should pace requests.

### 4. Direct CSV Parsing

**Why**: NSE returns CSV. Direct parsing in Rust avoids:
- Serializing to JSON (NSE doesn't do this)
- Deserializing to Python strings
- Re-parsing strings to objects

Result: 10-50ms overhead removed per request.

```rust
let mut reader = csv::Reader::from_reader(response_bytes);
let mut rows = Vec::new();
for result in reader.deserialize() {
    let record: PriceVolumeRow = result?;
    rows.push(record);
}
Ok(rows)
```

### 5. Type-Safe PyO3 Models

**Why**: Python typically uses dicts/Series (error-prone). PyO3 models:
- Provide IDE autocomplete
- Catch typos at runtime (not silent errors)
- Enable type checking with mypy

```python
# Type-safe (IDE knows fields)
data = client.price_volume_data(...)
price = data[0].close_price  # ✅ IDE suggests this field

# vs. dict-based (error-prone)
price = data[0]["close_price"]  # ❌ Typo? Silent KeyError at runtime
```

---

## Performance Optimizations

### 1. Connection Pooling
```rust
let client = reqwest::Client::builder()
    .pool_max_idle_per_host(10)  // Keep 10 idle connections
    .build()?;
```
Reduces TCP handshake overhead for subsequent requests.

### 2. Async Concurrency
```rust
// Fetch 100 symbols concurrently
let tasks: Vec<_> = symbols.iter()
    .map(|s| tokio::spawn(fetch_quote(s.clone())))
    .collect();
let results = futures::future::join_all(tasks).await;
```
Non-blocking I/O: 100 requests in ~1s instead of ~45s.

### 3. Zero-Copy CSV
Direct parse CSV bytes → PyO3 object without intermediate strings.

### 4. Request Batching
Some endpoints support bulk requests:
```
GET /live_analysis/query?symbol=RELIANCE|INFY|TCS
```
Returns all 3 symbols in one request.

---

## Extending financeindia

### Adding a New NSE Endpoint

1. **Create domain module** (e.g., `src/new_domain.rs`)
   ```rust
   pub async fn get_new_data(
       client: &AsyncClient,
       param1: &str,
   ) -> Result<Vec<NewDataRow>> {
       let url = format!("{}/path/to/endpoint", client.base_url);
       let response = client.get(&url).await?;
       
       // Parse response (CSV, JSON, or HTML)
       let data = parse_response(&response)?;
       Ok(data)
   }
   ```

2. **Add PyO3 models** (in `src/models.rs`)
   ```rust
   #[pyclass]
   pub struct NewDataRow {
       #[pyo3(get)]
       pub field1: String,
       #[pyo3(get)]
       pub field2: f64,
   }
   ```

3. **Expose in FinanceClient** (in `src/lib.rs`)
   ```rust
   #[pymethods]
   impl FinanceClient {
       pub fn get_new_data(&self, param1: &str) -> PyResult<Vec<NewDataRow>> {
           self.fetch_py!(
               async { fetch_new_data(&self.client, param1).await }
           )
       }
   }
   ```

4. **Update stub** (in `financeindia/financeindia.pyi`)
   ```python
   def get_new_data(self, param1: str) -> list[NewDataRow]: ...
   ```

5. **Add tests** (in `tests/test_client.py`)
   ```python
   def test_get_new_data(client):
       data = client.get_new_data("param")
       assert len(data) > 0
       assert hasattr(data[0], 'field1')
   ```

### Performance Profiling

```bash
# Profile Rust code
cargo build --release
time cargo test --release

# Profile Python calls
python -m cProfile -s cumtime script.py
```

---

## Testing Strategy

### Unit Tests (Rust)
```bash
cargo test
```
Tests parsing logic, error handling without network calls.

### Integration Tests (Python)
```bash
pytest tests/test_client.py
```
Live NSE calls; tests real API contracts.

### Offline Tests (CI-safe)
```bash
pytest tests/test_stub.py
```
Validates type stubs against models.rs; no network required.

---

## Roadmap

- [ ] WebSocket real-time data (in progress)
- [ ] Async Python API (`async def`)
- [ ] Better error types (custom exceptions)
- [ ] HTTP/2 support
- [ ] Caching layer (in-process or Redis)

---

## References

- [PyO3 Documentation](https://pyo3.rs/)
- [Tokio Runtime](https://tokio.rs/)
- [reqwest HTTP Client](https://github.com/seanmonstar/reqwest)
- [NSE Data Formats](https://www.nseindia.com/) (external, reverse-engineered)

---

**Last Updated**: 2026-09-19  
**financeindia Version**: 0.2.3
