use chrono::{Duration, Utc, TimeZone, NaiveDate, DateTime};
use google_calendar3::{CalendarHub, oauth2, api::Event, api::EventDateTime};
use hyper::{Client};
use hyper_rustls::HttpsConnectorBuilder;
use std::default::Default;
use std::path::Path;
use crate::models::{FoodStock, FoodType, StorageType, Unit, MajorNutrient};

pub async fn create_calendar_event(food: &FoodStock) -> Result<(), Box<dyn std::error::Error>> {
    // Debug: Print current working directory
    if let Ok(current_dir) = std::env::current_dir() {
        println!("DEBUG: Current working directory: {:?}", current_dir);
    }
    
    // Try multiple possible locations for secrets.json
    let possible_paths = [
        "secrets.json",           // Current directory
        "../secrets.json",        // One level up
        "../../secrets.json",     // Two levels up
        "./secrets.json",         // Explicit current directory
    ];
    
    let mut secrets_path = None;
    for path in &possible_paths {
        println!("DEBUG: Checking for secrets.json at: {}", path);
        if Path::new(path).exists() {
            secrets_path = Some(*path);
            println!("DEBUG: Found secrets.json at: {}", path);
            break;
        }
    }
    
    let secrets_path = secrets_path.ok_or_else(|| {
        format!("secrets.json not found. Searched in: {:?}. Please ensure your OAuth 2.0 credentials are saved as 'secrets.json' in an accessible location.", possible_paths)
    })?;

    println!("Reading OAuth credentials from {}...", secrets_path);
    let secret = oauth2::read_application_secret(secrets_path).await?;
    
    println!("Setting up OAuth2 authentication...");
    let auth = oauth2::InstalledFlowAuthenticator::builder(secret, oauth2::InstalledFlowReturnMethod::HTTPRedirect)
        .persist_tokens_to_disk("token.json")
        .build()
        .await?;

    println!("Creating Google Calendar client...");
    let hub = CalendarHub::new(
        Client::builder().build(
            HttpsConnectorBuilder::new()
                .with_native_roots()
                .https_or_http()
                .enable_http1()
                .build(),
        ),
        auth,
    );

    let reminder_date = Utc.from_utc_date(&food.expiry_date).and_hms_opt(9, 0, 0).unwrap();

    let summary = format!("Check your {} before it expires!", food.name);
    let quantity_str = match &food.quantity {
        Unit::Grams(g) => format!("{}g", g),
        Unit::Litres(l) => format!("{}L", l),
    };
    let description = format!(
        "This {} ({}) is expiring on {}. Storage type: {}, Nutrient type: {}",
        food.food_type,
        quantity_str,
        food.expiry_date,
        food.storage_type,
        food.nutrient
    );

    println!("Creating calendar event...");
    let event = Event {
        summary: Some(summary),
        description: Some(description),
        start: Some(EventDateTime {
            date_time: Some(reminder_date),
            time_zone: Some("UTC".to_string()),
            ..Default::default()
        }),
        end: Some(EventDateTime {
            date_time: Some(reminder_date + Duration::hours(1)),
            time_zone: Some("UTC".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let result = hub.events().insert(event, "primary").doit().await;

    match result {
        Ok((_, event)) => {
            println!("Successfully added event to calendar!");
            println!("Event link: {:?}", event.html_link);
            Ok(())
        },
        Err(e) => {
            eprintln!("Error adding event to calendar: {:?}", e);
            Err(e.into())
        }
    }
}
