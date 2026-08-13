"""Validates that `financeindia/financeindia.pyi` is the shipped, accurate
source of truth for the idiomatic Python API.

The Rust model types and the stub must not silently drift apart (the stub is
what IDEs, mypy and pyright show users). This test is offline - it only reads
the stub file bundled with the package.
"""

import re
from pathlib import Path

import financeindia

STUB_PATH = Path(financeindia.__file__).parent / "financeindia.pyi"

# (class, attr) -> annotation that must appear in the stub. Keep in sync with
# src/models.rs whenever you change a Rust model field's type.
EXPECTED_FIELDS = {
    "Holiday": {
        "sr_no": "Optional[int]",
    },
    "PriceVolumeRow": {
        "prev_close": "Optional[float]",
        "open_price": "Optional[float]",
        "high_price": "Optional[float]",
        "low_price": "Optional[float]",
        "last_price": "Optional[float]",
        "close_price": "Optional[float]",
        "average_price": "Optional[float]",
        "total_traded_quantity": "Optional[int]",
        "turnover": "Optional[float]",
        "no_of_trades": "Optional[int]",
    },
    "EquityInfo": {
        "paid_up_value": "Optional[float]",
        "market_lot": "Optional[str]",
        "face_value": "Optional[float]",
    },
    "GSMStock": {
        "stage": "Optional[int]",
    },
}


def _class_block(stub: str, class_name: str) -> str:
    match = re.search(
        rf"^class {re.escape(class_name)}:\n(?:.*\n)*?(?=^class |\Z)",
        stub,
        re.MULTILINE,
    )
    assert match, f"class {class_name} is missing from the stub"
    return match.group(0)


def test_stub_ships_inside_package():
    # maturin only bundles `.pyi` files that live inside `python-packages`.
    assert STUB_PATH.exists(), "financeindia.pyi must live inside the financeindia package"


def test_stub_annotations_match_rust_models():
    stub = STUB_PATH.read_text(encoding="utf-8")
    for class_name, fields in EXPECTED_FIELDS.items():
        block = _class_block(stub, class_name)
        for attr, annotation in fields.items():
            assert re.search(
                rf"^\s+{re.escape(attr)}:\s*{re.escape(annotation)}\s*$",
                block,
                re.MULTILINE,
            ), f"{class_name}.{attr} should be annotated as {annotation} in the stub"


def test_stub_exports_error_classes():
    stub = STUB_PATH.read_text(encoding="utf-8")
    exception_classes = [
        "FinanceException",
        "HTTPError",
        "ConnectionError",
        "TimeoutError",
        "StatusCodeError",
        "RateLimitError",
        "DataError",
        "JSONParseError",
        "CSVParseError",
        "XMLParseError",
        "ValidationError",
        "NetworkError",
        "UnknownError",
    ]
    # Verify stub coverage
    for cls in exception_classes:
        assert f"class {cls}" in stub, f"stub is missing exception class {cls}"
    assert "FinanceError = FinanceException" in stub

    # Verify runtime module exposes all exception classes
    for cls in exception_classes:
        assert hasattr(financeindia, cls), f"runtime module is missing exception class {cls}"
        assert isinstance(getattr(financeindia, cls), type), f"{cls} should be a class"

    # Verify FinanceError is the identical object as FinanceException
    assert hasattr(financeindia, "FinanceError"), "runtime module is missing FinanceError alias"
    assert financeindia.FinanceError is financeindia.FinanceException, \
        "FinanceError should be the same object as FinanceException"
