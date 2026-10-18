# Contexte

## Le besoin

Des naturalistes filment des falaises au crépuscule avec des jumelles thermiques pour repérer les chauves-souris.
Regarder des heures de vidéo à l'œil est long, et on rate des passages.
`batdetect` repère chaque objet qui bouge, le suit, et sort un extrait par passage, qu'une naturaliste confirme ou écarte.

L'outil signale que quelque chose a bougé à tel endroit, de telle seconde à telle seconde.
C'est la naturaliste qui dit si c'est une chauve-souris : Manon sert de référence et confirme les passages à l'œil.

## Les jumelles

Manon utilise des **Pulsar Symbion DXT50** (manuel : *Symbion full manual FR*, fourni par l'utilisateur). D'après les spécifications :

| Propriété | Valeur |
| --- | --- |
| Capteur thermique | microbolomètre 1280×1024, pas de 12 µm, NETD < 35 mK (capteur), < 20 mK (système) |
| Fréquence du capteur | 50 Hz |
| Grossissement thermique | 2× de base, zoom numérique ×1, ×2, ×4 |
| Enregistrement | 1440×1080, `.mp4`, clips de 5 min au plus |
| Réglages utiles | compression vidéo ON/OFF, calibrage auto, semi-auto ou manuel, amplification Normal/Haut/Ultra, filtre de lissage, stabilisation d'image, 9 palettes, luminosité et contraste de 0 à 20 |

Les « 2.0x » et « 6.5x » de l'affichage incrusté sont les grossissements de base des voies thermique et numérique : nos vidéos n'ont pas de zoom numérique.

## Récupérer les vidéos originales

**L'export par l'application Stream Vision 2 réencode les vidéos et divise leur débit par environ 12.**
On copie donc les originaux directement depuis les jumelles : les brancher en USB-C **allumées**, choisir le mode **« Accès aux fichiers »** avec la bague, valider par MENU.
Elles apparaissent alors comme une clé USB (manuel, section « Connexion USB »).

| | Export Stream Vision 2 | Original (USB) |
| --- | --- | --- |
| `video_092`, 5 min | 39 Mo | 454 Mo |
| Débit | 1,04 Mb/s | 12,1 Mb/s |
| Structure | I BBBP, 72 % de B-frames, image-clé toutes les 29 images | I PPP, sans B-frames, image-clé toutes les 15 images |
| Cadence | 30 i/s | 30,03 i/s (9 011 images) |

Les deux ont la même cadence : les saccades du mouvement viennent des jumelles.
Effet de l'export sur la détection : voir [03-resultats-092.md](03-resultats-092.md).

L'export complet de la 092 n'existe plus (supprimé par erreur le 3 octobre 2026).
La référence du projet est l'original `video_092_original.mp4`, dont l'extrait de test est tiré.

## Les vidéos

Mesuré par `ffprobe` sur l'original de la 092 :

| Propriété | Valeur |
| --- | --- |
| Résolution | 1440×1080, rééchantillonnée depuis le capteur 1280×1024 (format 5:4 vers 4:3) |
| Cadence | 30,03 i/s constante, pour un capteur à 50 Hz : le mouvement avance par saccades (voir [07](07-ameliorer-la-detection.md)) |
| Codec | H.264 High, 12,1 Mb/s, I PPP, une image-clé toutes les 15 images |
| Durée | 5 min par fichier (limite des clips des jumelles) |
| Audio | piste AAC, sans intérêt pour la détection |
| Affichage incrusté (OSD) | compteur d'enregistrement rouge, heure, date, batterie en haut ; grossissements (« 2.0x », « 6.5x ») en bas sur certains fichiers |

Sur l'original de la 092, sans aucun masque, aucune piste ne vient de l'affichage incrusté.
Ce qui reste fixe ne crée aucune détection.
Le compteur d'enregistrement change chaque seconde, mais sur place : ses détections ne se déplacent pas, et le suivi rejette ce qui parcourt moins de `min_travel`.
L'outil n'a donc **pas de masque par défaut**, ce qui le garde indépendant de la caméra.
`--osd-region` sert pour un appareil dont l'affichage bougerait.

## Ce que la prise de vue impose

**Les jumelles doivent être fixes.**
La vidéo `video_027` a été écartée : Manon avait bougé ses jumelles, tout le fond bouge, et la détection par soustraction de fond n'a plus de sens.
Sur `video_092`, posée, la luminosité moyenne ne varie que de 3 niveaux sur 255 en 5 minutes, et aucun pixel ne bouge en dehors des passages.

La chauve-souris est plus sombre que la roche.
En palette white-hot, la falaise chauffée par la journée sort blanche, et une chauve-souris en vol passe devant en tache sombre (contraste négatif).
Un outil qui suppose « chaud = clair » rate ces passages.
Sur le ciel froid, le contraste s'inverserait : `batdetect` détecte donc l'écart au fond dans les deux sens.

La compression fragmente les petits objets.
Sur l'export à 1 Mb/s, une chauve-souris de quelques pixels sortait souvent en deux taches (corps et aile, ou artefact de bloc).
C'est la raison d'être de `merge_radius` et `twin_distance` (voir [02-methode.md](02-methode.md)).

Les conseils de réglage des jumelles tirés de la littérature ([04](04-outils-existants.md)) sont donnés à la naturaliste dans [lire les résultats](lire-les-resultats.md).
