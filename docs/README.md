# Base de connaissances

## Pour utiliser l'outil

Ces documents sont écrits pour la naturaliste, sans rien de technique.

Première fois : commencez par la [première soirée](premiere-soiree.md).
Vous relisez les résultats d'une soirée : passez à [lire les résultats](lire-les-resultats.md).

| Document | Contenu |
| --- | --- |
| [premiere-soiree.md](premiere-soiree.md) | La première soirée pas à pas : installer, brancher les jumelles, copier les vidéos, lancer, attendre, ouvrir les résultats |
| [lire-les-resultats.md](lire-les-resultats.md) | Ce que l'outil ne voit pas, conseils de prise de vue, comment lire l'image résumé, juger un extrait et noter son verdict, et les mots de l'image résumé, du tableau des pistes et des extraits |

## Pour comprendre ou modifier l'outil

Ces documents disent ce que le code ne dit pas : pourquoi chaque réglage a sa valeur, ce qu'on a mesuré et ce qui reste à trancher.

| Document | Pour qui | Contenu |
| --- | --- | --- |
| [01-contexte.md](01-contexte.md) | Naturaliste et développeur | Le besoin, les vidéos, ce qu'on sait de la prise de vue |
| [02-methode.md](02-methode.md) | Développeur | L'algorithme pas à pas et la justification de chaque réglage |
| [03-resultats-092.md](03-resultats-092.md) | Développeur | Les pistes de la vidéo de référence et leur statut |
| [04-outils-existants.md](04-outils-existants.md) | Développeur | DVR-Scan et les autres options étudiées, avec mesures |
| [05-outillage.md](05-outillage.md) | Développeur | L'outillage du dépôt : mise, hooks, CI, langue des messages, Homebrew |
| [06-questions-ouvertes.md](06-questions-ouvertes.md) | Naturaliste et développeur | Ce qui reste à trancher ou à valider |
| [07-ameliorer-la-detection.md](07-ameliorer-la-detection.md) | Développeur | Diagnostic mesuré des pistes manquées et catalogue des améliorations (3 octobre 2026) |
| [08-banc-de-mesure.md](08-banc-de-mesure.md) | Développeur | Le banc de cibles synthétiques : commande, méthode, métriques, limites, puis les mesures de la v0.1.1 |
| [09-profilage.md](09-profilage.md) | Développeur | Où part le temps de calcul : étapes, points chauds, fils |
| [10-logique-de-detection.md](10-logique-de-detection.md) | Naturaliste et développeur | La logique expliquée : outil mathématique de chaque étape, choix et limites |
| [11-migration-rust.md](11-migration-rust.md) | Développeur | Le Rust : chaîne de construction, licences, lints, charte, dépendances, tests, encodage, crates, réglages, références v0.1.1 et v0.2.0 |
| [images/README.md](images/README.md) | Développeur | D'où vient chaque image des docs pour la naturaliste, et comment les refaire |
| [tools/docs-images.sh](../tools/docs-images.sh) | Développeur | Le script qui refait toutes les images de `docs/images/` depuis l'extrait de test |

## Qui tient quoi

Chaque vérité s'écrit à un seul endroit. Les autres documents y renvoient.
Un changement de comportement du code met à jour ce document-là, dans la même série de changements.

| Vérité | Document | Ailleurs |
| --- | --- | --- |
| Valeur de chaque réglage et sa justification mesurée | [02](02-methode.md) | 10 cite le défaut et renvoie à 02 ; le README n'en recopie aucun (`batdetect --help` affiche les défauts depuis le code) |
| Logique étape par étape, module de chaque étape | [10](10-logique-de-detection.md) | aucun |
| Chaîne de construction, charte, tests, licences, encodage | [11](11-migration-rust.md), sections Construction à Plage réelle des sources | 05 et 02 y renvoient |
| Détail des crates | [11](11-migration-rust.md#crates), qui suit les lots | la carte courte dans CLAUDE.md |
| Carte courte des crates, règles des agents | [CLAUDE.md](../CLAUDE.md) | aucun |
| Temps de calcul actuels | [09](09-profilage.md) | les autres citent ses chiffres |
| Questions ouvertes | [06](06-questions-ouvertes.md) | aucun |
| Utilisation | [README](../README.md) | les détails dans premiere-soiree et lire-les-resultats |
| Fichiers produits | [lire-les-resultats](lire-les-resultats.md#où-sont-les-résultats) | `params.json` et le ménage des sorties, pour le développeur, dans [02](02-methode.md#sorties) |
| Construire, vérifier, publier une version | [CONTRIBUTING](../CONTRIBUTING.md) | le détail de la CI et de la distribution dans [05](05-outillage.md) |
| Provenance et fabrication des images des docs | [images/README](images/README.md) et [tools/docs-images.sh](../tools/docs-images.sh) | les docs pour la naturaliste n'en disent rien |
| Parcours d'une soirée, des jumelles aux résultats, pour qui n'a jamais lancé l'outil | [premiere-soiree](premiere-soiree.md) | le README y renvoie |
| Lecture de l'image résumé : bandeau, trait, flèche, pastille, zone masquée, frise du temps, couleurs | [lire-les-resultats](lire-les-resultats.md#lire-limage-résumé) | le README y renvoie |
| Lecture d'un extrait, façon de désigner une piste et de noter son verdict, faux positifs observés | [lire-les-resultats](lire-les-resultats.md#juger-un-extrait) | le dessin des extraits et ses réglages sont dans [02](02-methode.md#extrait-zoomé-et-ralenti) |
| Histoire du dépôt | git et le [CHANGELOG](../CHANGELOG.md) ; les mesures datées qui justifient un réglage restent dans 07, 08 et 09 | aucun |
