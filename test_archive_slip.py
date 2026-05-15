import financeindia
import os
import zipfile

def test_archive_zip_slip():
    archive = financeindia.BhavArchive()
    dates = ["../../test_date", "valid_date"]
    output_path = "test_slip_archive.zip"

    try:
        # Pass a mock output path
        success_count, failed_dates = archive.archive_equities(dates, output_path)
        print(f"Success: {success_count}, Failed: {failed_dates}")

        if os.path.exists(output_path):
            with zipfile.ZipFile(output_path, 'r') as zip_ref:
                names = zip_ref.namelist()
                print(f"Archive names: {names}")
                for name in names:
                    assert "../" not in name
                    assert "/" not in name
                    assert "\\" not in name
    finally:
        if os.path.exists(output_path):
            os.remove(output_path)

if __name__ == "__main__":
    test_archive_zip_slip()
    print("Archive Zip Slip test complete.")
