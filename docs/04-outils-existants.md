# Outils existants

État au 3 octobre 2026. Le choix a été de développer `batdetect`, mais ces options
restent utiles pour comparer.

## DVR-Scan (testé, version 1.8.2.1)

Détecteur de mouvement générique par soustraction de fond (MOG2), pensé pour la
vidéosurveillance.

| Réglage | 092 | 027 |
| --- | --- | --- |
| Défaut | 0 événement | 5 (tremblement de la caméra) |
| Zone sans OSD, `-df 2 -k 3 -t 0.02 -l 3` | 11 événements, dont 3:36 et 3:58 | — |

Pourquoi il ne suffit pas :

- par défaut, il réduit l'image et filtre le bruit, ce qui efface une cible de quelques
  pixels ;
- son seuil porte sur la proportion de l'image qui bouge, mal adapté à un point minuscule ;
- il fusionne les passages proches en un événement et ne dessine qu'une boîte pour tout
  ce qui bouge ;
- il crée un faux événement au démarrage (0:00 à 0:04, le temps que le fond s'installe).

## Spécifiques chauves-souris

- **ThruTracker 3.0** (Corcoran, Hedrick, Bat Conservation International) : suivi 2D/3D
  validé sur des sorties de gîte en thermique. macOS 14+ natif, licence annuelle, essai
  gratuit de 10 jours. Le seul outil mûr et maintenu pour ce cas : référence à essayer
  pour comparer. <https://www.thrutrackeranalytics.com/>
- **BatCount** (Bentley) : gratuit, Windows seulement, précision publiée de 51 à 95 %
  selon les vidéos. <https://sourceforge.net/projects/batcount/>
- **ThermalTracker** (PNNL) : suppose des cibles claires en white-hot, dépend
  d'OpenCV 2.4. Incompatible avec le contraste négatif de nos vidéos.
- **Synthèse à lire** : Ahlberg, Kuczynska, Kloepper (2025), *A Perspective on Thermal
  Imagery for Bat Emergence Counts* : le contraste thermique est le premier facteur de
  réussite, il faut valider sur des extraits comptés à la main, éviter le zoom numérique.
  <https://www.eaglehill.us/nabr-pdfs-special/nabr-004-Kloepper.pdf>

## Idées détournées

- **Fiji + TrackMate** : suivi de particules de microscopie, détecteur LoG et filtre de
  Kalman. Un point sur fond uniforme est exactement son cas d'usage.
- **MetDetPy** : détecteur de météores, petits objets rapides sur fond uniforme.
- **Traînées ffmpeg** (`lagfun`) : inutiles ici, elles gardent le maximum et nos cibles
  sont sombres. Même inversées, les traînées restent invisibles à l'échelle de l'image.
- **YOLO** entraîné sur nos propres extraits : seulement si la végétation qui bouge
  noyait un jour la détection par soustraction de fond. Les pistes confirmées par Manon
  feraient les premières annotations.

## Écartés

Logiciels de vidéosurveillance (Frigate, ZoneMinder, Motion, Shinobi), DeepLabCut,
idtracker.ai, ToxTrac, AnimalTA, Kinovea : conçus pour du direct, des animaux en arène
ou des objets plus gros.
