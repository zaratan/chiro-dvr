# Documentation

## Pour utiliser l'outil

Ces documents s'adressent à la naturaliste. Ils ne contiennent rien de technique.

- [Première soirée](premiere-soiree.md) : régler les jumelles, filmer, copier les vidéos, lancer, ouvrir les résultats.
- [Lire les résultats](lire-les-resultats.md) : la page des résultats, l'image résumé, les extraits, le verdict, ce que l'outil ne voit pas.

## Pour comprendre ou modifier l'outil

- [Détection](detection.md) : chaque technique employée, étape par étape, avec ses réglages et leur mesure.
- [Développement](developpement.md) : construction, licences, charte, tests, encodage, contrats des crates, banc, temps actuels.
- [Résultats de la 092](resultats-092.md) : le verdict de l'experte, vérité terrain des tests.
- [CONTRIBUTING](../CONTRIBUTING.md) : construire, vérifier, publier une version, outillage et CI.
- [Images](images/README.md) : d'où vient chaque image des docs et comment la refaire.

L'histoire du dépôt est dans git et dans le [CHANGELOG](../CHANGELOG.md).

## Qui tient quoi

Chaque vérité s'écrit à un seul endroit. Les autres documents y renvoient.
Un changement de comportement du code met à jour ce document-là, dans la même série de changements.

| Vérité | Document |
| --- | --- |
| Chaque étape du calcul, sa technique, la valeur de chaque réglage et la mesure qui la fixe | [detection](detection.md) |
| Construction, licences, lints, charte, dépendances, tests, encodage, contrats des crates, banc, temps de calcul | [developpement](developpement.md) |
| Carte courte des crates, règles des agents | [CLAUDE.md](../CLAUDE.md) |
| Construire, vérifier, publier une version, CI | [CONTRIBUTING](../CONTRIBUTING.md) |
| Verdict de l'experte sur la 092 | [resultats-092](resultats-092.md) |
| Utilisation en bref | [README](../README.md) |
| Parcours d'une soirée, réglages des jumelles et prise de vue | [premiere-soiree](premiere-soiree.md) |
| Fichiers produits, lecture de l'image résumé et des extraits, verdict | [lire-les-resultats](lire-les-resultats.md) |
| Format de `params.json` et ménage des sorties | [developpement](developpement.md) |
| Provenance des images des docs | [images/README](images/README.md) et [tools/docs-images.sh](../tools/docs-images.sh) |
| Questions ouvertes et idées | les issues du dépôt |
