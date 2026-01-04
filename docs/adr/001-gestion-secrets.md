# ADR-001 : Gestion des secrets et credentials

**Date** : 2026-01-04
**Statut** : Accepté

## Contexte

Flux doit se connecter à des providers externes (GitLab, GitHub) pour tracker l'activité de code review. Ces connexions nécessitent des tokens d'authentification et des identifiants utilisateur.

Le premier design proposait de stocker ces credentials directement dans `~/.config/flux/config.toml`, ce qui pose des risques de sécurité :
- Commit accidentel du fichier de config
- Exposition via backup cloud
- Lecture par des applications tierces

## Décision

Séparer la configuration publique des secrets avec une stratégie de résolution en cascade :

1. **Variables d'environnement** (priorité maximale)
   - `FLUX_GITLAB_TOKEN`, `FLUX_GITLAB_USER_ID`
   - `FLUX_GITHUB_TOKEN`, `FLUX_GITHUB_USER_ID`

2. **Fichier secrets.toml** (fallback)
   - `~/.config/flux/secrets.toml`
   - Permissions restrictives (chmod 600)
   - Jamais versionné

3. **Config publique** (versionnable)
   - `~/.config/flux/config.toml`
   - Contient uniquement `base_url` pour les providers
   - Aucun secret

## Alternatives considérées

| Alternative | Avantages | Inconvénients |
|-------------|-----------|---------------|
| Token dans config.toml | Simple, un seul fichier | Risque de commit, pas de séparation |
| Keyring système (libsecret) | Plus sécurisé | Dépendance lourde, cross-platform complexe |
| `token_command` uniquement | Délègue au password manager | Requiert setup utilisateur |
| Variables d'env uniquement | Standard CI/CD | Pas pratique pour dev local |

## Conséquences

### Positives
- Pattern standard (même approche que gh, glab, docker CLI)
- Compatible CI/CD via env vars
- Fichier secrets.toml pour convenience en dev
- Skill `/security` peut scanner pour détecter les fuites

### Négatives
- Deux fichiers à gérer (config.toml + secrets.toml)
- L'utilisateur doit comprendre la séparation

## Notes

- Règle ajoutée dans CLAUDE.md : jamais de token en clair dans le code
- Skill `/security` créé pour scanner avant commit
- Commande `/security-scan` pour audit du repo
