# Convention de release

Ce document décrit la convention de versionnement, la rédaction des notes de version et la procédure de publication de Synapt.

Elle s'appuie sur deux standards ouverts de l'industrie :
- **Semantic Versioning 2.0.0** (SemVer) pour la numérotation des versions.
- **Keep a Changelog 1.1.0** pour la structure et la catégorisation des changements.

---

## 1. Versionnement sémantique (SemVer)

Les numéros de version suivent le format strict `MAJOR.MINOR.PATCH` (ex. `0.1.4`) :

- **MAJOR (Majeure)** : rupture de compatibilité (ex. migration majeure du schéma de base de données non rétrocompatible, changement profond de l'architecture locale).
- **MINOR (Mineure)** : ajout de fonctionnalités rétrocompatibles (ex. nouvel écran, nouveau flux métier comme les notes d'honoraires ou les rendez-vous sans client).
- **PATCH (Correctif)** : corrections d'anomalies, ajustements cosmétiques ou ergonomiques rétrocompatibles (ex. correction d'enregistrement d'une note, mémorisation de la géométrie de fenêtre).

Les étiquettes Git (tags) reprennent le numéro précédé d'un `v` : `vX.Y.Z` (ex. `v0.1.4`).

---

## 2. Rôles des fichiers

Synapt utilise deux fichiers Markdown complémentaires :

| Fichier | Emplacement | Rôle | Cible |
|---|---|---|---|
| `CHANGELOG.md` | Racine du dépôt | Historique complet et cumulatif de toutes les versions publiées et section en cours (`[Non publié]`). | Équipe de développement, suivi du projet. |
| `.github/release-body.md` | `.github/` | Contenu de la version en cours de publication uniquement. | Recopié automatiquement par GitHub Actions dans la GitHub Release et affiché dans l'updater de l'application. |

---

## 3. Catégories standard (Keep a Changelog)

Les modifications sont groupées selon les catégories suivantes (omettre les sections vides) :

- `### Nouveautés` : nouvelles fonctionnalités offertes au cabinet.
- `### Améliorations` : évolutions, simplifications ou perfectionnements de fonctionnalités existantes.
- `### Corrections` : résolutions de dysfonctionnements ou de comportements inattendus.
- `### Suppressions` : fonctionnalités retirées ou remplacées.
- `### Sécurité` : correctifs de sécurité ou de robustesse des données locales.

---

## 4. Règles de rédaction

Les notes de version s'adressent au praticien dans son utilisation quotidienne du cabinet :

1. **Orientation cabinet** : expliquer ce que le changement apporte concrètement à l'activité du cabinet, et non la mise en oeuvre technique interne (éviter le jargon SQL, IPC, Rust, Svelte, CI/CD).
2. **Ton sobre et concis** : phrases courtes, informatives et directes.
3. **Zéro emoji** : aucun emoji dans les titres, textes ou puces.
4. **Pas de tiret cadratin** : employer exclusivement le tiret simple `-` ou des parenthèses, jamais le tiret cadratin (`—`).
5. **Formatage Markdown** :
   - Titre de niveau 2 pour la version : `## [X.Y.Z] - AAAA-MM-JJ` (ou résumé en une ligne pour `release-body.md`).
   - Titre de niveau 3 pour les catégories (`### Nouveautés`, `### Corrections`, etc.).
   - Listes à puces Markdown (`- `) pour chaque point notable.

---

## 5. Procédure de publication (Checklist)

Pour publier une nouvelle version :

### Étape 1 : Préparation et vérifications locales
- S'assurer que la branche `main` est propre et à jour.
- Exécuter les vérifications de types et tests :
  ```bash
  bun run check
  bun test
  ```

### Étape 2 : Alignement des numéros de version
Mettre à jour le numéro de version `X.Y.Z` de manière synchronisée dans les 3 fichiers suivants :
- `package.json` (`"version": "X.Y.Z"`)
- `src-tauri/tauri.conf.json` (`"version": "X.Y.Z"`)
- `src-tauri/Cargo.toml` (`version = "X.Y.Z"`)

### Étape 3 : Rédaction des notes de version
- Mettre à jour `CHANGELOG.md` :
  - Transférer les entrées de la section `[Non publié]` vers une nouvelle section `## [X.Y.Z] - AAAA-MM-JJ`.
  - Mettre à jour les liens de comparaison en bas de fichier.
- Rédiger le contenu de la release dans `.github/release-body.md` (uniquement le texte destiné à cette version, sans l'en-tête de version déjà fourni par GitHub).

### Étape 4 : Commit de release
Créer le commit regroupant la montée de version et les notes :
```bash
git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml CHANGELOG.md .github/release-body.md
git commit -m "chore(release): publier X.Y.Z"
```

### Étape 5 : Création du tag et déclenchement du déploiement
Créer l'étiquette Git annotée et pousser sur le dépôt distant :
```bash
git tag vX.Y.Z
git push origin main
git push origin vX.Y.Z
```

### Étape 6 : Contrôle du workflow
- Suivre l'exécution du workflow GitHub Actions `Release`.
- Vérifier la génération du paquet `.deb`, du fichier `latest.json` et la bonne intégration du texte de `.github/release-body.md` dans la release GitHub.
