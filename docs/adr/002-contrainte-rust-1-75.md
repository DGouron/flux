# ADR-002 : Contrainte Rust 1.75 et pinning des dépendances

**Date** : 2026-01-04
**Statut** : Accepté

## Contexte

Le projet cible Rust 1.75 (version stable disponible sur la plupart des distributions Linux). Plusieurs dépendances transitives récentes requièrent des versions plus récentes de Rust :

- `indexmap >= 2.7` requiert Rust 1.82
- `zbus >= 5` requiert Rust 1.77
- `notify-rust >= 4.11` dépend de `zbus 5`

Ces incompatibilités cassent le build sur Rust 1.75.

## Décision

Pinner les dépendances problématiques à des versions compatibles Rust 1.75 :

```bash
# indexmap
cargo update indexmap@2.12.1 --precise 2.5.0

# notify-rust (utiliser 4.10 au lieu de 4.11)
notify-rust = "4.10"
```

Documenter ces contraintes et les mettre à jour quand la version minimale de Rust sera relevée.

## Alternatives considérées

| Alternative | Avantages | Inconvénients |
|-------------|-----------|---------------|
| Upgrader à Rust 1.82+ | Accès aux dernières versions | Exclut les users sur distros stables |
| Pinner les dépendances | Compatible Rust 1.75 | Maintenance manuelle, features manquantes |
| Fork des dépendances | Contrôle total | Maintenance lourde |
| Remplacer notify-rust | Évite zbus | Ré-implémenter les notifications |

## Conséquences

### Positives
- Build sur Rust 1.75 (Debian stable, Ubuntu LTS)
- Pas de fork à maintenir
- Solution simple et réversible

### Négatives
- Versions de dépendances figées
- Potentielles CVE non corrigées dans vieilles versions
- Doit être réévalué régulièrement

## Notes

Dépendances pinnées actuelles :
- `indexmap` : 2.5.0 (au lieu de 2.12.1)
- `notify-rust` : =4.8.0 (au lieu de 4.11.7) → évite zbus 5
- `ureq` : 2.9.1 (au lieu de 2.12.1)
- `url` : 2.5.0 (au lieu de 2.5.7) → évite icu_* crates

À réévaluer quand :
- La MSRV (Minimum Supported Rust Version) sera relevée
- Une CVE affecte une dépendance pinnée
