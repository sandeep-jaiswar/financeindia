use crate::error::FinanceError;
use pyo3::prelude::*;
use reqwest::Client;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use zip::write::FileOptions;

/// Utility to download multiple Bhavcopies and archive them into a single ZIP file.
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

    /// Downloads equity bhavcopies for a list of dates and packs them into a ZIP.
    /// Returns a tuple of (success_count, failed_dates).
    fn archive_equities(
        &self,
        py: Python<'_>,
        dates: Vec<String>,
        output_path: String,
    ) -> PyResult<(usize, Vec<String>)> {
        use std::sync::Arc;
        use tokio::sync::Semaphore;
        use tokio::task::JoinSet;

        py.allow_threads(|| {
            crate::runtime().block_on(async move {
                let path = Path::new(&output_path);
                let file = File::create(path).map_err(|e| FinanceError::Io(e))?;
                let mut zip = zip::ZipWriter::new(file);
                let options: FileOptions<'_, ()> = FileOptions::default()
                    .compression_method(zip::CompressionMethod::Stored)
                    .unix_permissions(0o755);

                let semaphore = Arc::new(Semaphore::new(5)); // Limit to 5 concurrent downloads
                let mut set = JoinSet::new();
                let client = self.client.clone();

                for date in dates {
                    let sem_clone = semaphore.clone();
                    let client_clone = client.clone();
                    let date_clone = date.clone();
                    set.spawn(async move {
                        let _permit = sem_clone.acquire().await.unwrap();
                        let res =
                            crate::equities::bhav_copy_equities(&client_clone, &date_clone).await;
                        (date_clone, res)
                    });
                }

                let mut success_count = 0;
                let mut failed_dates = Vec::new();

                while let Some(res) = set.join_next().await {
                    let (date, result) = res.map_err(|e| FinanceError::Runtime(e.to_string()))?;
                    match result {
                        Ok(data) => {
                            zip.start_file(format!("bhav_{}.csv", date), options)
                                .map_err(|e| FinanceError::Runtime(e.to_string()))?;
                            zip.write_all(&data).map_err(|e| FinanceError::Io(e))?;
                            success_count += 1;
                        }
                        Err(e) => {
                            eprintln!("Failed to download bhavcopy for {}: {}", date, e);
                            failed_dates.push(date);
                        }
                    }
                }

                zip.finish().map_err(|e| FinanceError::Runtime(e.to_string()))?;
                Ok::<_, FinanceError>((success_count, failed_dates))
            })
            .map_err(PyErr::from)
        })
    }
}
