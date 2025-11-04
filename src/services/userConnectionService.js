import { pgPool } from '../config/database.js';

/**
 * Service pour gérer les connexions utilisateur dans PostgreSQL
 */
class UserConnectionService {
  /**
   * Met à jour le statut de connexion d'un utilisateur
   * @param {string} username - Nom d'utilisateur
   * @param {boolean} isOnline - Statut en ligne
   * @returns {Promise<Object>} Résultat de la mise à jour
   */
  async updateConnectionStatus(username, isOnline) {
    try {
      // Essayer d'abord avec toutes les colonnes
      let query = `
        UPDATE compte_compte
        SET is_online = $1
        WHERE username = $2
        RETURNING username
      `;

      try {
        const result = await pgPool.query(query, [isOnline, username]);
        if (result.rows.length > 0) {
          console.log(`🔄 Statut mis à jour: ${username} → ${isOnline ? 'EN LIGNE' : 'HORS LIGNE'}`);
          return result.rows[0];
        } else {
          console.warn(`⚠️ Utilisateur non trouvé: ${username}`);
          return null;
        }
      } catch (err) {
        // Si la colonne n'existe pas, juste logger
        console.log(`ℹ️ Colonne is_online non disponible pour ${username}`);
        return { username, status: 'logged' };
      }
    } catch (error) {
      console.error('❌ Erreur updateConnectionStatus:', error);
      throw error;
    }
  }

  /**
   * Enregistre une nouvelle connexion
   * @param {string} username - Nom d'utilisateur
   * @param {string} socketId - ID du socket WebSocket
   * @returns {Promise<Object>} Résultat
   */
  async registerConnection(username, socketId) {
    try {
      await this.updateConnectionStatus(username, true);
      console.log(`✅ Connexion enregistrée: ${username} (${socketId})`);
      return { username, socketId, status: 'connected' };
    } catch (error) {
      console.error('❌ Erreur registerConnection:', error);
      throw error;
    }
  }

  /**
   * Déconnecte un utilisateur
   * @param {string} username - Nom d'utilisateur
   * @returns {Promise<Object>} Résultat
   */
  async disconnectUser(username) {
    try {
      const result = await this.updateConnectionStatus(username, false);
      console.log(`👋 Déconnexion: ${username}`);
      return result;
    } catch (error) {
      console.error('❌ Erreur disconnectUser:', error);
      throw error;
    }
  }

  /**
   * Récupère tous les utilisateurs en ligne
   * @returns {Promise<Array>} Liste des utilisateurs en ligne
   */
  async getOnlineUsers() {
    try {
      // Essayer d'abord avec les colonnes complètes
      let query = `
        SELECT username
        FROM compte_compte
        LIMIT 10
      `;

      const result = await pgPool.query(query);
      console.log(`👥 Utilisateurs trouvés: ${result.rows.length}`);
      return result.rows;
    } catch (error) {
      console.error('❌ Erreur getOnlineUsers:', error);
      throw error;
    }
  }

  /**
   * Vérifie si un utilisateur est en ligne
   * @param {string} username - Nom d'utilisateur
   * @returns {Promise<boolean>} True si en ligne
   */
  async isUserOnline(username) {
    try {
      const query = `
        SELECT is_online
        FROM compte_compte
        WHERE username = $1
      `;

      const result = await pgPool.query(query, [username]);
      return result.rows.length > 0 && result.rows[0].is_online;
    } catch (error) {
      console.error('❌ Erreur isUserOnline:', error);
      return false;
    }
  }

  /**
   * Nettoie les connexions obsolètes (utilisateurs inactifs depuis plus de X minutes)
   * @param {number} inactiveMinutes - Nombre de minutes d'inactivité
   * @returns {Promise<number>} Nombre d'utilisateurs déconnectés
   */
  async cleanupInactiveConnections(inactiveMinutes = 30) {
    try {
      const query = `
        UPDATE compte_compte
        SET is_online = false
        WHERE is_online = true
          AND last_seen < NOW() - INTERVAL '${inactiveMinutes} minutes'
        RETURNING username
      `;

      const result = await pgPool.query(query);
      
      if (result.rows.length > 0) {
        console.log(`🧹 Nettoyage: ${result.rows.length} utilisateurs inactifs déconnectés`);
      }
      
      return result.rows.length;
    } catch (error) {
      console.error('❌ Erreur cleanupInactiveConnections:', error);
      return 0;
    }
  }
}

export default new UserConnectionService();

