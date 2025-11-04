-- Mettre à jour les données de test pour Vero
UPDATE compte_compte
SET
  prenom = 'Samantha',
  date_de_naissance = '1995-05-15'::date,
  audio = 'https://example.com/photo-vero.jpg'
WHERE username = 'Vero';

