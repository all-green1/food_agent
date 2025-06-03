use std::collections::HashMap;
use chrono::{NaiveDate, Local, Duration};
use crate::models::{FoodStock, FoodType, StorageType, Unit};

use dotenvy::dotenv;
use mysql::*;
use mysql::prelude::*;
use std::env;

pub struct DbConfig {
    url: String,
    database: String,
}

impl DbConfig {
    pub fn new(url: &str, database: &str) -> Self {
        Self {
            url: url.to_string(),
            database: database.to_string()
        }
    }
}

pub struct FoodDb {
    pool: Pool,
}

#[derive(Debug)]
pub struct SearchResult {
    pub name: String,
    pub stock_date: String,
    pub food_type: String,
    pub nutrient: String,
    pub storage_type: String,
    pub expiry_date: String,
    pub quantity: String,
}

impl FoodDb {
    pub fn new(config: DbConfig) -> Result<Self, mysql::Error> {
        
        let pool = Pool::new(config.url.as_str())?;
        let mut conn = pool.get_conn()?;
        
        conn.query_drop(
            format!("CREATE DATABASE IF NOT EXISTS {}", config.database)
        )?;

        conn.query_drop(format!("USE {}", config.database))?;

        ///Create a table if it does not currently exist
        conn.exec_drop(
            r"CREATE TABLE IF NOT EXISTS food_stock (
                id INT AUTO_INCREMENT PRIMARY KEY,
                name VARCHAR(50) NOT NULL,
                stock_date DATE NOT NULL,
                food_type VARCHAR(50) NOT NULL,
                nutrient VARCHAR(50) NOT NULL,
                storage_type VARCHAR(50) NOT NULL,
                expiry_date DATE NOT NULL,
                quantity_value FLOAT NOT NULL,
                quantity_unit VARCHAR(2) NOT NULL
                )",
            (),
        )?;

        Ok(Self {pool})

    }
    pub fn add_food(&self, food: FoodStock) -> Result<(), mysql::Error> {
        println!("DEBUG: Starting add_food for: {}", food.name);
        
        let mut conn = self.pool.get_conn()?;
        println!("DEBUG: Got database connection");

        let result = conn.exec_drop(
            r"INSERT INTO food_stock
            (name, stock_date, food_type, nutrient, storage_type, expiry_date, quantity_value, quantity_unit)
            VALUES (:name, :stock_date, :food_type, :nutrient, :storage_type, :expiry_date, :quantity_value, :quantity_unit)",
            params! {
                "name" => &food.name,
                "stock_date" => food.stock_date.format("%Y-%m-%d").to_string(),
                "food_type" => format!("{:?}", food.food_type),
                "nutrient" => format!("{:?}", food.nutrient),
                "storage_type" => format!("{:?}", food.storage_type),
                "expiry_date" => food.expiry_date.format("%Y-%m-%d").to_string(),
                "quantity_value" => match food.quantity {
                    Unit::Grams(g) => g,
                    Unit::Litres(l) => l,
                },
                "quantity_unit" => match food.quantity {
                    Unit::Grams(_) => "g",
                    Unit::Litres(_) => "L",
                }
            }
        );
        
        match result {
            Ok(_) => {
                println!("DEBUG: Insert successful");
                Ok(())
            }
            Err(e) => {
                println!("DEBUG: Insert failed with error: {}", e);
                Err(e)
            }
        }
    }

    pub fn advanced_search(&self, keyword: &str, field: &str) -> Result<Vec<SearchResult>, mysql::Error> {
        println!("DEBUG: advanced_search called with keyword='{}', field='{}'", keyword, field);
        
        let search_pattern = format!("%{}%", &keyword.trim());
        println!("DEBUG: search_pattern='{}'", search_pattern);
        
        let mut conn = self.pool.get_conn()?;
        let allowed_fields = ["name", "food_type", "nutrient", "storage_type"];

        if !allowed_fields.contains(&field) {
            println!("DEBUG: Field '{}' not in allowed fields: {:?}", field, allowed_fields);
            return Err(mysql::Error::DriverError(mysql::DriverError::MissingNamedParameter(field.to_string())))
        }

        let query = format!("SELECT name, stock_date, food_type, nutrient, storage_type, expiry_date, quantity_value, quantity_unit FROM food_stock WHERE {} LIKE :search_string", field);
        println!("DEBUG: executing query: {}", query);
        
        let foods: Vec<SearchResult> = conn.exec_map(
            query,
            params! {
                "search_string" => &search_pattern
            },
            |(name, stock_date, food_type, nutrient, storage_type, expiry_date, quantity_value, quantity_unit): (String, mysql::Value, String, String, String, mysql::Value, f32, String)| {
                let quantity = format!("{}{}", quantity_value, quantity_unit);
                let stock_date_str = match stock_date {
                    mysql::Value::Date(year, month, day, _, _, _, _) => format!("{:04}-{:02}-{:02}", year, month, day),
                    _ => "Unknown".to_string(),
                };
                let expiry_date_str = match expiry_date {
                    mysql::Value::Date(year, month, day, _, _, _, _) => format!("{:04}-{:02}-{:02}", year, month, day),
                    _ => "Unknown".to_string(),
                };
                SearchResult {
                    name,
                    stock_date: stock_date_str,
                    food_type,
                    nutrient,
                    storage_type,
                    expiry_date: expiry_date_str,
                    quantity,
                }
            },
        )?;
        
        println!("DEBUG: advanced_search found {} results", foods.len());
        Ok(foods)
    }

    pub fn get_all_food(&self) -> Result<Vec<SearchResult>, mysql::Error> {
        println!("DEBUG: Starting get_all_food");
        let mut conn = self.pool.get_conn()?;
        println!("DEBUG: Got connection for select");
        
        let result = conn.query_map(
            r"SELECT name, stock_date, food_type, nutrient, storage_type, expiry_date, quantity_value, quantity_unit FROM food_stock",
                |(name, stock_date, food_type, nutrient, storage_type, expiry_date, quantity_value, quantity_unit): (String, String, String, String, String, String, String, String) |{
                let quantity = format!("{}{}", quantity_value, quantity_unit);
                SearchResult {
                    name,
                    stock_date,
                    food_type,
                    nutrient,
                    storage_type,
                    expiry_date,
                    quantity,
                }
            },
        );
        
        match &result {
            Ok(foods) => println!("DEBUG: Found {} food items", foods.len()),
            Err(e) => println!("DEBUG: Select failed: {}", e),
        }
        
        result
    }
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