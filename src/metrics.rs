
use arrow::array::RecordBatch;
use arrow::pyarrow::FromPyArrow;
use crate::core::computing::execute;
use crate::core::definition::{BuiltInMetricsBuilder, Transformation};
use crate::storage::StorageBackend;
use crate::MetricError;
use pyo3::prelude::*;

/// `MetricsManager` is responsible for managing and executing transformations on data record batches.
/// # Examples
/// ```ignore
/// MetricsManager::default()
///             .transform(BuiltInMetricsBuilder::new().count_null("value", None))
///             .execute(vec![record_batch.unwrap()])
///             .publish(StorageBackend::Stdout)
///             .await
///             .unwrap()
/// ```
#[derive(Debug, Default,Clone)]
struct MetricsManager {
    transformation: Transformation,
    batches: Vec<RecordBatch>,
}
impl MetricsManager {
    pub fn default() -> MetricsManager {
        MetricsManager {
            transformation: Transformation::default(),
            batches: Vec::new(),
        }
    }
    pub fn transform(mut self, transformation: Transformation) -> MetricsManager {
        self.transformation = transformation;
        self
    }

    pub fn execute(mut self, batches: Vec<RecordBatch>) -> MetricsManager {
        self.batches = batches;
        self
    }

    /// Execution the instructions and publishes the results of the transformation to the specified storage backend.
    ///
    /// # Arguments
    ///
    /// * `storage_backend` - The `StorageBackend` where the results will be published.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    ///
    /// # Errors
    ///
    /// This function will return an error if the specified storage backend is not supported.
    pub async fn publish(&self, storage_backend: StorageBackend) -> Result<(), MetricError> {
        let result = execute(self.batches.clone(), &self.transformation)
            .await
            .unwrap();

        match storage_backend {
            StorageBackend::Stdout => {
                for batch in result {
                    //todo: use std::io::stdout instead of print
                    println!("{:?}", batch);
                }
                Ok(())
            }
            _ => Err(MetricError::StorageBackendNotSupported(
                storage_backend.to_string(),
            )),
        }
    }
}

#[pyclass]
pub struct PyMetricsManager {
    inner: MetricsManager,
}

#[pymethods]
impl PyMetricsManager {
    #[new]
    pub fn new() -> Self {
        PyMetricsManager {
            inner: MetricsManager::default(),
        }
    }

    pub fn transform(mut slf: PyRefMut<'_, Self>, transformation: &PyTransformation) ->PyResult<Py<PyMetricsManager>>  {
        slf.inner = MetricsManager {
            transformation: transformation.inner.clone(),
            batches: slf.inner.batches.clone(),
        };
        Ok(slf.into())
    }

    pub fn execute(mut slf: PyRefMut<'_, Self>, py: Python<'_>, py_batches: Vec<PyObject>) -> PyResult<Py<PyMetricsManager>>  {
        let mut batches = Vec::new();
        Python::with_gil(|py| -> PyResult<()> {
            for batch in py_batches {
                //  
                let record_batch = RecordBatch::from_pyarrow_bound(batch)?;
                batches.push(record_batch);
            }
            Ok(())
        })?;

        slf.inner = MetricsManager {
            transformation: slf.inner.transformation.clone(),
            batches,
        };
        Ok(slf.into())
    }

    pub fn publish(&self, storage_backend: PyStorageBackend) -> PyResult<()> {
        Python::with_gil(|py| {
            let inner = self.inner.clone();
            pyo3_asyncio::tokio::future_into_py(py, async move {
                inner.publish(storage_backend.into()).await?;
                Ok(())
            })
        })
    }
}

#[pyclass]
pub struct PyBuiltInMetricsBuilder {
    inner: BuiltInMetricsBuilder,
}

#[pymethods]
impl PyBuiltInMetricsBuilder {
    #[new]
    pub fn new() -> Self {
        PyBuiltInMetricsBuilder {
            inner: BuiltInMetricsBuilder::new(),
        }
    }

    #[pyo3(signature = (column, alias=None))]
    pub fn count_null(&mut self, column: &str, alias: Option<&str>) -> PyResult<PyTransformation> {
        Ok(PyTransformation {
            inner: self.inner.count_null(column, None),
        })
    }

    // Add other methods from BuiltInMetricsBuilder as needed
}

#[pyclass]
#[derive(Clone)] 
pub struct PyTransformation {
    inner: Transformation,
}

#[derive(PartialEq,Clone)]
#[pyclass(eq, eq_int)]
pub enum PyStorageBackend {
    Stdout,
}

impl From<PyStorageBackend> for StorageBackend {
    fn from(backend: PyStorageBackend) -> Self {
        match backend {
            PyStorageBackend::Stdout => StorageBackend::Stdout,
        }
    }
}



#[cfg(test)]
mod test {
    use crate::core::definition::{AggregateType, BuiltInMetricsBuilder, TransformationBuilder};
    use crate::metrics::MetricsManager;
    use crate::storage::StorageBackend;
    use crate::test::generate_dataset;

    #[tokio::test]
    async fn test_metrics_manager() {
        let record_batch = generate_dataset();
        MetricsManager::default()
            .transform(
                TransformationBuilder::new()
                    .select(vec!["id", "value", "category"])
                    .aggregate(AggregateType::Sum, vec!["value"])
                    .group_by(vec!["category"])
                    .build(),
            )
            .execute(vec![record_batch.unwrap()])
            .publish(StorageBackend::Stdout)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_count_null_metrics() {
        let record_batch = generate_dataset();
        MetricsManager::default()
            .transform(BuiltInMetricsBuilder::new().count_null("value", None))
            .execute(vec![record_batch.unwrap()])
            .publish(StorageBackend::Stdout)
            .await
            .unwrap()
    }
}
