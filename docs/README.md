# Base de connaissances

## Pour utiliser l'outil

Ces documents s'adressent à la naturaliste et ne contiennent rien de technique : ce qui sert à
les refaire est rangé côté développeur, plus bas.

Première fois : commencez par la [première soirée](premiere-soiree.md). Vous relisez les
résultats d'une soirée : passez à [lire les résultats](lire-les-resultats.md).

| Document | Contenu |
| --- | --- |
| [premiere-soiree.md](premiere-soiree.md) | La première soirée pas à pas : installer, brancher les jumelles, copier les vidéos, lancer, attendre, ouvrir les résultats |
| [lire-les-resultats.md](lire-les-resultats.md) | Ce que l'outil ne voit pas, conseils de prise de vue, comment lire l'image résumé, juger un extrait et noter son verdict, et les mots de l'image résumé, du tableau des pistes et des extraits |

## Pour comprendre ou modifier l'outil

Ce dossier garde ce qu'on a appris en construisant `batdetect` : ce que le code ne dit
pas, et pourquoi les choses sont comme elles sont.

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
| [09-profilage.md](09-profilage.md) | Développeur | Où part le temps de calcul du Rust : étapes, points chauds, fils ; puis le profil de la v0.1.1 en Python |
| [10-logique-de-detection.md](10-logique-de-detection.md) | Naturaliste et développeur | La logique expliquée : outil mathématique de chaque étape, choix et limites |
| [11-migration-rust.md](11-migration-rust.md) | Développeur | La réécriture en Rust : bilan, chaîne de construction, licences, charte, tests, encodage, et l'histoire de la migration |
| [images/README.md](images/README.md) | Développeur | D'où vient chaque image des docs pour la naturaliste, et comment les refaire |
| [tools/docs-images.sh](../tools/docs-images.sh) | Développeur | Le script qui refait toutes les images de `docs/images/` depuis l'extrait de test |

## Qui tient quoi

Chaque vérité nouvelle s'écrit à un seul endroit ; les autres documents y renvoient. Les mesures
datées que 02 et 10 partageaient avant cette règle y restent. Toute modification
de comportement du code met à jour ce document-là, dans la même série de changements.

| Vérité | Document | Ailleurs |
| --- | --- | --- |
| Valeur de chaque réglage et sa justification mesurée | [02](02-methode.md) | 10 cite le défaut et renvoie à 02 ; le README n'en recopie aucun (`batdetect --help` affiche les défauts depuis le code) |
| Logique étape par étape, module de chaque étape | [10](10-logique-de-detection.md) | — |
| Chaîne de construction, charte, tests, licences, encodage | [11](11-migration-rust.md), sections Construction à Plage réelle des sources | 05 et 02 y renvoient |
| Détail des crates | [11](11-migration-rust.md#crates), figé au 16 octobre 2026 | l'état courant est dans 10, 02 et CLAUDE.md |
| Carte courte des crates, règles des agents | [CLAUDE.md](../CLAUDE.md) | — |
| Temps de calcul actuels | [09](09-profilage.md) | les autres citent ses chiffres |
| Questions ouvertes | [06](06-questions-ouvertes.md) | — |
| Utilisation, fichiers produits | [README](../README.md) et [lire-les-resultats](lire-les-resultats.md) | — |
| Provenance et fabrication des images des docs | [images/README](images/README.md) et [tools/docs-images.sh](../tools/docs-images.sh) | les docs pour la naturaliste n'en disent rien |
| Parcours d'une soirée, des jumelles aux résultats, pour qui n'a jamais lancé l'outil | [premiere-soiree](premiere-soiree.md) | le README y renvoie |
| Lecture de l'image résumé : bandeau, trait, flèche, pastille, zone masquée, frise du temps, couleurs | [lire-les-resultats](lire-les-resultats.md#lire-limage-résumé) | le README y renvoie |
| Lecture d'un extrait, façon de désigner une piste et de noter son verdict, faux positifs observés | [lire-les-resultats](lire-les-resultats.md#juger-un-extrait) | le dessin des extraits et ses réglages sont dans [02](02-methode.md#extrait-zoomé-et-ralenti) |
| Histoire et mesures datées | 11 et les sections datées de 03, 07, 08, 09 | — |
