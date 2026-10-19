# Outils existants

État au 3 octobre 2026.
On a écrit `batdetect`, mais ces outils restent des points de comparaison.

## DVR-Scan (testé, version 1.8.2.1)

Détecteur de mouvement générique par soustraction de fond (MOG2).
Il est pensé pour la vidéosurveillance.
Avant la copie des originaux, il a été essayé le 3 octobre 2026 sur l'export de Stream Vision 2 (1 Mb/s).

| Réglage | 092 | 027 |
| --- | --- | --- |
| Défaut | 0 événement | 5 (tremblement de la caméra) |
| Zone sans OSD, `-df 2 -k 3 -t 0.02 -l 3` | 11 événements, dont 3:36 et 3:58 | |

Il vise des objets bien plus gros qu'une chauve-souris. Sur nos vidéos :

- Par défaut, il réduit l'image et filtre le bruit. Cela efface une cible de quelques pixels.
- Son seuil porte sur la proportion de l'image qui bouge. Un point minuscule n'en remue presque rien.
- Il fusionne les passages proches en un seul événement, avec une seule boîte pour tout ce qui bouge.
- Il crée un faux événement au démarrage (0:00 à 0:04) : son modèle du fond met ce temps à se former.

## Spécifiques chauves-souris

- **ThruTracker 3.0** (Corcoran, Hedrick, Bat Conservation International) : suivi 2D/3D validé sur des sorties de gîte en thermique.
  Il tourne en natif sur macOS 14+, sous licence annuelle, avec un essai gratuit de 10 jours.
  C'est le seul outil mûr et maintenu pour ce cas.
  Il est à essayer pour comparer ([06](06-questions-ouvertes.md)). <https://www.thrutrackeranalytics.com/>
- **BatCount** (Bentley) : gratuit, pour Windows seulement.
  Sa précision publiée va de 51 à 95 % selon les vidéos. <https://sourceforge.net/projects/batcount/>
- **ThermalTracker** (PNNL) : suppose des cibles claires en white-hot et dépend d'OpenCV 2.4.
  Il ne voit pas le contraste négatif de nos vidéos.
- **Synthèse à lire** : Ahlberg, Kuczynska, Kloepper (2025), *A Perspective on Thermal Imagery for Bat Emergence Counts*. <https://www.eaglehill.us/nabr-pdfs-special/nabr-004-Kloepper.pdf>
  Le contraste thermique y est le premier facteur de réussite.
  Il faut valider sur des extraits comptés à la main, et éviter le zoom numérique.

## Idées détournées

- **Fiji + TrackMate** : suivi de particules de microscopie, détecteur LoG et filtre de Kalman.
  Un point sur fond uniforme est exactement son cas d'usage.
- **MetDetPy** : détecteur de météores, petits objets rapides sur fond uniforme.
- **Traînées ffmpeg** (`lagfun`) : elles gardent le maximum. Or nos cibles sont sombres.
  Même inversées, les traînées restent invisibles à l'échelle de l'image.
- **YOLO** entraîné sur nos propres extraits : seulement si la végétation qui bouge noyait un jour la détection par soustraction de fond.
  Les pistes confirmées par Manon feraient les premières annotations.

## Écartés

Les logiciels de vidéosurveillance (Frigate, ZoneMinder, Motion, Shinobi) sont conçus pour du direct.
DeepLabCut, idtracker.ai, ToxTrac, AnimalTA et Kinovea visent des animaux en arène ou des objets plus gros.
