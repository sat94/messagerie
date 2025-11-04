use actix_web::{web, HttpResponse};
use mongodb::{bson::doc, Database};
use serde_json::json;
use futures_util::stream::TryStreamExt;
use std::sync::Arc;
use crate::{ApiResponse, Message};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct UserInfo {
    pub username: String,
    pub prenom: String,
    pub age: i32,
    pub photo: String,
    pub last_message: String,
    pub last_timestamp: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct HistoryResponse {
    pub username: String,
    pub conversations: Vec<UserInfo>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ConversationResponse {
    pub messages: Vec<Message>,
    pub count: usize,
}

pub async fn get_history(
    db: web::Data<Database>,
    pg_client: web::Data<Option<Arc<tokio_postgres::Client>>>,
    username: web::Path<String>,
) -> HttpResponse {
    let username = username.into_inner();
    let messages_collection = db.collection::<mongodb::bson::Document>("messages");

    // Récupérer tous les messages de l'utilisateur
    let filter = doc! {
        "$or": [
            { "from": &username },
            { "to": &username }
        ]
    };

    match messages_collection.find(filter, None).await {
        Ok(mut cursor) => {
            use std::collections::HashMap;
            let mut conversations_map: HashMap<String, UserInfo> = HashMap::new();

            // Parcourir tous les messages et construire les conversations
            while let Ok(Some(msg_doc)) = cursor.try_next().await {
                let from = msg_doc.get_str("from").unwrap_or("").to_string();
                let to = msg_doc.get_str("to").unwrap_or("").to_string();
                let message = msg_doc.get_str("message").unwrap_or("").to_string();
                let timestamp = msg_doc.get_str("timestamp").unwrap_or("").to_string();

                // Déterminer l'autre utilisateur
                let other_user = if from == username { to } else { from };

                // Mettre à jour le dernier message et timestamp
                conversations_map.entry(other_user.clone())
                    .and_modify(|conv| {
                        conv.last_message = message.clone();
                        conv.last_timestamp = timestamp.clone();
                    })
                    .or_insert_with(|| UserInfo {
                        username: other_user.clone(),
                        prenom: String::new(),
                        age: 0,
                        photo: String::new(),
                        last_message: message,
                        last_timestamp: timestamp,
                    });
            }

            // Récupérer les infos de profil depuis la collection conversations
            let conversations_collection = db.collection::<mongodb::bson::Document>("conversations");
            let conv_filter = doc! { "user_id": &username };

            if let Ok(Some(conv_doc)) = conversations_collection.find_one(conv_filter, None).await {
                if let Some(convs) = conv_doc.get_array("conversations").ok() {
                    for conv_doc in convs {
                        if let Some(conv_obj) = conv_doc.as_document() {
                            if let Some(user) = conv_obj.get_str("username").ok() {
                                if let Some(conv_info) = conversations_map.get_mut(user) {
                                    if let Some(prenom) = conv_obj.get_str("prenom").ok() {
                                        conv_info.prenom = prenom.to_string();
                                    }
                                    if let Some(age) = conv_obj.get_i32("age").ok() {
                                        conv_info.age = age;
                                    }
                                    if let Some(photo) = conv_obj.get_str("photo").ok() {
                                        conv_info.photo = photo.to_string();
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Si PostgreSQL est disponible, récupérer les infos manquantes
            if let Some(client) = pg_client.as_ref() {
                for conv_info in conversations_map.values_mut() {
                    // Si les infos sont vides, les récupérer depuis PostgreSQL
                    if conv_info.prenom.is_empty() || conv_info.age == 0 || conv_info.photo.is_empty() {
                        if let Ok(rows) = client.query(
                            "SELECT \"firstName\" FROM users WHERE email = $1 LIMIT 1",
                            &[&conv_info.username]
                        ).await {
                            if let Some(row) = rows.first() {
                                if conv_info.prenom.is_empty() {
                                    if let Ok(first_name) = row.try_get::<_, Option<String>>("firstName") {
                                        conv_info.prenom = first_name.unwrap_or_default();
                                    }
                                }
                            }
                        }
                    }
                }
            }

            let mut conversations: Vec<UserInfo> = conversations_map.into_values().collect();
            // Trier par timestamp décroissant (plus récent en premier)
            conversations.sort_by(|a, b| b.last_timestamp.cmp(&a.last_timestamp));

            let response = HistoryResponse {
                username: username.clone(),
                conversations,
            };

            HttpResponse::Ok().json(ApiResponse::ok(response))
        }
        Err(e) => {
            log::error!("Erreur MongoDB: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::err(format!("Erreur: {}", e)))
        }
    }
}

pub async fn get_conversation(
    db: web::Data<Database>,
    path: web::Path<(String, String)>,
) -> HttpResponse {
    let (user1, user2) = path.into_inner();
    let messages_collection = db.collection::<mongodb::bson::Document>("messages");

    let filter = doc! {
        "$or": [
            { "from": &user1, "to": &user2 },
            { "from": &user2, "to": &user1 }
        ]
    };

    match messages_collection.find(filter, None).await {
        Ok(mut cursor) => {
            let mut messages = Vec::new();
            while let Ok(Some(doc)) = cursor.try_next().await {
                if let Ok(msg) = convert_doc_to_message(doc) {
                    messages.push(msg);
                }
            }

            let count = messages.len();
            let response = ConversationResponse {
                messages,
                count,
            };

            HttpResponse::Ok().json(ApiResponse::ok(response))
        }
        Err(e) => {
            log::error!("Erreur MongoDB: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::err(format!("Erreur: {}", e)))
        }
    }
}

pub async fn delete_conversation(
    db: web::Data<Database>,
    path: web::Path<(String, String)>,
) -> HttpResponse {
    let (user_a, user_b) = path.into_inner();
    let messages_collection = db.collection::<mongodb::bson::Document>("messages");

    let filter = doc! {
        "$or": [
            { "from": &user_a, "to": &user_b },
            { "from": &user_b, "to": &user_a }
        ]
    };

    match messages_collection.delete_many(filter, None).await {
        Ok(result) => {
            let deleted_count = result.deleted_count;

            #[derive(serde::Serialize)]
            struct DeleteResponse {
                deleted_count: u64,
            }

            let response = DeleteResponse {
                deleted_count,
            };

            HttpResponse::Ok().json(ApiResponse::ok(response))
        }
        Err(e) => {
            log::error!("Erreur MongoDB: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::err(format!("Erreur: {}", e)))
        }
    }
}

fn convert_doc_to_message(doc: mongodb::bson::Document) -> Result<Message, Box<dyn std::error::Error>> {
    let id = doc.get_object_id("_id").ok().map(|oid| oid.to_string());
    let from = doc.get_str("from").unwrap_or("").to_string();
    let to = doc.get_str("to").unwrap_or("").to_string();
    let message = doc.get_str("message").unwrap_or("").to_string();
    let timestamp = doc.get_str("timestamp").unwrap_or("").to_string();
    let read = doc.get_bool("read").unwrap_or(false);

    Ok(Message {
        id,
        from,
        to,
        message,
        timestamp,
        read,
    })
}

