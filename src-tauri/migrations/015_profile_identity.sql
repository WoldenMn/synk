-- Migration 015 : donnees d'identite de la carte chiffree.
-- user_profile est une vraie table (pas une vue de retrocompat), donc
-- ALTER TABLE ADD COLUMN passe sans risque en migration versionnee.
ALTER TABLE user_profile ADD COLUMN birth_date TEXT;
ALTER TABLE user_profile ADD COLUMN handedness TEXT;
ALTER TABLE user_profile ADD COLUMN eye_color TEXT;
ALTER TABLE user_profile ADD COLUMN hair_color TEXT;
ALTER TABLE user_profile ADD COLUMN blood_group TEXT;
ALTER TABLE user_profile ADD COLUMN allergies TEXT;
ALTER TABLE user_profile ADD COLUMN vision_correction TEXT;
ALTER TABLE user_profile ADD COLUMN medical_history TEXT;
ALTER TABLE user_profile ADD COLUMN vaccines TEXT;
