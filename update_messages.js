// Ajouter is_connect: false à tous les messages existants
db.messages.updateMany(
  { is_connect: { $exists: false } },
  { $set: { is_connect: false } }
);

// Afficher les messages
db.messages.find().pretty();

