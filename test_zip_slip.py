import financeindia
import zipfile
import tempfile
from pathlib import Path

def test_archive_zip_slip():
    """
    Exercises the sanitization in BhavArchive.archive_equities:
    Checks that path separators in dates/names are replaced by '_'
    and that no traversal sequences (..) are present in ZIP entry names.
    """
    archive = financeindia.BhavArchive()
    
    # Use TemporaryDirectory to avoid manual cleanup and keep the imports clean
    with tempfile.TemporaryDirectory() as tmpdir:
        output_path = Path(tmpdir) / "test_archive.zip"
        
        # Use dates containing path separators to test sanitization
        # Now supporting both / and \ due to parse_date_robust fix
        dates = ["2023/01/01", "2023\\02\\01"]
        
        success_count, failed_dates = archive.archive_equities(dates, str(output_path))
        
        # Open the produced ZIP and inspect entry names
        with zipfile.ZipFile(output_path, 'r') as zf:
            names = zf.namelist()
            
            # Ensure we actually have entries to check
            assert len(names) > 0, "ZIP archive should contain CSV entries"
            
            for name in names:
                # Assert names are sanitized
                assert '/' not in name, f"Path separator '/' found in entry name: {name}"
                assert '\\' not in name, f"Path separator '\\' found in entry name: {name}"
                assert '..' not in name, f"Traversal sequence '..' found in entry name: {name}"
                
                # Since inputs had separators, the output names should contain '_' 
                # (sanitized from the original input)
                assert '_' in name, f"Expected underscore in sanitized name: {name}"
