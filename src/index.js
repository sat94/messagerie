import express from 'express';
import cors from 'cors';
import dotenv from 'dotenv';
import { connectMongoDB, testPostgresConnection, closeConnections } from './config/database.js';
import messageRoutes from './routes/messageRoutes.js';
import userConnectionRoutes from './routes/userConnectionRoutes.js';

// Charger les variables d'environnement
dotenv.config();

const app = express();
const PORT = process.env.PORT || 3000;

// Middleware
app.use(cors());
app.use(express.json());
app.use(express.urlencoded({ extended: true }));

// Routes API
app.use('/api/messages', messageRoutes);
app.use('/api/users', userConnectionRoutes);

// Route de santé
app.get('/health', (req, res) => {
  res.json({
    status: 'OK',
    timestamp: new Date().toISOString()
  });
});

// Route d'accueil
app.get('/', (req, res) => {
  res.json({
    name: 'Messagerie Rush API',
    version: '1.0.0',
    endpoints: {
      health: '/health',
      messages: {
        history: 'GET /api/messages/history/:username?limit=100',
        conversation: 'GET /api/messages/conversation/:user1/:user2?limit=100',
        send: 'POST /api/messages/send'
      },
      users: {
        connect: 'POST /api/users/connect',
        disconnect: 'POST /api/users/disconnect',
        online: 'GET /api/users/online',
        status: 'GET /api/users/status/:username'
      }
    }
  });
});

// Gestion des erreurs 404
app.use((req, res) => {
  res.status(404).json({
    success: false,
    error: 'Route non trouvée'
  });
});

// Gestion des erreurs globales
app.use((err, req, res, next) => {
  console.error('❌ Erreur serveur:', err);
  res.status(500).json({
    success: false,
    error: 'Erreur interne du serveur'
  });
});

// Fonction de démarrage
async function startServer() {
  try {
    console.log('🚀 Démarrage du serveur...\n');

    // Connexion à MongoDB
    console.log('📦 Connexion à MongoDB...');
    await connectMongoDB();

    // Test de connexion PostgreSQL
    console.log('📦 Test de connexion PostgreSQL...');
    await testPostgresConnection();

    // Démarrage du serveur HTTP
    app.listen(PORT, () => {
      console.log(`\n✅ Serveur HTTP démarré sur http://localhost:${PORT}`);
    });

    console.log('\n🎉 Système de messagerie prêt !\n');
    console.log('📝 Endpoints disponibles:');
    console.log(`   - API REST: http://localhost:${PORT}`);
    console.log('\n💡 Utilisez GET / pour voir tous les endpoints disponibles\n');

  } catch (error) {
    console.error('❌ Erreur lors du démarrage:', error);
    process.exit(1);
  }
}

// Gestion de l'arrêt propre
process.on('SIGINT', async () => {
  console.log('\n\n🛑 Arrêt du serveur...');
  await closeConnections();
  process.exit(0);
});

process.on('SIGTERM', async () => {
  console.log('\n\n🛑 Arrêt du serveur...');
  await closeConnections();
  process.exit(0);
});

// Démarrer le serveur
startServer();

