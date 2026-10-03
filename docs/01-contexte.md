# Contexte

## Le besoin

Des naturalistes filment des falaises au crépuscule avec des jumelles thermiques pour
repérer les chauves-souris. Regarder des heures de vidéo à l'œil est long et on rate
des passages. `batdetect` repère automatiquement chaque objet en mouvement, le suit, et
sort un extrait par passage pour qu'une naturaliste confirme ou rejette.

L'outil ne dit pas « c'est une chauve-souris » : il dit « quelque chose a bougé ici,
de telle seconde à telle seconde ». La décision reste humaine.

Manon, naturaliste, sert de référence : elle confirme les passages à l'œil.

## Les jumelles

Manon utilise des **Pulsar Symbion DXT50** (manuel : *Symbion full manual FR*,
fourni par l'utilisateur). D'après les spécifications :

| Propriété | Valeur |
| --- | --- |
| Capteur thermique | microbolomètre 1280×1024, pas de 12 µm, NETD < 35 mK (capteur), < 20 mK (système) |
| Fréquence du capteur | 50 Hz |
| Grossissement thermique | 2× de base, zoom numérique ×1, ×2, ×4 |
| Enregistrement | 1440×1080, `.mp4`, clips de 5 min au plus |
| Réglages utiles | compression vidéo ON/OFF, calibrage auto, semi-auto ou manuel, amplification Normal/Haut/Ultra, filtre de lissage, stabilisation d'image, 9 palettes, luminosité et contraste de 0 à 20 |

Les « 2.0x » et « 6.5x » de l'affichage incrusté sont les grossissements de base des
voies thermique et numérique : pas de zoom numérique sur nos vidéos.

## Récupérer les vidéos originales

**L'export par l'application Stream Vision 2 réencode les vidéos et divise leur débit
par environ 12.** Il faut copier les originaux directement depuis les jumelles :
brancher en USB-C les jumelles **allumées**, choisir le mode **« Accès aux fichiers »**
avec la bague, valider par MENU. Les jumelles apparaissent comme une clé USB
(manuel, section « Connexion USB »).

| | Export Stream Vision 2 | Original (USB) |
| --- | --- | --- |
| `video_092`, 5 min | 39 Mo | 454 Mo |
| Débit | 1,04 Mb/s | 12,1 Mb/s |
| Structure | I BBBP, 72 % de B-frames, image-clé toutes les 29 images | I PPP, sans B-frames, image-clé toutes les 15 images |
| Cadence | 30 i/s | 30,03 i/s (9 011 images) |

La cadence est la même : la saccade du mouvement vient des jumelles, pas de l'export.
Effet sur la détection : voir [03-resultats-092.md](03-resultats-092.md).

La version exportée complète de la 092 a été supprimée par erreur le 3 octobre 2026. La
référence du projet est désormais l'original `video_092_original.mp4`, et l'extrait de
test en est tiré.

## Les vidéos

Mesuré sur la version exportée, sur laquelle le projet a démarré (`ffprobe`) :

| Propriété | Valeur |
| --- | --- |
| Résolution | 1440×1080, rééchantillonnée depuis le capteur 1280×1024 (format 5:4 vers 4:3) |
| Cadence | 30 i/s constante, pour un capteur à 50 Hz : le mouvement avance par saccades (voir [07](07-ameliorer-la-detection.md)) |
| Codec | H.264 High, environ 1 Mb/s (option « Compression vidéo » probablement sur ON), I BBBP, une image-clé toutes les 29 images |
| Durée | 5 min par fichier (limite des clips des jumelles) |
| Audio | piste AAC présente, sans intérêt pour la détection |
| Affichage incrusté (OSD) | compteur d'enregistrement rouge, heure, date, batterie en haut ; grossissements (« 2.0x », « 6.5x ») en bas sur certains fichiers |

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
