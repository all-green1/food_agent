use chrono::{NaiveDate, Duration};
use std::fmt;


#[derive(Debug, Clone, PartialEq)]
pub enum FoodType {
    Vegetable,
    Fruit,
    Beverage,
    Grains,
    Breakfast_cereal,
    Meat,
    Dairy,
    Non_dairy,
    Edible_oils,
}

impl fmt::Display for FoodType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FoodType::Vegetable => write!(f, "Vegetable"),
            FoodType::Fruit => write!(f, "Fruit"),
            FoodType::Beverage => write!(f, "Beverage"),
            FoodType::Grains => write!(f, "Grains"),
            FoodType::Breakfast_cereal => write!(f, "Breakfast Cereal"),
            FoodType::Meat => write!(f, "Meat"),
            FoodType::Dairy => write!(f, "Dairy"),
            FoodType::Non_dairy => write!(f, "Non-dairy"),
            FoodType::Edible_oils => write!(f, "Edible Oils"),
        }
    }
}

/// Represents the unit of measurement for food quantity
#[derive(Debug, Clone, PartialEq)]
pub enum Unit {
    Grams(f32),
    Litres(f32),
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Unit::Grams(g) => write!(f, "{}g", g),
            Unit::Litres(l) => write!(f, "{}L", l),
        }
    }
}

/// Represents the storage conditions for food items
#[derive(Debug, Clone, PartialEq)]
pub enum StorageType {
    Cold,
    RoomTemperature,
}

impl fmt::Display for StorageType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StorageType::Cold => write!(f, "Cold Storage"),
            StorageType::RoomTemperature => write!(f, "Room Temperature"),
        }
    }
}

/// Represents the major nutrient content of food items
#[derive(Debug, Clone, PartialEq)]
pub enum MajorNutrient {
    Protein,
    Fat,
    Carbohydrate,
    Sugars,
    Water,
    Minerals_and_vitamins,
    Balanced,
}

impl fmt::Display for MajorNutrient {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MajorNutrient::Protein => write!(f, "Protein"),
            MajorNutrient::Fat => write!(f, "Fat"),
            MajorNutrient::Carbohydrate => write!(f, "Carbohydrate"),
            MajorNutrient::Sugars => write!(f, "Sugars"),
            MajorNutrient::Water => write!(f, "Water"),
            MajorNutrient::Minerals_and_vitamins => write!(f, "Minerals and Vitamins"),
            MajorNutrient::Balanced => write!(f, "Balanced"),
        }
    }
}

/// Food stock item with different properties
#[derive(Debug, Clone)]
pub struct FoodStock {
    pub index: u32,
    pub stock_date: NaiveDate,
    pub food_type: FoodType,
    pub nutrient: MajorNutrient,
    pub storage_type: StorageType,
    pub expiry_date: NaiveDate,
    pub quantity: Unit,
}

impl FoodStock {
    /// Creates a new FoodStock instance
    pub fn new(
        index: u32,
        stock_date: NaiveDate,
        food_type: FoodType,
        nutrient: MajorNutrient,
        storage_type: StorageType,
        expiry_date: NaiveDate,
        quantity: Unit,
    ) -> Self {
        Self {
            index,
            stock_date,
            food_type,
            nutrient,
            storage_type,
            expiry_date,
            quantity,
        }
    }

    /// Estimates the expiry date based on food type and storage conditions
    pub fn estimate_expiry(&self) -> NaiveDate {
        let days_to_add = match (&self.food_type, &self.storage_type) {
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
        self.stock_date + Duration::days(days_to_add)
    }
}

impl fmt::Display for FoodStock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let quantity_str = match &self.quantity {
            Unit::Grams(g) => format!("{}g", g),
            Unit::Litres(l) => format!("{}L", l),
        };
        
        write!(f, "Food Stock #{}: {} ({}) - Stored: {}, Expires: {}, Storage: {}, Nutrient: {}",
            self.index,
            self.food_type,
            quantity_str,
            self.stock_date,
            self.expiry_date,
            self.storage_type,
            self.nutrient
        )
    }
} 