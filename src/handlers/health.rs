use actix_web::HttpResponse;
use serde_json::json;

pub async fn index() -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "name": "Messagerie Rush API",
        "version": "1.0.0",
        "framework": "Actix-web (Rust)",
        "endpoints": {
            "health": "/health",
            "messages": {
                "history": "GET /api/messages/history/{username}?limit=100",
                "conversation": "GET /api/messages/conversation/{user1}/{user2}?limit=100",
                "send": "POST /api/messages/send"
            },
            "users": {
                "connect": "POST /api/users/connect",
                "disconnect": "POST /api/users/disconnect",
                "online": "GET /api/users/online",
                "status": "GET /api/users/status/{username}"
            }
        }
    }))
}

pub async fn health() -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "status": "OK",
        "timestamp": chrono::Utc::now()
    }))
}

