# Questions ouvertes

Une question tranchée garde son numéro, avec sa réponse et sa date.

1. **Validation par Manon** des 14 pistes de la 092.
   **Fait (9 octobre 2026)** : 13 sont des chauves-souris, seule la piste lente 3:29 → 3:39 n'en est pas une ([03](03-resultats-092.md#verdict-de-manon-9-octobre-2026)).
   La règle qui en sort : mieux vaut un faux positif facile à écarter qu'un passage manqué ([02](02-methode.md)).
2. **Nature de la piste 9** (3:29 à 3:39, lente).
   **Tranché (9 octobre 2026)** : ce n'est pas une chauve-souris, et le passage de 3:36 confirmé en septembre était cet objet.
3. **Deux chauves-souris simultanées** : la fusion de deux pistes proches en un seul animal (`twin_distance`) n'a été testée que sur des trajectoires fabriquées.
   Il faut un extrait réel où Manon sait qu'il y en a deux.
4. **Contraste inversé sur le ciel** : aucune vidéo n'a encore de passage sur ciel froid.
   La détection dans les deux sens n'est testée que sur des images fabriquées.
5. **Jumelles qui bougent** (cas de la 027) : stabiliser l'image avant détection, ou au moins repérer et signaler les passages où tout le fond bouge.
   - **Fait (4 octobre)** : les images saturées de taches et leurs abords sont ignorés et signalés ([02](02-methode.md#4-bis-images-saturées-de-taches)).
     Stabiliser l'image pour garder ces périodes reste possible, mais coûterait au moins 29 s par vidéo.
   - **Essaim** : 20 chauves-souris ou plus dans la même image seraient prises pour un mouvement du cadre.
     Réponse du 4 octobre : c'est très peu probable ; si ça arrive, `--max-blobs` règle la limite et `--max-blobs 0` désactive le filtre.
   - Les pistes restantes de la 089 et de la 091 étaient surtout des points d'un pixel de travail enchaînés au hasard.
     **Fait (4 octobre)** : `min_hits` 6 et filtre de virage médian ([02](02-methode.md#7-filtres-finaux)).
     Reste à montrer à Manon les 4 pistes de la 089 et les 23 de la 091.
6. **Classification grossière** par vitesse : vol rapide, déplacement lent, quasi immobile.
7. **Comparaison avec ThruTracker** sur la même vidéo, pendant l'essai gratuit.
8. **Temps de calcul et traitement par lots.** Une soirée d'une heure fait 12 clips de 5 min.
   Le 16 octobre 2026, sur l'original de la 092 (Apple M1 Max, 10 cœurs, 64 Go), un clip prend 19,81 s, pour 143 s de temps CPU : environ 7 cœurs occupés en moyenne ([09](09-profilage.md)).
   Une heure d'enregistrement prend donc environ 4 min.
   - **La machine cible est celle de Manon** : Apple M3 (8 cœurs, dont 4 de performance) et 16 Go.
     Le temps par clip n'y a pas été mesuré.
     Si c'est un MacBook Air, qui n'a pas de ventilateur, il ralentira sur un long lot.
   - **Continuité entre clips** : une chauve-souris qui passe à la frontière de deux clips consécutifs (`video_092` puis `video_093`) sort en deux pistes, dans deux dossiers.
     Traiter une série comme un flux continu, avec un fond et un suivi qui enjambent la frontière, en donnerait une seule.
   - **Hystérésis 3D** ([07](07-ameliorer-la-detection.md)) : garder en mémoire le volume de résidus d'un clip en pleine résolution (environ 14 Go) est exclu sur 16 Go.
     Elle devra travailler sur une fenêtre glissante.
9. **Distribution** aux naturalistes.
   **Tranché** : un binaire autonome, installé par Homebrew (`brew install zaratan/bat-tools/batdetect`, 8 octobre, #30).
   Le code est sous MIT, FFmpeg sous LGPL 2.1 ou ultérieure, lié en statique, et chaque release joint les sources de FFmpeg et d'openh264 pour pouvoir relier (16 octobre, #224).
   Licences et brevets d'openh264 : [11](11-migration-rust.md#licences) ; publication : [05](05-outillage.md#distribution).
10. **Lisibilité de l'image résumé** (`<nom>_resume.png`), constatée sur l'original de la 092 : les étiquettes des pistes qui entrent par le bord droit sortaient du cadre, celles des pistes groupées se chevauchaient, le rouge se lisait mal sur la roche claire, et rien n'indiquait le sens du vol.
    **Fait (4 octobre)** : le fond est la médiane de 25 images, moins bruitée et sans chauve-souris.
    Chaque piste a sa couleur, choisie pour différer de ses voisines, avec un liseré sombre et une flèche de fin.
    Son numéro est dans une pastille posée dans le cadre, à l'écart des autres, et un bandeau à droite sert de légende.
    Les tailles sont proportionnelles à la largeur de la vidéo.
    **Fait (16 octobre, #13)** : la vidéo annotée, les extraits et les zooms reprennent la couleur de chaque piste.
    La lecture de l'image est décrite dans [lire les résultats](lire-les-resultats.md#lire-limage-résumé), son dessin dans [02](02-methode.md#rendu).
    - L'affichage des jumelles (heure, date, batterie, chrono flou) reste dans le fond médian de l'image résumé, puisqu'il est fixe.
      `--osd-region` le masque (`0,0,1,0.07` pour les Symbion) ; sans option, rien n'est masqué, pour ne dépendre d'aucune caméra ([01](01-contexte.md)).
