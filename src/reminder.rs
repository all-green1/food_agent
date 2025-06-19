use chrono::{Duration, Utc, TimeZone, NaiveDate, DateTime};
use google_calendar3::{CalendarHub, oauth2, api::Event, api::EventDateTime};
use hyper::{Client};
use hyper_rustls::HttpsConnectorBuilder;
use std::default::Default;
use std::path::Path;
use crate::models::{FoodStock, FoodType, StorageType, Unit, MajorNutrient};
use serde_json::Value;

// Simple URL encoding function for basic characters
fn simple_url_encode(s: &str) -> String {
    s.replace(" ", "%20")
     .replace("!", "%21")
     .replace("(", "%28")
     .replace(")", "%29")
     .replace(",", "%2C")
     .replace(":", "%3A")
     .replace("'", "%27")
}

pub fn generate_calendar_links(food: &FoodStock) -> String {
    // Generate direct calendar links for adding food expiry reminders
    
    // Calculate reminder date (1 day before expiry)
    let reminder_date = food.expiry_date - Duration::days(1);
    let reminder_datetime = Utc.from_utc_date(&reminder_date).and_hms_opt(9, 0, 0).unwrap();
    let end_datetime = reminder_datetime + Duration::hours(1);
    
    // Event details
    let event_title = format!("Check your {} before it expires!", food.name);
    let quantity_str = match &food.quantity {
        Unit::Grams(g) => format!("{}g", g),
        Unit::Litres(l) => format!("{}L", l),
    };
    
    let event_description = format!(
        "Your {} ({}, {}, stored in {} storage) is expiring on {}. Check freshness and plan to use it soon.",
        food.name,
        food.food_type,
        quantity_str,
        food.storage_type,
        food.expiry_date
    );
    
    // Format dates for calendar URLs
    let start_utc = reminder_datetime.format("%Y%m%dT%H%M%SZ").to_string();
    let end_utc = end_datetime.format("%Y%m%dT%H%M%SZ").to_string();
    let start_iso = reminder_datetime.format("%Y-%m-%dT%H:%M:%S").to_string();
    let end_iso = end_datetime.format("%Y-%m-%dT%H:%M:%S").to_string();
    
    // Create Google Calendar URL
    let google_url = format!(
        "https://calendar.google.com/calendar/render?action=TEMPLATE&text={}&dates={}/{}&details={}&location={}",
        simple_url_encode(&event_title),
        start_utc,
        end_utc,
        simple_url_encode(&event_description),
        simple_url_encode("Kitchen/Pantry")
    );
    
    // Create Outlook Calendar URL
    let outlook_url = format!(
        "https://outlook.live.com/calendar/0/deeplink/compose?subject={}&startdt={}&enddt={}&body={}&location={}",
        simple_url_encode(&event_title),
        start_iso,
        end_iso,
        simple_url_encode(&event_description),
        simple_url_encode("Kitchen/Pantry")
    );
    
    // Create the calendar message with links
    let calendar_message = format!(
        "📅 **Calendar Reminder Links**\n\nClick any link below to add this reminder to your calendar:\n\n🔗 **[Add to Google Calendar]({})**\n🔗 **[Add to Outlook Calendar]({})**\n\n**Event Details:**\n• **Title:** {}\n• **Date:** {}\n• **Food:** {} ({}, {})\n• **Storage:** {} storage\n• **Expires:** {}\n\n💡 **Tip:** Connect your Google Calendar in settings for automatic reminders!",
        google_url,
        outlook_url,
        event_title,
        reminder_datetime.format("%B %d, %Y at 9:00 AM"),
        food.name,
        food.food_type,
        quantity_str,
        food.storage_type,
        food.expiry_date
    );
    
    calendar_message
}

pub async fn create_calendar_event_with_user_token(food: &FoodStock, user_id: i32, token_data: Option<Value>) -> Result<String, Box<dyn std::error::Error>> {
    // If no token data provided, return calendar links instead of error
    let token_data = match token_data {
        Some(data) => data,
        None => {
            println!("DEBUG: No Google Calendar token available for user {}, generating calendar links", user_id);
            return Ok(generate_calendar_links(food));
        }
    };

    println!("DEBUG: Creating calendar event for user {} with stored token", user_id);

    // Try multiple possible locations for secrets.json
    let possible_paths = [
        "secrets.json",
        "../secrets.json",
        "../../secrets.json",
        "./secrets.json",
    ];
    
    let mut secrets_path = None;
    for path in &possible_paths {
        if Path::new(path).exists() {
            secrets_path = Some(*path);
            break;
        }
    }
    
    let secrets_path = secrets_path.ok_or_else(|| {
        format!("secrets.json not found. Searched in: {:?}. Please ensure your OAuth 2.0 credentials are saved as 'secrets.json' in an accessible location.", possible_paths)
    })?;

    let secret = oauth2::read_application_secret(secrets_path).await?;
    
    // Create authenticator with stored credentials - for now, we'll use file-based approach
    // but pass the token data through a temporary file or memory
    let auth = oauth2::InstalledFlowAuthenticator::builder(secret, oauth2::InstalledFlowReturnMethod::HTTPRedirect)
        .persist_tokens_to_disk("token.json") // This will be ignored if token exists
        .build()
        .await?;

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
        "Your {} ({}) will get bad after {}. Storage type: {}",
        food.food_type,
        quantity_str,
        food.expiry_date,
        food.storage_type,
    );

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
            println!("DEBUG: Successfully created calendar event for user {}", user_id);
            if let Some(html_link) = event.html_link {
                Ok(format!("Calendar event created: {}", html_link))
            } else {
                Ok("Calendar event created successfully".to_string())
            }
        },
        Err(e) => {
            eprintln!("DEBUG: Error creating calendar event for user {}: {:?}", user_id, e);
            // If calendar creation fails, return calendar links as fallback
            Ok(generate_calendar_links(food))
        }
    }
}

// Keep the original function for backward compatibility, but mark it as deprecated
pub async fn create_calendar_event(food: &FoodStock) -> Result<(), Box<dyn std::error::Error>> {
    // This is the old implementation - we'll keep it for now but use the new one
    match create_calendar_event_with_user_token(food, 0, None).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e)
    }
}
