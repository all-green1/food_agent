#![allow(warnings)]
use std::io;
use chrono::{NaiveDate, Local, Duration};
use crate::models::{FoodType, Unit, StorageType, MajorNutrient, FoodStock};
use crate::storage::FoodDb;
use crate::reminder::create_calendar_event;

/// Handles user input operations
pub struct InputHandler;

impl InputHandler {
    pub fn new() -> Self {
        Self
    }
    /// Gets a date input from the user
    pub fn get_date(&self, input: &str) -> Result<NaiveDate, String> {
        match input {
            "today" => Ok(Local::now().naive_local().date()),
            "yesterday" => Ok(Local::now().naive_local().date().pred_opt()
                .expect("Failed to get yesterday's date")),
            _ => NaiveDate::parse_from_str(input, "%d-%m-%Y")
                .map_err(|_| "Please enter a valid date in DD-MM-YYYY format.".to_string())
        }
    }
    /// Gets a food type from the user
    pub fn get_food_type(&self, input: &str) -> Result<FoodType, String> {
        match input {
            "vegetable" => Ok(FoodType::Vegetable),
            "fruit" => Ok(FoodType::Fruit),
            "grains" => Ok(FoodType::Grains),
            "breakfast-cereal" => Ok(FoodType::Breakfast_cereal),
            "beverage" => Ok(FoodType::Beverage),
            "meat" => Ok(FoodType::Meat),
            "dairy" => Ok(FoodType::Dairy),
            "non-dairy" => Ok(FoodType::Non_dairy),
            "edible-oils" => Ok(FoodType::Edible_oils),
            _ => Err("Invalid food type".to_string()),
        }
    }
    pub fn get_food_name(&self, input: &str) -> Result<String, String> {
        if input.trim().is_empty() {
            println!("Invalid input");
            Err("Input was empty".to_string())
        } else {
            Ok(input.to_string())
        }
    }
    

    /// Gets a storage type from the user
    pub fn get_storage_type(&self, input: &str) -> Result<StorageType, String> {
        match input {
            "cold" => Ok(StorageType::Cold),
            "warm" => Ok(StorageType::RoomTemperature),
            _ => Err("Invalid storage type".to_string()),
        }
    }

    /// Gets a quantity from the user
    pub fn get_quantity(&self, input: &str) -> Result<Unit, String> {

        let last_char = input.chars().last().ok_or("Invalid quantity format")?.to_ascii_lowercase();
        let number_part = &input[..input.len().saturating_sub(1)].trim();

        match last_char {
            'g' => number_part.parse::<f32>()
                .map(Unit::Grams)
                .map_err(|_| "Invalid number format".to_string()),
            'l' => number_part.parse::<f32>()
                .map(Unit::Litres)
                .map_err(|_| "Invalid number format".to_string()),
            _ => Err("Invalid unit. Use 'g' for grams or 'l' for litres".to_string()),
        }
    }
}

/// Handles command processing
pub struct CommandHandler {
    pub input_handler: InputHandler,
}

impl CommandHandler {
    pub fn new() -> Self {
        Self {
            input_handler: InputHandler::new(),
        }
    }

    /// Handles the add command
    pub async fn handle_add(
        &self, 
        name: String,
        storage: &mut FoodDb,
        stock_date: String,
        food_type: String,
        storage_type: String,
        quantity: String,
        expiry_date: Option<String>
    ) -> Result<bool, String> {
        println!("DEBUG: handle_add called with name: {}", name);
        println!("\nAdding new food stock...");
        
        let name = self.input_handler.get_food_name(&name)?;
        let stock_date = self.input_handler.get_date(&stock_date)?;
        let food_type = self.input_handler.get_food_type(&food_type)?;
        let storage_type = self.input_handler.get_storage_type(&storage_type)?;
        let quantity = self.input_handler.get_quantity(&quantity)?;

        let nutrient = match &food_type {
            FoodType::Vegetable => MajorNutrient::Minerals_and_vitamins,
            FoodType::Dairy => MajorNutrient::Balanced,
            FoodType::Non_dairy => MajorNutrient::Carbohydrate,
            FoodType::Fruit => MajorNutrient::Carbohydrate,
            FoodType::Meat => MajorNutrient::Protein,
            FoodType::Breakfast_cereal => MajorNutrient::Carbohydrate,
            FoodType::Grains => MajorNutrient::Carbohydrate,
            FoodType::Beverage => MajorNutrient::Sugars,
            FoodType::Edible_oils => MajorNutrient::Fat,
        };

        let expiry_date = match expiry_date {
            Some(date_str) if date_str.to_lowercase() == "none" => {
                // Calculate expiry date based on food type and storage type
                let days_to_add = match (&food_type, &storage_type) {
                    (FoodType::Meat, StorageType::Cold) => 7,
                    (FoodType::Meat, StorageType::RoomTemperature) => 2,
                    (FoodType::Vegetable, StorageType::Cold) => 5,
                    (FoodType::Vegetable, StorageType::RoomTemperature) => 2,
                    (FoodType::Grains, _) => 90,
                    (FoodType::Dairy, StorageType::Cold) => 10,
                    (FoodType::Dairy, StorageType::RoomTemperature) => 1,
                    (FoodType::Non_dairy, StorageType::Cold) => 10,
                    (FoodType::Non_dairy, StorageType::RoomTemperature) => 2,
                    (FoodType::Fruit, StorageType::Cold) => 7,
                    (FoodType::Fruit, StorageType::RoomTemperature) => 3,
                    (_, StorageType::Cold) => 5,
                    (_, StorageType::RoomTemperature) => 2,
                };
                stock_date + Duration::days(days_to_add)
            },
            Some(date_str) => {
                // Use the provided date
                self.input_handler.get_date(&date_str)?
            },
            None => {
                // Calculate expiry date based on food type and storage type
                let days_to_add = match (&food_type, &storage_type) {
                    (FoodType::Meat, StorageType::Cold) => 7,
                    (FoodType::Meat, StorageType::RoomTemperature) => 2,
                    (FoodType::Vegetable, StorageType::Cold) => 5,
                    (FoodType::Vegetable, StorageType::RoomTemperature) => 2,
                    (FoodType::Grains, _) => 90,
                    (FoodType::Dairy, StorageType::Cold) => 10,
                    (FoodType::Dairy, StorageType::RoomTemperature) => 1,
                    (FoodType::Non_dairy, StorageType::Cold) => 10,
                    (FoodType::Non_dairy, StorageType::RoomTemperature) => 2,
                    (FoodType::Fruit, StorageType::Cold) => 7,
                    (FoodType::Fruit, StorageType::RoomTemperature) => 3,
                    (_, StorageType::Cold) => 5,
                    (_, StorageType::RoomTemperature) => 2,
                };
                stock_date + Duration::days(days_to_add)
            }
        };

        let food_stock = FoodStock::new(
            name,
            stock_date,
            food_type,
            nutrient,
            storage_type,
            expiry_date,
            quantity,
        );

        if let Err(e) = storage.add_food(food_stock.clone()) {
            return Err(e.to_string());
        }
        
        if let Err(e) = create_calendar_event(&food_stock).await {
            eprintln!("Failed to create calendar event: {}", e);
        }

        println!("Food stock added successfully!");
        Ok(true)
    }

    /// Handles the view all command
    pub fn handle_view_all(&self, storage: &FoodDb) -> Result<bool, String> {
        println!("DEBUG: handle_view_all called");
        println!("\nCurrent food stocks:");
        match storage.get_all_food() {
            Ok(foods) => {
                for food in foods {
                    println!("{:?}", food);
                }
                Ok(true)
            }
            Err(e) => {
                eprintln!("Error retrieving food: {}", e);
                return Err(e.to_string());
            }
        }
    }

    /// Handles the search command
    // pub fn handle_search(&self, storage: &FoodStorage) -> Result<bool, String> {
    //     println!("\nSearch by:");
    //     println!("1. Food type");
    //     println!("2. Storage type");
    //     println!("3. Expiry date");
        
    //     let mut input = String::new();
    //     io::stdin().read_line(&mut input).map_err(|e| e.to_string())?;
        
    //     match input.trim() {
    //         "1" => {
    //             let food_type = self.input_handler.get_food_type(&input)?;
    //             let results = storage.search_by_type(&food_type);
    //             self.display_search_results(&results);
    //         }
    //         "2" => {
    //             let storage_type = self.input_handler.get_storage_type(&input)?;
    //             let results = storage.search_by_storage(&storage_type);
    //             self.display_search_results(&results);
    //         }
    //         "3" => {
    //             println!("Enter date to search (today/tomorrow/yesterday/DD-MM-YYYY)");
    //             let mut date_input = String::new();
    //             io::stdin().read_line(&mut date_input).map_err(|e| e.to_string())?;
    //             match storage.search_by_expiry(date_input.trim()) {
    //                 Ok(results) => self.display_search_results(&results),
    //                 Err(e) => println!("Error: {}", e),
    //             }
    //         }
    //         _ => return Err("Invalid search option".to_string()),
    //     }
        
    //     Ok(true)
    // }

    /// Displays search results
    fn display_search_results(&self, results: &Vec<&FoodStock>) {
        if results.is_empty() {
            println!("No matching food stocks found.");
            return;
        }

        println!("\nSearch Results:");
        for stock in results {
            println!("{}", stock);
        }
    }
}