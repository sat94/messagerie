use actix_web::{web, HttpResponse};
use mongodb::{Database, bson::doc};
use serde_json::json;
use crate::models::{SendMessageRequest, ApiResponse};
use crate::services::message_service::MessageService;

pub async fn get_history(
    db: web::Data<Database>,
    username: web::Path<String>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    let limit = query
        .get("limit")
        .and_then(|l| l.parse::<i64>().ok())
        .unwrap_or(100);

    match MessageService::get_history(&db, &username, limit).await {
        Ok(messages) => {
            let response = ApiResponse::ok(json!({
                "username": username.into_inner(),
                "messages": messages,
                "count": messages.len()
            }));
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            let response: ApiResponse<()> = ApiResponse::err(format!("Erreur: {}", e));
            HttpResponse::InternalServerError().json(response)
        }
    }
}

pub async fn get_conversation(
    db: web::Data<Database>,
    path: web::Path<(String, String)>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> HttpResponse {
    let (user1, user2) = path.into_inner();
    let limit = query
        .get("limit")
        .and_then(|l| l.parse::<i64>().ok())
        .unwrap_or(100);

    match MessageService::get_conversation(&db, &user1, &user2, limit).await {
        Ok(messages) => {
            let response = ApiResponse::ok(json!({
                "participants": [user1, user2],
                "messages": messages,
                "count": messages.len()
            }));
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            let response: ApiResponse<()> = ApiResponse::err(format!("Erreur: {}", e));
            HttpResponse::InternalServerError().json(response)
        }
    }
}

pub async fn send_message(
    db: web::Data<Database>,
    req: web::Json<SendMessageRequest>,
) -> HttpResponse {
    match MessageService::save_message(
        &db,
        &req.sender,
        &req.recipient,
        &req.content,
        &req.message_type,
    )
    .await
    {
        Ok(message) => {
            let response = ApiResponse::ok(message);
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            let response: ApiResponse<()> = ApiResponse::err(format!("Erreur: {}", e));
            HttpResponse::InternalServerError().json(response)
        }
    }
}

pub async fn debug_count_messages(
    db: web::Data<Database>,
) -> HttpResponse {
    let collection = db.collection::<mongodb::bson::Document>("messages");

    match collection.count_documents(doc! {}, None).await {
        Ok(count) => {
            let response = ApiResponse::ok(json!({
                "total_messages": count,
                "database": "meetvoice_gateway",
                "collection": "messages"
            }));
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            let response: ApiResponse<()> = ApiResponse::err(format!("Erreur: {}", e));
            HttpResponse::InternalServerError().json(response)
        }
    }
}

pub async fn debug_list_users(
    db: web::Data<Database>,
) -> HttpResponse {
    let collection = db.collection::<mongodb::bson::Document>("messages");

    // Get all unique senders and recipients
    match collection.distinct("sender", None, None).await {
        Ok(senders) => {
            match collection.distinct("recipient", None, None).await {
                Ok(recipients) => {
                    let response = ApiResponse::ok(json!({
                        "senders": senders,
                        "recipients": recipients
                    }));
                    HttpResponse::Ok().json(response)
                }
                Err(e) => {
                    let response: ApiResponse<()> = ApiResponse::err(format!("Erreur: {}", e));
                    HttpResponse::InternalServerError().json(response)
                }
            }
        }
        Err(e) => {
            let response: ApiResponse<()> = ApiResponse::err(format!("Erreur: {}", e));
            HttpResponse::InternalServerError().json(response)
        }
    }
}

pub async fn debug_list_collections(
    db: web::Data<Database>,
) -> HttpResponse {
    match db.list_collection_names(None).await {
        Ok(collections) => {
            let response = ApiResponse::ok(json!({
                "collections": collections,
                "database": "meetvoice_gateway"
            }));
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            let response: ApiResponse<()> = ApiResponse::err(format!("Erreur: {}", e));
            HttpResponse::InternalServerError().json(response)
        }
    }
}

pub async fn debug_collection_stats(
    db: web::Data<Database>,
) -> HttpResponse {
    let mut stats = json!({});

    // Check each collection
    for collection_name in &["messages", "conversations", "message_queue"] {
        let collection = db.collection::<mongodb::bson::Document>(collection_name);
        match collection.count_documents(doc! {}, None).await {
            Ok(count) => {
                stats[collection_name] = json!({
                    "count": count
                });
            }
            Err(e) => {
                stats[collection_name] = json!({
                    "error": e.to_string()
                });
            }
        }
    }

    let response = ApiResponse::ok(stats);
    HttpResponse::Ok().json(response)
}

pub async fn debug_sample_document(
    db: web::Data<Database>,
) -> HttpResponse {
    let collection = db.collection::<mongodb::bson::Document>("messages");

    match collection.find_one(None, None).await {
        Ok(Some(doc)) => {
            let response = ApiResponse::ok(json!({
                "sample_document": doc,
                "fields": doc.keys().collect::<Vec<_>>()
            }));
            HttpResponse::Ok().json(response)
        }
        Ok(None) => {
            let response: ApiResponse<()> = ApiResponse::err("Aucun document trouvé".to_string());
            HttpResponse::NotFound().json(response)
        }
        Err(e) => {
            let response: ApiResponse<()> = ApiResponse::err(format!("Erreur: {}", e));
            HttpResponse::InternalServerError().json(response)
        }
    }
}

pub async fn debug_all_users(
    db: web::Data<Database>,
) -> HttpResponse {
    let collection = db.collection::<mongodb::bson::Document>("messages");

    // Get all unique "from" values
    match collection.distinct("from", None, None).await {
        Ok(from_users) => {
            match collection.distinct("to", None, None).await {
                Ok(to_users) => {
                    let response = ApiResponse::ok(json!({
                        "from_users": from_users,
                        "to_users": to_users,
                        "from_count": from_users.len(),
                        "to_count": to_users.len()
                    }));
                    HttpResponse::Ok().json(response)
                }
                Err(e) => {
                    let response: ApiResponse<()> = ApiResponse::err(format!("Erreur: {}", e));
                    HttpResponse::InternalServerError().json(response)
                }
            }
        }
        Err(e) => {
            let response: ApiResponse<()> = ApiResponse::err(format!("Erreur: {}", e));
            HttpResponse::InternalServerError().json(response)
        }
    }
}

