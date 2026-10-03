# Contexte

## Le besoin

Des naturalistes filment des falaises au crépuscule avec des jumelles thermiques pour
repérer les chauves-souris. Regarder des heures de vidéo à l'œil est long et on rate
des passages. `batdetect` repère automatiquement chaque objet en mouvement, le suit, et
sort un extrait par passage pour qu'une naturaliste confirme ou rejette.

L'outil ne dit pas « c'est une chauve-souris » : il dit « quelque chose a bougé ici,
de telle seconde à telle seconde ». La décision reste humaine.

Manon, naturaliste, sert de référence : elle confirme les passages à l'œil.

## Les vidéos

Mesuré sur les fichiers reçus (`ffprobe`) :

| Propriété | Valeur |
| --- | --- |
| Résolution | 1440×1080, très probablement agrandie depuis un capteur plus petit |
| Cadence | 30 i/s constante |
| Codec | H.264, environ 1 Mb/s, une image-clé toutes les 0,97 s |
| Durée | 5 min par fichier (découpage automatique des jumelles) |
| Audio | piste AAC présente, sans intérêt pour la détection |
| Affichage incrusté (OSD) | compteur d'enregistrement rouge, heure, date, batterie en haut ; icônes de zoom en bas sur certains fichiers |

Le compteur d'enregistrement change chaque seconde : sans masque, n'importe quel
détecteur de mouvement le prend pour un passage. D'où les bandes `osd_top` et
`osd_bottom`.

## Ce que la prise de vue impose

**Les jumelles doivent être fixes.** La vidéo `video_027` a été écartée : Manon avait
bougé ses jumelles, tout le fond bouge, et la détection par soustraction de fond n'a plus
de sens. Sur `video_092`, posée, la luminosité moyenne ne varie que de 3 niveaux sur 255
en 5 minutes et aucun pixel ne bouge en dehors des passages.

**La chauve-souris est plus sombre que la roche.** En palette white-hot, la falaise
chauffée par la journée sort blanche ; une chauve-souris en vol passe devant en tache
sombre (contraste négatif). Tout outil qui suppose « chaud = clair » rate ces passages.
Sur le ciel froid, le contraste s'inverserait. `batdetect` détecte donc l'écart au fond
dans les deux sens.

**La compression écrase les petits objets.** À 1 Mb/s, une chauve-souris de quelques
pixels est souvent fragmentée en deux taches (corps et aile, ou artefact de bloc). C'est
la raison d'être de `merge_radius` et `twin_distance` (voir [02-methode.md](02-methode.md)).

Recommandations de réglage des jumelles, issues de la littérature (voir
[04-outils-existants.md](04-outils-existants.md)) : trépied, palette et contraste fixes,
calibration (NUC) manuelle ou semi-automatique pendant l'enregistrement, débit maximal,
pas de zoom numérique.
