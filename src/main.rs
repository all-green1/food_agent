#![allow(warnings)]
mod models;
mod handlers;
mod storage;
mod reminder;

use handlers::CommandHandler;
use storage::FoodStorage;

#[tokio::main]
async fn main() {
    // println!("Welcome to Food Stock Manager!");
    // println!("I'm your food log assistant. What would you like me to do for you?");
    
    let mut storage = FoodStorage::new();
    let command_handler = CommandHandler::new();
    
    // loop {
    //     println!("\nAvailable commands:");
    //     println!("- Add: Add new food stock");
    //     println!("- View All: View all food stocks");
    //     println!("- Search: Search food stocks");
    //     println!("- Exit: Exit the program");
        
    //     let mut command = String::new();
    //     if let Err(e) = std::io::stdin().read_line(&mut command) {
    //         eprintln!("Error reading command: {}", e);
    //         continue;
    //     }
        
    //     let command = command.trim().to_lowercase();
        
//         match command.as_str() {
//             "add" => {
//                 if let Err(e) = command_handler.handle_add(&mut storage, stock_date, food_type, storage_type, quantity, expiry_date).await {
//                     eprintln!("Error: {}", e);
//                 }
//             }
//             "view all" => {
//                 if let Err(e) = command_handler.handle_view_all(&storage) {
//                     eprintln!("Error: {}", e);
//                 }
//             }
//             "search" => {
//                 if let Err(e) = command_handler.handle_search(&storage) {
//                     eprintln!("Error: {}", e);
//                 }
//             }
//             "exit" => {
//                 println!("Goodbye!");
//                 break;
//             }
//             _ => println!("Invalid command. Please try again."),
//         }
//     }
}