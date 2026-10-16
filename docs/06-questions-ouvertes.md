# Questions ouvertes

1. **Validation par Manon** des 14 pistes de la 092. **Fait (9 octobre 2026)** : 13 sont des
   chauves-souris, seule la piste lente 3:29 → 3:39 n'en est pas une
   ([03](03-resultats-092.md#verdict-de-manon-9-octobre-2026)). Règle qui en sort : mieux
   vaut un faux positif facile à écarter qu'un passage manqué ([02](02-methode.md)). #6 fermée.
2. **Nature de la piste 9** (3:29 à 3:39, lente). **Tranché (9 octobre 2026)** : pas une
   chauve-souris ; le « 3:36 » de septembre était cette chose.
3. **Deux chauves-souris simultanées** : la fusion des jumelles n'a été testée que sur
   des trajectoires fabriquées. Il faut un extrait réel où Manon sait qu'il y en a deux.
4. **Contraste inversé sur le ciel** : aucune vidéo avec des passages sur ciel froid
   pour l'instant. La détection dans les deux sens est testée sur des images
   fabriquées seulement.
5. **Jumelles qui bougent** (cas de la 027) : stabiliser l'image avant détection, ou au
   moins repérer et signaler les passages où tout le fond bouge.
   - **Fait (4 octobre)** : les images saturées de taches et leurs abords sont ignorés
     et signalés ([02](02-methode.md#4-bis-images-saturées-de-taches)). 089 : 1 546 →
     96 pistes, 091 : 4 327 → 53. Stabiliser l'image pour garder ces périodes reste
     possible, mais coûterait au moins 29 s par vidéo.
   - **Essaim** : 20 chauves-souris ou plus dans la même image seraient prises pour un
     mouvement. Réponse du 4 octobre : très peu probable ; si ça arrive, `--max-blobs`
     règle la limite et `--max-blobs 0` désactive le filtre.
   - Les pistes restantes de la 089 et de la 091 étaient surtout des points d'1 pixel de
     travail enchaînés au hasard. **Fait (4 octobre)** : `min_hits` 6 et filtre de virage
     médian ([02](02-methode.md#7-filtres-finaux)) : 089 → 4 pistes, 091 → 23.
     À montrer à Manon : les 4 de la 089 et les 23 de la 091.
6. **Classification grossière** par vitesse : vol rapide, déplacement lent, quasi
   immobile.
7. **Comparaison avec ThruTracker** sur la même vidéo, pendant l'essai gratuit.
8. **Temps de calcul et traitement par lots.** Une soirée d'une heure fait 12 clips de
   5 min. Mesuré le 16 octobre 2026 sur l'original de la 092 (Apple M1 Max, 10 cœurs,
   64 Go) : 19,81 s par clip, pour 143 s de temps CPU, soit environ 7 cœurs occupés en
   moyenne ([09](09-profilage.md)). Une heure d'enregistrement prend donc environ 4 min. La
   v0.1.1 en prend 135,4 s (#198, [09](09-profilage.md)), et ses pistes d'accélération (tranches de temps,
   médiane sur GPU, Python sans GIL) n'ont plus d'objet : le Rust partage déjà la détection
   entre tous les cœurs.
   - **La machine cible est celle de Manon** : Apple M3 (8 cœurs, dont 4 de
     performance) et 16 Go. Le temps par clip n'y a pas été mesuré. Si c'est un MacBook Air,
     sans ventilateur, il ralentira sur un long lot.
   - **Continuité entre clips** : une chauve-souris qui passe à la frontière de deux
     clips consécutifs (`video_092` puis `video_093`) est aujourd'hui coupée en deux
     pistes, dans deux dossiers. Traiter une série comme un flux continu (fond et suivi
     qui enjambent la frontière) donnerait une seule piste.
   - **Hystérésis 3D** : garder en mémoire le volume de résidus d'un clip en pleine
     résolution (environ 14 Go) est exclu sur 16 Go ; une hystérésis 3D devra travailler sur
     une fenêtre glissante.
9. **Distribution** aux naturalistes.
   - **Tranché (8 octobre, #30) pour la v0.1.1 : binaire autonome par Homebrew**,
     `brew install zaratan/bat-tools/batdetect`, plutôt qu'une formule sur le Python et
     l'OpenCV de Homebrew (91 formules de plus, environ 3,2 Go, et des surfaces un peu
     différentes). Mac Apple Silicon, macOS 14 minimum, binaire sous GPL v3 à cause du
     FFmpeg embarqué.
   - **Tranché (16 octobre, #224) pour le Rust** : le binaire est sous MIT pour le code et
     LGPL 2.1 ou ultérieure pour FFmpeg, lié en statique. Chaque release joint les sources
     de FFmpeg et d'openh264, pour pouvoir relier. openh264 est construit depuis les
     sources, donc sans la licence de brevets de Cisco : brevets acceptés, et `NOTICE` le
     dit ([11](11-migration-rust.md#licences), [05](05-outillage.md#distribution)).
10. **Lisibilité de l'image résumé** (`<nom>_resume.png`), constatée sur l'original de la 092 :
    les étiquettes des pistes qui entrent par le bord droit sortaient du cadre, celles des
    pistes groupées se chevauchaient, le rouge se lisait mal sur la roche claire, et rien
    n'indiquait le sens du vol.
    - **Fait (4 octobre).** Fond médian de 25 images (moins bruité, sans chauve-souris),
      une couleur par piste choisie pour différer de ses voisines, liseré sombre, flèche
      de fin, numéro dans une pastille posée dans le cadre, à l'écart des autres pastilles, des
      flèches et des croisements autant que possible, et légende dans un bandeau à droite (numéro,
      début en m:ss), sur plusieurs colonnes si besoin. Tailles proportionnelles à la
      largeur de la vidéo. Une image par tranche de 10 min pour les vidéos plus longues ;
      depuis #157, coupée en parties au-delà de 20 pistes ; depuis #127, une frise du temps
      et les plages illisibles datées.
    - Reste : les mêmes couleurs et pastilles sur la vidéo annotée et les extraits.
    - L'affichage des jumelles (heure, date, batterie, chrono flou) reste dans le fond
      médian, puisqu'il est fixe. `--osd-region` le masque (`0,0,1,0.07` pour les
      Symbion) ; sans option, rien n'est masqué, pour ne dépendre d'aucune caméra.
