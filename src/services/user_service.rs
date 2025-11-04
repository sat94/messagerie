use tokio_postgres::Client;
use log::info;

pub struct UserService;

impl UserService {
    /// Met à jour le statut de connexion d'un utilisateur (INSERT OR UPDATE)
    pub async fn update_connection_status(
        client: &Client,
        username: &str,
        is_online: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // UPSERT: INSERT si n'existe pas, UPDATE sinon
        let query = "
            INSERT INTO compte_compte (username, is_online, connected_at, last_seen)
            VALUES ($1, $2, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            ON CONFLICT (username) DO UPDATE
            SET is_online = $2, last_seen = CURRENT_TIMESTAMP
        ";

        match client.execute(query, &[&username, &is_online]).await {
            Ok(_) => {
                info!("✅ Statut mis à jour: {} → {}", username, if is_online { "EN LIGNE" } else { "HORS LIGNE" });
                Ok(())
            }
            Err(e) => {
                info!("❌ Erreur mise à jour statut: {}", e);
                Err(Box::new(e))
            }
        }
    }

    /// Récupère tous les utilisateurs en ligne
    pub async fn get_online_users(
        client: &Client,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let query = "
            SELECT username
            FROM compte_compte
            WHERE is_online = true
            ORDER BY username
        ";

        match client.query(query, &[]).await {
            Ok(rows) => {
                let users: Vec<String> = rows.iter()
                    .map(|row| row.get(0))
                    .collect();
                info!("👥 Utilisateurs en ligne: {}", users.len());
                Ok(users)
            }
            Err(e) => {
                info!("ℹ️ Erreur récupération utilisateurs: {}", e);
                Ok(Vec::new())
            }
        }
    }

    /// Vérifie si un utilisateur est en ligne
    pub async fn is_user_online(
        client: &Client,
        username: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let query = "
            SELECT is_online
            FROM compte_compte
            WHERE username = $1
        ";

        match client.query_opt(query, &[&username]).await {
            Ok(Some(row)) => {
                let is_online: bool = row.get(0);
                Ok(is_online)
            }
            Ok(None) => Ok(false),
            Err(_) => Ok(false),
        }
    }

    /// Récupère tous les utilisateurs
    pub async fn get_all_users(
        client: &Client,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let query = "
            SELECT username
            FROM compte_compte
            ORDER BY username
        ";

        match client.query(query, &[]).await {
            Ok(rows) => {
                let users: Vec<String> = rows.iter()
                    .map(|row| row.get(0))
                    .collect();
                info!("👥 Total utilisateurs: {}", users.len());
                Ok(users)
            }
            Err(e) => {
                info!("ℹ️ Erreur récupération utilisateurs: {}", e);
                Ok(Vec::new())
            }
        }
    }
}

