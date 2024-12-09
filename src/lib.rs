use pyo3::{prelude::*, types::PyType};

// A trait which can be implemented in Rust or by Python objects with the right signature.
trait Quacker {
    fn quack(&self);
}

// A quacker which uses either a Rust or Python impl.
#[pyclass]
struct AnyQuacker {
    quacker: Box<dyn Quacker + Sync + Send>,
}

#[pymethods]
impl AnyQuacker {
    // This is fine.
    #[classmethod]
    fn rust_quacker(_cls: &Bound<'_, PyType>) -> Self {
        Self {
            quacker: Box::new(RustDuck {
                name: "Rust".to_string(),
            }),
        }
    }

    // This compiled fine in PyO3 0.21, but pyclasses can no longer be passed without
    // being wrapped in something like Py or Bound.
    //
    // Error: the trait `Clone` is not implemented for `PyDuck`, which is required by `PyDuck: PyFunctionArgument<'_, '_>`
    #[classmethod]
    fn python_quacker(_cls: &Bound<'_, PyType>, obj: PyDuck) -> Self {
        Self {
            quacker: Box::new(obj),
        }
    }

    // The signature is now correct for PyO3 0.22, and the function only accepts
    // PyDuck objects, but we can't store Py<PyDuck> in our Box because it doesn't
    // implement Quacker..
    //
    // Error: the trait `Quacker` is not implemented for `pyo3::Py<PyDuck>`
    #[classmethod]
    fn python_quacker_0_22(_cls: &Bound<'_, PyType>, obj: Py<PyDuck>) -> Self {
        Self {
            quacker: Box::new(obj),
        }
    }

    // This works. It does mean knowing something about the constructor of a PyDuck
    // though, and won't fail until we try to call the quack method.
    //
    // We don't need our PyDuck to be a pyclass any more, maybe that's good?
    #[classmethod]
    fn python_quacker_0_22_working(_cls: &Bound<'_, PyType>, obj: Py<PyAny>) -> Self {
        Self {
            quacker: Box::new(PyDuck::new(obj)),
        }
    }
}

struct RustDuck {
    name: String,
}

impl Quacker for RustDuck {
    fn quack(&self) {
        println!("Quack, {}!", self.name);
    }
}

#[pyclass]
struct PyDuck {
    inner: Py<PyAny>,
}

#[pymethods]
impl PyDuck {
    #[new]
    fn new(inner: Py<PyAny>) -> Self {
        Self { inner }
    }
}

impl Quacker for PyDuck {
    fn quack(&self) {
        Python::with_gil(|py| {
            self.inner.call_method0(py, "quack").unwrap();
        })
    }
}
