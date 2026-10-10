# video_094

fps 30.026, 9011 images, 13 images réparées, 0 sauts de pts
Détections : python 155182, fast 155449, python retrouvées par fast 153157 (98.70%), fast retrouvées par python 153157 (98.53%)
Plages abîmées (communes) : 01:17.50–01:19.43, 01:22.43–01:28.26, 04:12.15–04:13.58, 04:14.15–04:18.08
Détections python sans voisine de l'autre côté : 2025, dont 8 hors plages ignorées (python), amplitude médiane 14.6 (max 19.4), surface médiane 4, sur 7 images (01:42.84…04:31.86)
Détections fast sans voisine de l'autre côté : 2292, dont 9 hors plages ignorées (fast), amplitude médiane 14.4 (max 21.4), surface médiane 4, sur 7 images (03:40.84…03:58.59)
Plages instables identiques : 01:20.03–01:22.36, 02:07.19–02:09.25, 02:28.87–02:30.97, 02:40.03–02:43.76, 03:04.91–03:39.98, 03:42.51–03:50.20, 04:01.29–04:05.05, 04:09.78–04:12.08
Pistes : python 14, fast 14, appariées ±5 images 13, dans la tolérance de points ±3 13
Écarts de début (fast−python) : [0]; écarts de points : [0]

Pistes python : 00:01.80(15), 00:01.90(17), 01:42.21(19), 01:45.57(10), 01:54.23(8), 01:55.73(15), 02:27.84(6), 02:39.29(17), 02:58.44(40), 03:40.11(8), 03:56.76(17), 03:57.59(15), 04:28.20(12), 04:42.75(9)

- **python seule** : 01:42.21–01:44.04 (images 3069–3124), 19 points, corde 940 px, amplitude médiane 20.3 (min 13.6), surface médiane 11
  - côté fast : 18/19 points ont une détection à moins de 3 px avant exclusion, 18/19 après ; 0/19 points dans une plage instable fast
  - amplitude fast/python sur les points communs : rapport médian 1.00
  - points repris par : piste fast 01:43.08 (15 pts, 15 pts communs)
  - images sans détection fast : [3088]
- **fast seule** : 01:43.08–01:44.04 (images 3095–3124), 15 points, corde 451 px, amplitude médiane 21.8 (min 14.2), surface médiane 18
  - côté python : 15/15 points ont une détection à moins de 3 px avant exclusion, 15/15 après ; 0/15 points dans une plage instable python
  - amplitude python/fast sur les points communs : rapport médian 1.00
  - points repris par : piste python 01:42.21 (19 pts, 15 pts communs)
