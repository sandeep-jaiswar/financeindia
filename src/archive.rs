use crate::error::FinanceError;
use pyo3::prelude::*;
use reqwest::Client;
use std::env;
use std::fs::{File, symlink_metadata};
use std::io::Write;
use std::path::{Component, Path};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use zip::write::FileOptions;

#[pyclass]
pub struct BhavArchive {
    client: Client,
}

#[pymethods]
impl BhavArchive {
    #[new]
    pub fn new() -> PyResult<Self> {
        let client = crate::common::build_client(None).map_err(PyErr::from)?;
        Ok(BhavArchive { client })
    }

    fn archive_equities(
        &self,
        py: Python<'_>,
        dates: Vec<String>,
        output_path: String,
    ) -> PyResult<(usize, Vec<String>)> {
        let path = Path::new(&output_path);
        for component in path.components() {
            match component {
                Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                    return Err(pyo3::exceptions::PyValueError::new_err(
                        "Invalid output path: Path traversal or absolute paths are not allowed.",
                    ));
                }
                _ => {}
            }
        }

        py.allow_threads(|| {
            crate::runtime()
                .block_on(async move {
                    let path = Path::new(&output_path);
                    if path.exists() {
                        let metadata = symlink_metadata(path).map_err(FinanceError::Io)?;
                        if metadata.is_symlink() {
                            return Err(FinanceError::PyErr(
                                pyo3::exceptions::PyValueError::new_err(
                                    "Invalid output path: Symlinks are not allowed.",
                                ),
                            ));
                        }
                    }
                    let base = env::current_dir().map_err(FinanceError::Io)?;
                    if let Some(parent) = path.parent() {
                        let canonical_parent = parent.canonicalize().map_err(FinanceError::Io)?;
                        if !canonical_parent.starts_with(&base) {
                            return Err(FinanceError::PyErr(
                                pyo3::exceptions::PyValueError::new_err(
                                    "Invalid output path: Path resolves outside allowed directory.",
                                ),
                            ));
                        }
                    }

                    let file = File::create(path).map_err(FinanceError::Io)?;
                    let mut zip = zip::ZipWriter::new(file);
                    let options: FileOptions<'_, ()> = FileOptions::default()
                        .compression_method(zip::CompressionMethod::Stored)
                        .unix_permissions(0o644);

                    let semaphore = Arc::new(Semaphore::new(5));
                    let mut set: JoinSet<(String, crate::error::FinanceResult<bytes::Bytes>)> =
                        JoinSet::new();
                    let client = self.client.clone();

                    for date in dates {
                        let sem_clone = semaphore.clone();
                        let client_clone = client.clone();
                        let date_clone = date.clone();
                        set.spawn(async move {
                            let _permit = sem_clone
                                .acquire()
                                .await
                                .expect("BhavArchive semaphore should never close");
                            let res =
                                crate::equities::bhav_copy_equities(&client_clone, &date_clone)
                                    .await;
                            (date_clone, res)
                        });
                    }

                    let mut success_count = 0;
                    let mut failed_dates = Vec::new();

                    while let Some(res) = set.join_next().await {
                        let (date, result) =
                            res.map_err(|e| FinanceError::Runtime(e.to_string()))?;
                        match result {
                            Ok(data) => {
                                let clean_date = sanitize_date_for_archive(&date);
                                zip.start_file(format!("bhav_{}.csv", clean_date), options)
                                    .map_err(|e| FinanceError::Runtime(e.to_string()))?;
                                zip.write_all(&data).map_err(FinanceError::Io)?;
                                success_count += 1;
                            }
                            Err(e) => {
                                log::warn!("Failed to download bhavcopy for {}: {}", date, e);
                                failed_dates.push(date);
                            }
                        }
                    }

                    zip.finish()
                        .map_err(|e| FinanceError::Runtime(e.to_string()))?;

                    if success_count == 0 {
                        let _ = std::fs::remove_file(path).map_err(|e| {
                            log::error!("Failed to remove empty archive at {}: {}", output_path, e);
                        });
                    }

                    Ok::<_, FinanceError>((success_count, failed_dates))
                })
                .map_err(PyErr::from)
        })
    }
}

fn sanitize_date_for_archive(date: &str) -> String {
    date.replace('/', "_").replace('\\', "_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_date_for_archive_forward_slash() {
        assert_eq!(sanitize_date_for_archive("2023/01/01"), "2023_01_01");
    }

    #[test]
    fn test_sanitize_date_for_archive_backslash() {
        assert_eq!(sanitize_date_for_archive("2023\\02\\01"), "2023_02_01");
    }

    #[test]
    fn test_sanitize_date_for_archive_mixed() {
        assert_eq!(sanitize_date_for_archive("2023/01\\02"), "2023_01_02");
    }

    #[test]
    fn test_sanitize_date_for_archive_no_separators() {
        assert_eq!(sanitize_date_for_archive("2023-01-01"), "2023-01-01");
    }
}