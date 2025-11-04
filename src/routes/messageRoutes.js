import express from 'express';
import messageService from '../services/messageService.js';
import userConnectionService from '../services/userConnectionService.js';

const router = express.Router();

/**
 * GET /api/messages/history/:username
 * Récupère l'historique des messages pour un utilisateur
 */
router.get('/history/:username', async (req, res) => {
  try {
    const { username } = req.params;
    const limit = parseInt(req.query.limit) || 100;

    const messages = await messageService.getMessageHistory(username, limit);

    res.json({
      success: true,
      data: {
        username,
        messages,
        count: messages.length
      }
    });
  } catch (error) {
    console.error('❌ Erreur /history:', error);
    res.status(500).json({
      success: false,
      error: 'Erreur lors de la récupération de l\'historique'
    });
  }
});

/**
 * GET /api/messages/conversation/:user1/:user2
 * Récupère la conversation entre deux utilisateurs
 */
router.get('/conversation/:user1/:user2', async (req, res) => {
  try {
    const { user1, user2 } = req.params;
    const limit = parseInt(req.query.limit) || 100;

    const messages = await messageService.getConversation(user1, user2, limit);

    res.json({
      success: true,
      data: {
        participants: [user1, user2],
        messages,
        count: messages.length
      }
    });
  } catch (error) {
    console.error('❌ Erreur /conversation:', error);
    res.status(500).json({
      success: false,
      error: 'Erreur lors de la récupération de la conversation'
    });
  }
});

/**
 * POST /api/messages/send
 * Envoie un message (alternative à WebSocket)
 */
router.post('/send', async (req, res) => {
  try {
    const { sender, recipient, content, type = 'text' } = req.body;

    if (!sender || !recipient || !content) {
      return res.status(400).json({
        success: false,
        error: 'sender, recipient et content sont requis'
      });
    }

    const message = await messageService.saveMessage({
      sender,
      recipient,
      content,
      type
    });

    res.json({
      success: true,
      data: message
    });
  } catch (error) {
    console.error('❌ Erreur /send:', error);
    res.status(500).json({
      success: false,
      error: 'Erreur lors de l\'envoi du message'
    });
  }
});

/**
 * PUT /api/messages/mark-read
 * Marque les messages comme lus
 */
router.put('/mark-read', async (req, res) => {
  try {
    const { sender, recipient } = req.body;

    if (!sender || !recipient) {
      return res.status(400).json({
        success: false,
        error: 'sender et recipient sont requis'
      });
    }

    const count = await messageService.markAsRead(sender, recipient);

    res.json({
      success: true,
      data: { count }
    });
  } catch (error) {
    console.error('❌ Erreur /mark-read:', error);
    res.status(500).json({
      success: false,
      error: 'Erreur lors du marquage des messages'
    });
  }
});

/**
 * GET /api/users/online
 * Récupère la liste des utilisateurs en ligne
 */
router.get('/users/online', async (req, res) => {
  try {
    const users = await userConnectionService.getOnlineUsers();

    res.json({
      success: true,
      data: {
        users,
        count: users.length
      }
    });
  } catch (error) {
    console.error('❌ Erreur /users/online:', error);
    res.status(500).json({
      success: false,
      error: 'Erreur lors de la récupération des utilisateurs en ligne'
    });
  }
});

/**
 * GET /api/users/status/:username
 * Vérifie si un utilisateur est en ligne
 */
router.get('/users/status/:username', async (req, res) => {
  try {
    const { username } = req.params;
    const isOnline = await userConnectionService.isUserOnline(username);

    res.json({
      success: true,
      data: {
        username,
        isOnline
      }
    });
  } catch (error) {
    console.error('❌ Erreur /users/status:', error);
    res.status(500).json({
      success: false,
      error: 'Erreur lors de la vérification du statut'
    });
  }
});

export default router;

