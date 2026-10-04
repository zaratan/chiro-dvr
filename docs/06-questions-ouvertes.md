# Questions ouvertes

1. **Validation par Manon** des 12 pistes de la 092 : lesquelles sont des chauves-souris,
   lesquelles sont autre chose (insecte proche, oiseau). Donnera le taux de vraies et
   fausses détections, et permettra d'ajuster `threshold`, `min_hits` et `min_area`.
2. **Nature de la piste 9** (3:29 à 3:39, lente) : voir [03-resultats-092.md](03-resultats-092.md).
3. **Deux chauves-souris simultanées** : la fusion des jumelles n'a été testée que sur
   des trajectoires fabriquées. Il faut un extrait réel où Manon sait qu'il y en a deux.
4. **Contraste inversé sur le ciel** : aucune vidéo avec des passages sur ciel froid
   pour l'instant. La détection dans les deux sens est testée sur des images
   fabriquées seulement.
5. **Jumelles qui bougent** (cas de la 027) : stabiliser l'image avant détection, ou au
   moins repérer et signaler les passages où tout le fond bouge.
6. **Classification grossière** par vitesse : vol rapide, déplacement lent, quasi
   immobile.
7. **Comparaison avec ThruTracker** sur la même vidéo, pendant l'essai gratuit.
8. **Temps de calcul et traitement par lots.** Une soirée d'une heure fait 12 clips de
   5 min. Mesuré sur l'original de la 092 (Apple M1 Max, 10 cœurs, 64 Go) : 4 min 29 par clip,
   pour 1 300 s de temps CPU, soit environ 4,8 cœurs déjà occupés en moyenne par OpenCV,
   ffmpeg et numpy. En séquentiel, une heure d'enregistrement prend donc environ 54 min.
   Pistes :
   - **Fait (4 octobre) : détection découpée en tranches de temps**, une par processus,
     identique au bit près au séquentiel. Gain mesuré ×2,3 sur le banc à 480 px avec
     10 processus sur le M1 Max ; à mesurer sur le M3, dont 4 cœurs sur 8 sont des cœurs
     économes (la tranche la plus lente fixe le temps total). Chaque tranche relit la
     vidéo depuis le début pour éviter un positionnement imprécis : environ 90 s de
     rattrapage pour la dernière tranche d'une vidéo d'une heure.
   - **Le rendu domine maintenant le temps total.** Sur l'original, avec la détection
     parallèle : 3 min 35 au total, pour environ 1 min de détection (déduit du banc, deux
     passes en 1 min 58). Le reste vient de la vidéo annotée complète (redécodage,
     dessin, réencodage x264 de 5 min en 1440×1080, sur un seul flux) et des extraits
     recoupés depuis elle. Pistes : encodage matériel `h264_videotoolbox` ; extraits
     tirés directement de l'original au lieu de la vidéo annotée ; vidéo complète en
     option ; rendu par tranches. Mesurer d'abord la part de chaque étape.
   - **Médiane sur le GPU (Metal)** : le GPU reste à 0 % pendant le traitement. La médiane
     d'une pile de 11 images est un calcul massivement parallèle, adapté au GPU : MLX
     (Apple, léger) ou PyTorch avec le backend MPS (lourd). Ce serait une dépendance
     optionnelle, réservée aux Mac, avec repli sur numpy, puisque la CI tourne sous Linux.
     Gain à mesurer face au découpage multiprocessus, avec lequel il se combine mal (un
     seul GPU partagé par les processus).
   - **Python sans GIL (free-threading)** : des threads au lieu de processus
     partageraient la mémoire. Bloqué au 3 octobre 2026 : numpy publie des wheels
     `cp314t`, mais `opencv-python-headless` 5.0 seulement des wheels `abi3`,
     incompatibles avec un Python sans GIL. À revoir avec Python 3.15 et les wheels
     d'OpenCV qui suivront.
   - **Traiter plusieurs vidéos en parallèle** (un processus par vidéo, nombre de
     processus réglable). Avec 4,8 cœurs déjà pris par vidéo, deux vidéos à la fois
     occupent la machine : gain attendu d'environ ×2, pas ×10. Estimation, à mesurer.
     La mémoire n'est pas une limite : la détection lit la vidéo en flux et ne garde
     qu'une seconde d'images. Avec 64 Go, on pourrait même garder tout le volume de
     résidus d'un clip (environ 1,5 Go à 480 px de large, 14 Go en pleine résolution),
     ce qu'exigerait une hystérésis 3D sur toute la durée.
   - **Réduire le coût par vidéo**, qui multiplie tous les gains :
     - rendre la vidéo annotée complète optionnelle, et ne produire que les extraits
       autour des pistes. Le rendu redécode et réencode 5 min en 1440×1080 ; sa part du
       temps total reste à mesurer ;
     - encodage matériel (`h264_videotoolbox` sur Mac) pour les extraits ;
     - fond médian recalculé seulement une image sur N (la médiane faisait 85 % du
       temps de détection sur l'export) ;
     - cache des détections : rejouer le suivi et le rendu sans redétecter, pour régler
       les paramètres en quelques secondes.
   - **La machine cible est celle de Manon** : Apple M3 (8 cœurs, dont 4 de
     performance) et 16 Go. Le temps par clip n'y a pas été mesuré. Avec moins de cœurs
     de performance que le M1 Max, le parallélisme y rapportera peu : c'est la réduction
     du coût par vidéo qui compte. Avec 16 Go, garder en mémoire le volume de résidus d'un
     clip en pleine résolution (14 Go) est exclu : une hystérésis 3D devra travailler sur
     une fenêtre glissante. Si c'est un MacBook Air, sans ventilateur, il ralentira sur
     un long lot. Le nombre de processus parallèles doit donc s'adapter à la machine.
   - **Continuité entre clips** : une chauve-souris qui passe à la frontière de deux
     clips consécutifs (`video_092` puis `video_093`) est aujourd'hui coupée en deux
     pistes, dans deux dossiers. Traiter une série comme un flux continu (fond et suivi
     qui enjambent la frontière) donnerait une seule piste.
9. **Distribution** aux naturalistes : binaire unique ou installation via uv.
10. **Lisibilité de l'image résumé** (`*.tracks.png`), constatée sur l'original de la 092 :
    les étiquettes des pistes qui entrent par le bord droit sortent du cadre, celles des
    pistes groupées se chevauchent, le rouge se lit mal sur la roche claire, et rien
    n'indique le sens du vol. Proposition : seulement un numéro dans une pastille sombre,
    posée au milieu de la trajectoire et décalée en cas de collision ; une légende dans un
    bandeau ajouté sur le côté (numéro, début, durée, vitesse) ; une couleur par piste
    reprise dans la légende ; une flèche au bout de chaque trajectoire ; les mêmes
    pastilles sur la vidéo annotée. À faire relire par l'agent UI/UX avant de coder.
