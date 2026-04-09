use crate::error::FinanceError;
use pyo3::prelude::*;
use reqwest::Client;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use zip::write::FileOptions;

/// Utility to download multiple Bhavcopies concurrently and archive them into a ZIP file.
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

    /// Downloads equity bhavcopies for a list of dates and packs them into a single ZIP file.
    ///
    /// Up to 5 downloads run concurrently. Download failures are non-fatal: failed dates
    /// are collected and returned alongside the success count.
    ///
    /// Returns `(success_count, failed_dates)`.
    fn archive_equities(
        &self,
        py: Python<'_>,
        dates: Vec<String>,
        output_path: String,
    ) -> PyResult<(usize, Vec<String>)> {
        py.allow_threads(|| {
            crate::runtime().block_on(async move {
                let path = Path::new(&output_path);
                let file = File::create(path).map_err(FinanceError::Io)?;
                let mut zip = zip::ZipWriter::new(file);
                // 0o644 — readable data files, not executable.
                let options: FileOptions<'_, ()> = FileOptions::default()
                    .compression_method(zip::CompressionMethod::Stored)
                    .unix_permissions(0o644);

                // Limit concurrent NSE downloads to avoid rate-limiting.
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
                            crate::equities::bhav_copy_equities(&client_clone, &date_clone).await;
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
                            // Sanitize the date string to prevent Zip Slip / path traversal vulnerabilities
                            let safe_date = date.replace('/', "_").replace('\\', "_");
                            zip.start_file(format!("bhav_{}.csv", safe_date), options)
                                .map_err(|e| FinanceError::Runtime(e.to_string()))?;
                            zip.write_all(&data).map_err(FinanceError::Io)?;
                            success_count += 1;
                        }
                        Err(e) => {
                            // Log the failure but continue downloading the remaining dates.
                            log::warn!("Failed to download bhavcopy for {}: {}", date, e);
                            failed_dates.push(date);
                        }
                    }
                }

                zip.finish().map_err(|e| FinanceError::Runtime(e.to_string()))?;

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
