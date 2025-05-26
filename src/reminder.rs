use chrono::{Duration, Utc, TimeZone, NaiveDate, DateTime};
use google_calendar3::{CalendarHub, oauth2, api::Event, api::EventDateTime};
use hyper::{Client};
use hyper_rustls::HttpsConnectorBuilder;
use std::default::Default;
use std::path::Path;
use crate::models::{FoodStock, FoodType, StorageType, Unit, MajorNutrient};

pub async fn create_calendar_event(food: &FoodStock) -> Result<(), Box<dyn std::error::Error>> {
    // Check if secrets.json exists
    if !Path::new("secrets.json").exists() {
        return Err("secrets.json not found. Please download your OAuth 2.0 credentials from Google Cloud Console and save them as 'secrets.json' in the project root.".into());
    }

    println!("Reading OAuth credentials from secrets.json...");
    let secret = oauth2::read_application_secret("secrets.json").await?;
    
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

    let summary = format!("Check your {} before it expires!", food.food_type);
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
