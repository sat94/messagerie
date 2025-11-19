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
    pub date_de_naissance: String,
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
                        date_de_naissance: String::new(),
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
                                    if let Some(dob) = conv_obj.get_str("date_de_naissance").ok() {
                                        conv_info.date_de_naissance = dob.to_string();
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
                    if conv_info.prenom.is_empty() || conv_info.date_de_naissance.is_empty() || conv_info.photo.is_empty() {
                        if let Ok(rows) = client.query(
                            "SELECT cc.prenom, cc.date_de_naissance::text,
                                    COALESCE(
                                        (SELECT cp.photos FROM compte_photo cp WHERE cp.compte_id = cc.id AND cp.type_photo = 'principale' LIMIT 1),
                                        (SELECT cp.photos FROM compte_photo cp WHERE cp.compte_id = cc.id ORDER BY cp.ordre ASC LIMIT 1)
                                    ) as photos
                             FROM compte_compte cc
                             WHERE cc.username = $1 LIMIT 1",
                            &[&conv_info.username]
                        ).await {
                            if let Some(row) = rows.first() {
                                if conv_info.prenom.is_empty() {
                                    if let Ok(prenom) = row.try_get::<_, Option<String>>("prenom") {
                                        conv_info.prenom = prenom.unwrap_or_default();
                                    }
                                }
                                if conv_info.date_de_naissance.is_empty() {
                                    if let Ok(dob) = row.try_get::<_, Option<String>>("date_de_naissance") {
                                        if let Some(date_str) = dob {
                                            conv_info.date_de_naissance = date_str;
                                        }
                                    }
                                }
                                if conv_info.photo.is_empty() {
                                    if let Ok(photo) = row.try_get::<_, Option<String>>("photos") {
                                        conv_info.photo = photo.unwrap_or_default();
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
    let (requester, other_user) = path.into_inner();
    let messages_collection = db.collection::<mongodb::bson::Document>("messages");

    // Supprimer uniquement les messages de l'utilisateur qui fait la requête
    // Respecte la RGPD : chaque utilisateur ne supprime que ses propres messages
    let filter = doc! {
        "from": &requester,
        "to": &other_user
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
    let is_connect = doc.get_bool("is_connect").unwrap_or(false);

    Ok(Message {
        id,
        from,
        to,
        message,
        timestamp,
        read,
        is_connect,
    })
}

// ============ NOUVEAUX ENDPOINTS ============

// 1. POST /api/notifications/system-message
#[derive(serde::Deserialize)]
pub struct CreateSystemNotificationRequest {
    pub to: String,
    pub title: String,
    pub message: String,
    pub priority: Option<String>,
    pub action_url: Option<String>,
    pub created_by: String,
}

pub async fn create_system_notification(
    db: web::Data<Database>,
    req: web::Json<CreateSystemNotificationRequest>,
) -> HttpResponse {
    let notifications_collection = db.collection::<mongodb::bson::Document>("system_notifications");

    let now = chrono::Utc::now().to_rfc3339();

    let doc = doc! {
        "from": "meet-voice.fr",
        "to": &req.to,
        "type": "system",
        "title": &req.title,
        "message": &req.message,
        "timestamp": &now,
        "read": false,
        "priority": req.priority.as_deref().unwrap_or("normal"),
        "action_url": &req.action_url,
        "created_by": &req.created_by,
    };

    match notifications_collection.insert_one(doc, None).await {
        Ok(result) => {
            HttpResponse::Created().json(ApiResponse::ok(json!({
                "id": result.inserted_id.to_string(),
                "message": "Notification créée avec succès"
            })))
        }
        Err(e) => {
            log::error!("Erreur MongoDB: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::err(format!("Erreur: {}", e)))
        }
    }
}

// 2. POST /api/requests/group-access
#[derive(serde::Deserialize)]
pub struct CreateGroupAccessRequest {
    pub requester_username: String,
    pub group_id: String,
    pub group_name: String,
    pub group_owner: String,
}

pub async fn create_group_access_request(
    db: web::Data<Database>,
    req: web::Json<CreateGroupAccessRequest>,
) -> HttpResponse {
    let requests_collection = db.collection::<mongodb::bson::Document>("group_access_requests");

    let now = chrono::Utc::now().to_rfc3339();

    let doc = doc! {
        "requester_username": &req.requester_username,
        "group_id": &req.group_id,
        "group_name": &req.group_name,
        "group_owner": &req.group_owner,
        "status": "pending",
        "timestamp": &now,
        "response_timestamp": null,
        "response_message": null,
    };

    match requests_collection.insert_one(doc, None).await {
        Ok(result) => {
            HttpResponse::Created().json(ApiResponse::ok(json!({
                "id": result.inserted_id.to_string(),
                "message": "Demande d'accès au groupe créée"
            })))
        }
        Err(e) => {
            log::error!("Erreur MongoDB: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::err(format!("Erreur: {}", e)))
        }
    }
}

// 3. POST /api/requests/private-photos-permission
#[derive(serde::Deserialize)]
pub struct CreatePhotoPermissionRequest {
    pub requester_username: String,
    pub target_username: String,
}

pub async fn create_photo_permission_request(
    db: web::Data<Database>,
    req: web::Json<CreatePhotoPermissionRequest>,
) -> HttpResponse {
    let requests_collection = db.collection::<mongodb::bson::Document>("photo_permission_requests");

    let now = chrono::Utc::now().to_rfc3339();

    let doc = doc! {
        "requester_username": &req.requester_username,
        "target_username": &req.target_username,
        "status": "pending",
        "timestamp": &now,
        "response_timestamp": null,
        "response_message": null,
        "permission_expires_at": null,
    };

    match requests_collection.insert_one(doc, None).await {
        Ok(result) => {
            HttpResponse::Created().json(ApiResponse::ok(json!({
                "id": result.inserted_id.to_string(),
                "message": "Demande de permission photos créée"
            })))
        }
        Err(e) => {
            log::error!("Erreur MongoDB: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::err(format!("Erreur: {}", e)))
        }
    }
}

// 4. POST /api/requests/event-participation
#[derive(serde::Deserialize)]
pub struct CreateEventParticipationRequest {
    pub requester_username: String,
    pub event_id: String,
    pub event_name: String,
    pub event_creator: String,
    pub participation_role: Option<String>,
}

pub async fn create_event_participation_request(
    db: web::Data<Database>,
    req: web::Json<CreateEventParticipationRequest>,
) -> HttpResponse {
    let requests_collection = db.collection::<mongodb::bson::Document>("event_participation_requests");

    let now = chrono::Utc::now().to_rfc3339();

    let doc = doc! {
        "requester_username": &req.requester_username,
        "event_id": &req.event_id,
        "event_name": &req.event_name,
        "event_creator": &req.event_creator,
        "status": "pending",
        "timestamp": &now,
        "response_timestamp": null,
        "response_message": null,
        "participation_role": req.participation_role.as_deref().unwrap_or("participant"),
    };

    match requests_collection.insert_one(doc, None).await {
        Ok(result) => {
            HttpResponse::Created().json(ApiResponse::ok(json!({
                "id": result.inserted_id.to_string(),
                "message": "Demande de participation créée"
            })))
        }
        Err(e) => {
            log::error!("Erreur MongoDB: {}", e);
            HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::err(format!("Erreur: {}", e)))
        }
    }
}

