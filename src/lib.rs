use thiserror::Error;
use pyo3::prelude::*;
mod core;
mod metrics;
mod storage;
mod test;

#[derive(Error, Debug)]
pub enum MetricError {
    #[error("DataFusionError: {0}")]
    DataFusionError(#[from] datafusion::error::DataFusionError),
    #[error("Not supported storage backend: {0}")]
    StorageBackendNotSupported(String),
}


#[pymodule]
fn df_metrics(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add("MetricsManager", m.py_class::<metrics::PyMetricsManager>())?;
    m.add("BuiltInMetricsBuilder", m.py_class::<metrics::PyBuiltInMetricsBuilder>())?;
    m.add("Transformation", m.py_class::<metrics::PyTransformation>())?;
    m.add("StorageBackend", m.py_class::<metrics::PyStorageBackend>())?;
    Ok(())
}
