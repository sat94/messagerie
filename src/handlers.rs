use actix_web::{web, HttpResponse};
use mongodb::{bson::doc, Database};
use serde_json::json;
use futures_util::stream::TryStreamExt;
use crate::{ApiResponse, Message};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct UserInfo {
    pub username: String,
    pub prenom: String,
    pub age: i32,
    pub photo: String,
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
    username: web::Path<String>,
) -> HttpResponse {
    let username = username.into_inner();
    let conversations_collection = db.collection::<mongodb::bson::Document>("conversations");

    let filter = doc! { "user_id": &username };

    match conversations_collection.find_one(filter, None).await {
        Ok(Some(doc)) => {
            let mut conversations = Vec::new();

            if let Some(convs) = doc.get_array("conversations").ok() {
                for conv_doc in convs {
                    if let Some(conv_obj) = conv_doc.as_document() {
                        if let (Some(user), Some(prenom), Some(age), Some(photo)) = (
                            conv_obj.get_str("username").ok(),
                            conv_obj.get_str("prenom").ok(),
                            conv_obj.get_i32("age").ok(),
                            conv_obj.get_str("photo").ok(),
                        ) {
                            conversations.push(UserInfo {
                                username: user.to_string(),
                                prenom: prenom.to_string(),
                                age,
                                photo: photo.to_string(),
                            });
                        }
                    }
                }
            }

            let response = HistoryResponse {
                username: username.clone(),
                conversations,
            };

            HttpResponse::Ok().json(ApiResponse::ok(response))
        }
        Ok(None) => {
            let response = HistoryResponse {
                username: username.clone(),
                conversations: Vec::new(),
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

