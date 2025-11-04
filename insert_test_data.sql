-- Mettre à jour les données de test pour Vero
UPDATE compte_compte
SET
  prenom = 'Samantha',
  date_de_naissance = '1995-05-15'::date,
  audio = 'https://example.com/photo-vero.jpg'
WHERE username = 'Vero';

-- Ajouter une photo principale pour Vero
INSERT INTO compte_photo (photos, type_photo, ordre, date_ajout, est_active, is_nsfw_checked, is_nsfw, is_shocking_checked, is_shocking, compte_id)
SELECT
  'https://example.com/vero-main-photo.jpg',
  'main',
  1,
  NOW(),
  true,
  false,
  false,
  false,
  false,
  cc.id
FROM compte_compte cc
WHERE cc.username = 'Vero'
ON CONFLICT (compte_id, type_photo) WHERE type_photo = 'main' DO UPDATE SET
  photos = 'https://example.com/vero-main-photo.jpg';

