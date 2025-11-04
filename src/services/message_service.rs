use mongodb::{Database, bson::{doc, Document}};
use crate::models::Message;
use log::info;
use chrono::Utc;

pub struct MessageService;

impl MessageService {
    /// Récupère l'historique des messages pour un utilisateur
    pub async fn get_history(
        db: &Database,
        username: &str,
        limit: i64,
    ) -> Result<Vec<Message>, Box<dyn std::error::Error>> {
        let collection = db.collection::<Document>("messages");

        let filter = doc! {
            "$or": [
                { "from": username },
                { "to": username }
            ]
        };

        let options = mongodb::options::FindOptions::builder()
            .sort(doc! { "timestamp": -1 })
            .limit(limit)
            .build();

        let mut cursor = collection.find(filter, options).await?;
        let mut messages = Vec::new();

        while cursor.advance().await? {
            let doc = cursor.deserialize_current()?;
            let message = convert_document_to_message(doc)?;
            messages.push(message);
        }

        messages.reverse(); // Ordre chronologique
        info!("📨 Historique récupéré pour {}: {} messages", username, messages.len());

        Ok(messages)
    }

    /// Récupère la conversation entre deux utilisateurs
    pub async fn get_conversation(
        db: &Database,
        user1: &str,
        user2: &str,
        limit: i64,
    ) -> Result<Vec<Message>, Box<dyn std::error::Error>> {
        let collection = db.collection::<Document>("messages");

        let filter = doc! {
            "$or": [
                { "from": user1, "to": user2 },
                { "from": user2, "to": user1 }
            ]
        };

        let options = mongodb::options::FindOptions::builder()
            .sort(doc! { "timestamp": -1 })
            .limit(limit)
            .build();

        let mut cursor = collection.find(filter, options).await?;
        let mut messages = Vec::new();

        while cursor.advance().await? {
            let doc = cursor.deserialize_current()?;
            let message = convert_document_to_message(doc)?;
            messages.push(message);
        }

        messages.reverse(); // Ordre chronologique
        info!("💬 Conversation {} ↔ {}: {} messages", user1, user2, messages.len());

        Ok(messages)
    }

    /// Enregistre un nouveau message
    pub async fn save_message(
        db: &Database,
        sender: &str,
        recipient: &str,
        content: &str,
        message_type: &str,
    ) -> Result<Message, Box<dyn std::error::Error>> {
        let collection = db.collection::<Message>("messages");

        let message = Message {
            id: None,
            sender: sender.to_string(),
            recipient: recipient.to_string(),
            content: content.to_string(),
            message_type: message_type.to_string(),
            timestamp: Utc::now().to_rfc3339(),
            read: false,
            read_at: None,
        };

        let _result = collection.insert_one(&message, None).await?;
        info!("✉️ Message enregistré: {} → {}", sender, recipient);

        Ok(message)
    }

    /// Marque les messages comme lus
    pub async fn mark_as_read(
        db: &Database,
        sender: &str,
        recipient: &str,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        let collection = db.collection::<Document>("messages");

        let filter = doc! {
            "sender": sender,
            "recipient": recipient,
            "read": false
        };

        let now = Utc::now();
        let update = doc! {
            "$set": {
                "read": true,
                "read_at": now.to_rfc3339()
            }
        };

        let result = collection.update_many(filter, update, None).await?;
        info!("✓ {} messages marqués comme lus", result.modified_count);

        Ok(result.modified_count)
    }
}

/// Convertit un Document BSON en Message
fn convert_document_to_message(doc: Document) -> Result<Message, Box<dyn std::error::Error>> {
    let id = doc.get_object_id("_id").ok().map(|oid| oid.to_string());

    // Map "from" to "sender" and "to" to "recipient"
    let sender = doc.get_str("from")
        .or_else(|_| doc.get_str("sender"))
        .unwrap_or("")
        .to_string();

    let recipient = doc.get_str("to")
        .or_else(|_| doc.get_str("recipient"))
        .unwrap_or("")
        .to_string();

    // Map "message" to "content"
    let content = doc.get_str("message")
        .or_else(|_| doc.get_str("content"))
        .unwrap_or("")
        .to_string();

    let message_type = doc.get_str("message_type").unwrap_or("text").to_string();

    // Récupère le timestamp - peut être une string ou un BSON DateTime
    let timestamp = if let Ok(ts_str) = doc.get_str("timestamp") {
        ts_str.to_string()
    } else if let Ok(bson_dt) = doc.get_datetime("timestamp") {
        bson_dt.to_rfc3339_string()
    } else {
        Utc::now().to_rfc3339()
    };

    let read = doc.get_bool("read").unwrap_or(false);
    let read_at = doc.get_str("read_at").ok().map(|s| s.to_string());

    Ok(Message {
        id,
        sender,
        recipient,
        content,
        message_type,
        timestamp,
        read,
        read_at,
    })
}

