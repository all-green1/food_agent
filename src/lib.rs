use pyo3::prelude::*;
use serde_json;
use std::env;

mod handlers;
mod models;
mod storage;
mod reminder;



use crate::handlers::{CommandHandler};
use crate::storage::{DbConfig, FoodDb};

#[pymodule]
fn food_agent(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyCommandHandler>()?;
    Ok(())
}

#[pyclass]
struct PyCommandHandler {
    handler: CommandHandler,
    storage: FoodDb,
}

#[pymethods]
impl PyCommandHandler {
    #[new]
    fn new() -> Self {
        let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
        let confg = DbConfig::new(&db_url, "food_registry");
        
        Self {
            handler: CommandHandler::new(),
            storage: FoodDb::new(confg).expect("Failed to initialize Db"),
        }
    }

    /// Add food to storage
    pub fn add_food(
        &mut self,
        name: String,
        stock_date: String,
        food_type: String,
        storage_type: String,
        quantity: String,
        expiry_date: Option<String>,
        user_id: Option<i32>,
        google_token_json: Option<String>,
    ) -> PyResult<String> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        // Parse Google token JSON if provided
        let google_token = if let Some(token_str) = google_token_json {
            match serde_json::from_str(&token_str) {
                Ok(token) => Some(token),
                Err(_) => {
                    println!("DEBUG: Failed to parse Google token JSON");
                    None
                }
            }
        } else {
            None
        };
        
        match rt.block_on(self.handler.handle_add(
            name,
            &mut self.storage,
            stock_date,
            food_type,
            storage_type,
            quantity,
            expiry_date,
            user_id,
            google_token,
        )) {
            Ok(_) => Ok("Food stock added successfully!".to_string()),
            Err(e) => Ok(format!("Error: {}", e)),
        }
    }

    /// View all food in storage
    fn view_all_food(&self) -> PyResult<String> {
        println!("DEBUG: view_all_food called");
        match self.storage.get_all_food() {
            Ok(foods) => {
                if foods.is_empty() {
                    Ok("No food items found in storage.".to_string())
                } else {
                    let mut result = String::new();
                    for food in foods {
                        result.push_str(&format!("{}\n", food));
                    }
                    Ok(result.trim().to_string())
                }
            }
            Err(e) => Ok(format!("Error retrieving food: {}", e)),
        }
    }

    /// Search storage by keyword and field
    fn search_storage(&self, keyword: &str, field: &str) -> PyResult<String> {
        println!("DEBUG: search_storage called");
        match self.storage.advanced_search(keyword, field) {
            Ok(foods) => {
                if foods.is_empty() {
                    Ok(format!("There is no {} available in storage", keyword))
                } else {
                    let mut result = "These are the search results:\n".to_string();
                    for food in foods {
                        result.push_str(&format!(
                            "- {} ({}): {} stored {}, expires {}\n",
                            food.name,
                            food.food_type,
                            food.quantity,
                            food.storage_type.to_lowercase(),
                            food.expiry_date
                        ));
                    }
                    Ok(result)
                }
            }
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        }
    }

    // fn search_by_type(&self, food_type: String) -> PyResult<String> {
    //     let food_type = self.handler.input_handler.get_food_type(&food_type)
    //         .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;
    //     let results = self.storage.search_by_type(&food_type);
    //     Ok(format!("{:?}", results))
    // }

    // fn search_by_storage(&self, storage_type: String) -> PyResult<String> {
    //     let storage_type = self.handler.input_handler.get_storage_type(&storage_type)
    //         .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))?;
    //     let results = self.storage.search_by_storage(&storage_type);
    //     Ok(format!("{:?}", results))
    // }

    // fn search_by_expiry(&self, date: String) -> PyResult<String> {
    //     match self.storage.search_by_expiry(&date) {
    //         Ok(results) => Ok(format!("{:?}", results)),
    //         Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e)),
    //     }
    // }
} 