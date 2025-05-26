use std::collections::HashMap;
use chrono::{NaiveDate, Local, Duration};
use crate::models::{FoodStock, FoodType, StorageType};

/// Manages the storage and retrieval of food stock data
pub struct FoodStorage {
    foods: HashMap<u32, FoodStock>,
    next_id: u32,
}

impl FoodStorage {
    /// Creates a new FoodStorage instance
    pub fn new() -> Self {
        Self {
            foods: HashMap::new(),
            next_id: 1,
        }
    }

    /// Gets the next available index
    pub fn next_index(&self) -> u32 {
        self.next_id
    }

    /// Adds a food stock to storage
    pub fn add_food(&mut self, food: FoodStock) {
        self.foods.insert(food.index, food);
        self.next_id += 1;
    }

    /// Gets all food stocks
    pub fn get_all_food(&self) -> Vec<&FoodStock> {
        self.foods.values().collect()
    }

    /// Searches for food stocks by type
    pub fn search_by_type(&self, food_type: &FoodType) -> Vec<&FoodStock> {
        self.foods.values()
            .filter(|food| std::mem::discriminant(&food.food_type) == std::mem::discriminant(food_type))
            .collect()
    }

    /// Searches for food stocks by storage type
    pub fn search_by_storage(&self, storage_type: &StorageType) -> Vec<&FoodStock> {
        self.foods.values()
            .filter(|food| std::mem::discriminant(&food.storage_type) == std::mem::discriminant(storage_type))
            .collect()
    }

    /// Parses a date query string into a NaiveDate
    /// 
    /// # Arguments
    /// 
    /// * `query` - A string that can be "today", "tomorrow", "yesterday", or a date in "DD-MM-YYYY" format
    /// 
    /// # Returns
    /// 
    /// * `Result<NaiveDate, String>` - The parsed date or an error message
    pub fn parse_query_to_date(query: &str) -> Result<NaiveDate, String> {
        let query = query.trim().to_lowercase();
        match query.as_str() {
            "today" => Ok(Local::now().naive_local().date()),
            "tomorrow" => Ok(Local::now().naive_local().date() + Duration::days(1)),
            "yesterday" => Ok(Local::now().naive_local().date() - Duration::days(1)),
            _ => NaiveDate::parse_from_str(&query, "%d-%m-%Y")
                .map_err(|_| "Invalid date format. Use 'today', 'tomorrow', 'yesterday', or DD-MM-YYYY".to_string())
        }
    }

    /// Searches for food stocks by expiry date
    /// 
    /// # Arguments
    /// 
    /// * `query` - A string that can be "today", "tomorrow", "yesterday", or a date in "DD-MM-YYYY" format
    /// 
    /// # Returns
    /// 
    /// * `Result<Vec<&FoodStock>, String>` - A vector of matching food stocks or an error message
    pub fn search_by_expiry(&self, query: &str) -> Result<Vec<&FoodStock>, String> {
        let date = Self::parse_query_to_date(query)?;
        Ok(self.foods.values()
            .filter(|food| food.expiry_date == date)
            .collect())
    }

    /// Removes a food stock by index
    pub fn remove_food(&mut self, index: u32) -> Option<FoodStock> {
        self.foods.remove(&index)
    }

    /// Updates a food stock
    pub fn update_food(&mut self, food: FoodStock) -> Option<FoodStock> {
        self.foods.insert(food.index, food)
    }
} 