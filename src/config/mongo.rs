use mongodb::{Client, Database};
use std::env;
use log::info;

pub async fn init_mongo() -> Result<Database, Box<dyn std::error::Error>> {
    let mongo_uri = env::var("MONGO_URI")
        .unwrap_or_else(|_| "mongodb://gateway_user:gateway_password_2025@localhost:27019/meetvoice_gateway?authSource=admin".to_string());

    info!("📦 Connexion à MongoDB: {}", mongo_uri);

    let client = Client::with_uri_str(&mongo_uri).await?;
    let db = client.database("meetvoice_gateway");

    // Test de connexion
    db.run_command(mongodb::bson::doc! { "ping": 1 }, None).await?;
    info!("✅ Connecté à MongoDB (meetvoice_gateway)");

    Ok(db)
}

