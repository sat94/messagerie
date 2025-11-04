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
    let postgres_uri = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://meetvoice_api_user:meetvoice_api_password@192.168.1.42:5432/meetvoice_api".to_string());

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
    })
    .bind("0.0.0.0:3000")?
    .run()
    .await
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(json!({"status": "ok"}))
}
