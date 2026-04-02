import pytest
import financeindia
import zipfile
import os
import shutil

def test_archive_zip_slip():
    archive = financeindia.BhavArchive()
    assert archive is not None
