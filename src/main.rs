use actix_web::{web, App, HttpServer, HttpResponse};
use actix_cors::Cors;
use mongodb::{Client, Database};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

mod handlers;

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub from: String,
    pub to: String,
    pub message: String,
    pub timestamp: String,
    #[serde(default)]
    pub read: bool,
    #[serde(default)]
    pub is_connect: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemNotification {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub from: String,
    pub to: String,
    pub r#type: String,
    pub title: String,
    pub message: String,
    pub timestamp: String,
    #[serde(default)]
    pub read: bool,
    pub priority: String,
    pub action_url: Option<String>,
    pub created_by: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupAccessRequest {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub requester_username: String,
    pub group_id: String,
    pub group_name: String,
    pub group_owner: String,
    pub status: String,
    pub timestamp: String,
    pub response_timestamp: Option<String>,
    pub response_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PhotoPermissionRequest {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub requester_username: String,
    pub target_username: String,
    pub status: String,
    pub timestamp: String,
    pub response_timestamp: Option<String>,
    pub response_message: Option<String>,
    pub permission_expires_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventParticipationRequest {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub requester_username: String,
    pub event_id: String,
    pub event_name: String,
    pub event_creator: String,
    pub status: String,
    pub timestamp: String,
    pub response_timestamp: Option<String>,
    pub response_message: Option<String>,
    pub participation_role: String,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        ApiResponse {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(error: String) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    dotenv::dotenv().ok();

    let mongo_uri = std::env::var("MONGO_URI")
        .unwrap_or_else(|_| "mongodb://gateway_user:gateway_password_2025@localhost:27019/meetvoice_gateway?authSource=admin".to_string());

    let client = Client::with_uri_str(&mongo_uri)
        .await
        .expect("Failed to connect to MongoDB");

    let db = client.database("meetvoice_gateway");
    let db_data = web::Data::new(db);

    // PostgreSQL connection (optional)
    let db_user = std::env::var("DB_USER").unwrap_or_else(|_| "meetvoice_api_user".to_string());
    let db_password = std::env::var("DB_PASSWORD").unwrap_or_else(|_| "meetvoice_api_2025".to_string());
    let db_host = std::env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string());
    let db_port = std::env::var("DB_PORT").unwrap_or_else(|_| "5432".to_string());
    let db_name = std::env::var("DB_NAME").unwrap_or_else(|_| "meetvoice_api".to_string());

    let postgres_uri = format!(
        "postgres://{}:{}@{}:{}/{}",
        db_user, db_password, db_host, db_port, db_name
    );

    let pg_client = match tokio_postgres::connect(&postgres_uri, tokio_postgres::tls::NoTls).await {
        Ok((client, connection)) => {
            log::info!("✅ PostgreSQL connecté");
            tokio::spawn(async move {
                if let Err(e) = connection.await {
                    log::error!("PostgreSQL connection error: {}", e);
                }
            });
            Some(Arc::new(client))
        }
        Err(e) => {
            log::warn!("⚠️  PostgreSQL non disponible: {}. Les infos de profil seront récupérées uniquement depuis MongoDB.", e);
            None
        }
    };
    let pg_data = web::Data::new(pg_client);

    log::info!("🚀 Serveur démarrant sur http://0.0.0.0:3000");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials();

        App::new()
            .wrap(cors)
            .app_data(db_data.clone())
            .app_data(pg_data.clone())
            .route("/health", web::get().to(health_check))
            .route("/api/messages/history/{username}", web::get().to(handlers::get_history))
            .route("/api/messages/conversation/{user1}/{user2}", web::get().to(handlers::get_conversation))
            .route("/api/messages/conversation/{user1}/{user2}", web::delete().to(handlers::delete_conversation))
            // Nouveaux endpoints
            .route("/api/notifications/system-message", web::post().to(handlers::create_system_notification))
            .route("/api/requests/group-access", web::post().to(handlers::create_group_access_request))
            .route("/api/requests/private-photos-permission", web::post().to(handlers::create_photo_permission_request))
            .route("/api/requests/event-participation", web::post().to(handlers::create_event_participation_request))
    })
    .bind("0.0.0.0:3000")?
    .run()
    .await
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(json!({"status": "ok"}))
}
