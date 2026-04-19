import pytest
import financeindia
import zipfile
import tempfile
import os

def test_archive_zip_slip():
    archive = financeindia.BhavArchive()
    with tempfile.NamedTemporaryFile(suffix='.zip', delete=False) as f:
        output_path = f.name
    try:
        # Use dates containing path separators to test sanitization
        dates = ["2023/01/01", "2023\\02\\01"]
        success_count, failed_dates = archive.archive_equities(dates, output_path)
        # Open the produced ZIP and inspect entry names
        with zipfile.ZipFile(output_path, 'r') as zf:
            names = zf.namelist()
            for name in names:
                assert '/' not in name
                assert '\\' not in name
                assert '..' not in name
                # Since inputs had '/' and '\', the names should contain '_'
                assert '_' in name
    finally:
        os.unlink(output_path)
