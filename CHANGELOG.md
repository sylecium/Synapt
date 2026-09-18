# Journal des modifications

Toutes les modifications notables apportées au projet Synapt sont consignées dans ce fichier.

Le format est basé sur [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/) et ce projet adhère à la norme [Semantic Versioning](https://semver.org/lang/fr/).

---

## [Non publié]

### Nouveautés
- *(Aucune nouveauté pour le moment)*

---

## [0.1.5] - 2026-09-18

### Nouveautés
- Tableaux dans les notes : insertion par défaut mesurée, ajout de lignes et colonnes en un clic ou par clic droit, et redimensionnement à la souris avec mémorisation durable des largeurs.
- Raccourcis clavier d'écriture : formatage rapide (gras, italique, souligné, titres, listes) via les raccourcis usuels (Ctrl+B, Ctrl+I, Ctrl+U, etc.), avec rappel de la combinaison au survol des boutons.

### Améliorations
- Fluidité et réactivité : chargement accéléré du tableau de bord et de l'agenda grâce à une gestion optimisée de la base de données locale.
- Navigation d'agenda : rafraîchissement fluide et ciblé lors de la création ou du déplacement de rendez-vous, sans rechargement global.
- Réglages du cabinet : enregistrement instantané des coordonnées et mentions du cabinet, avec test direct de notification vers le téléphone.
- Sécurisation de fermeture : sauvegarde automatique de toutes les notes en attente dès le changement d'écran ou la fermeture de l'application.

### Corrections
- Conflit de raccourci : la combinaison Ctrl+B applique désormais le texte en gras sans replier le menu latéral de l'application.
- Notes d'honoraires PDF : fiabilisation de la mise en page (adresses sur plusieurs lignes, caractères accentués) et gestion rigoureuse de la numérotation.
- Annulation de facture : suppression sécurisée du fichier PDF associé en cas d'annulation d'une note d'honoraires.

---

## [0.1.4] - 2026-09-17

### Nouveautés
- Suppression de tarif : possibilité de retirer un tarif depuis la liste ou par clic droit, avec vérification préalable qu'aucun rendez-vous prévu ne l'utilise.

### Améliorations
- Mémorisation de l'affichage : la fenêtre se souvient de sa taille et de sa position entre chaque ouverture.

### Corrections
- Tarifs personnalisés : la création d'un tarif accepte désormais n'importe quel montant saisi au lieu de se réinitialiser à 50,00 €.

---

## [0.1.3] - 2026-09-17

### Nouveautés
- Rendez-vous sans client : possibilité de bloquer un créneau dans l'agenda sans désigner de fiche client. La mention « Sans client » s'affiche et un client peut être rattaché par la suite.

---

## [0.1.2] - 2026-09-17

### Nouveautés
- Notes d'honoraires : création de reçus et factures au format PDF (TVA 20 %) pour une ou plusieurs séances, avec numéro figé et mention acquittée, directement accessibles dans le dossier `Synapt/honoraires`.
- Saisie des tarifs : choix de saisie du montant en HT ou TTC pour chaque tarif configuré.

### Corrections
- Enregistrement des notes : fiabilisation de la sauvegarde automatique des notes saisies dans le volet de rendez-vous.
- Fiches clients : boîte de dialogue de confirmation avant la suppression définitive d'un client.

---

## [0.1.1] - 2026-09-16

### Nouveautés
- Fiches clients détaillées : ajout du statut de suivi, du tarif habituel, de la fréquence, de l'orientation, du contact d'urgence et de l'adresse de facturation.
- Menu d'ajout de champs : possibilité d'ajouter des informations optionnelles à la volée sur une fiche client.

### Améliorations
- Saisie de la date de naissance dans un champ unique `jj/mm/aaaa`.
- Sélection de date de rendez-vous avec calendrier et possibilité d'effacer la sélection.

---

## [0.1.0] - 2026-09-12

### Nouveautés
- Lancement initial de Synapt pour cabinet psy : tableau de bord d'accueil, agenda avec repère visuel de l'heure, suivi des clients, gestion des tarifs et espace de notes riches.
- Rappels et notifications discrètes vers mobile via ntfy.
- Mises à jour automatiques de l'application au format paquet Debian (.deb).

[Non publié]: https://github.com/sylecium/Synapt/compare/v0.1.5...HEAD
[0.1.5]: https://github.com/sylecium/Synapt/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/sylecium/Synapt/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/sylecium/Synapt/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/sylecium/Synapt/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/sylecium/Synapt/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/sylecium/Synapt/releases/tag/v0.1.0
