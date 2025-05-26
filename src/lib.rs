use pyo3::prelude::*;
use pyo3::types::PyDict;

mod handlers;
mod storage;
mod models;
mod reminder;

use crate::handlers::CommandHandler;
use crate::storage::FoodStorage;

#[pymodule]
fn food_agent(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyCommandHandler>()?;
    Ok(())
}

#[pyclass]
struct PyCommandHandler {
    handler: CommandHandler,
    storage: FoodStorage,
}

#[pymethods]
impl PyCommandHandler {
    #[new]
    fn new() -> Self {
        Self {
            handler: CommandHandler::new(),
            storage: FoodStorage::new(),
        }
    }

    fn handle_command(
        &mut self, 
        command: String, 
        stock_date: String,
        food_type: String,
        storage_type: String,
        quantity: String,
        expiry_date: String
    ) -> PyResult<String> {
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(self.handler.handle_add(
                &mut self.storage,
                stock_date,
                food_type,
                storage_type,
                quantity,
                Some(expiry_date)
            ));
        
        match result {
            Ok(continue_running) => {
                if !continue_running {
                    Ok("Exiting application".to_string())
                } else {
                    Ok("Command executed successfully".to_string())
                }
            }
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())),
        }
    }

    fn view_all_food(&mut self) -> PyResult<String> {
        match self.handler.handle_view_all(&self.storage) {
            Ok(_) => Ok("Food items displayed successfully".to_string()),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))
        }
    }

    fn search_by_type(&self, food_type: String) -> PyResult<String> {
        let food_type = self.handler.input_handler.get_food_type(&food_type)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;
        let results = self.storage.search_by_type(&food_type);
        Ok(format!("{:?}", results))
    }

    fn search_by_storage(&self, storage_type: String) -> PyResult<String> {
        let storage_type = self.handler.input_handler.get_storage_type(&storage_type)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;
        let results = self.storage.search_by_storage(&storage_type);
        Ok(format!("{:?}", results))
    }

    fn search_by_expiry(&self, date: String) -> PyResult<String> {
        match self.storage.search_by_expiry(&date) {
            Ok(results) => Ok(format!("{:?}", results)),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e)),
        }
    }
} 