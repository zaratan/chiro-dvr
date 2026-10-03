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
8. **Temps de calcul** : 2 min 40 pour 5 min de vidéo, dont environ 100 s de détection.
   Un cache des détections rendrait le réglage du suivi instantané.
9. **Distribution** aux naturalistes : binaire unique ou installation via uv.
