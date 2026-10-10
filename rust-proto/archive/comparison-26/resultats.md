# Détection fast contre Python, toutes les vidéos

Même `Probe` (frames.txt) des deux côtés ; `exclude` puis `track_detections`, configs par défaut. Appariement des pistes par image de début ±5, points ±3.

| Vidéo | Dét. py | Dét. fast | % py retrouvées | Pistes py | Pistes fast | Appariées dans tol. | Sans correspondant (début, points) | Python réel s | fast réel / CPU s |
|---|---|---|---|---|---|---|---|---|---|
| 089 | 4791 | 4784 | 98.85% (98.19% hors ignorées) | 1 | 1 | 1/1 | — | 122 (det 122 s, RSS 355 Mo) | 11.1 / 101.5 |
| 090 | 1972 | 1976 | 98.94% (98.94% hors ignorées) | 20 | 20 | 20/20 | — | 123 (det 123 s, RSS 332 Mo) | 11.0 / 101.2 |
| 091 | 34524 | 34504 | 99.30% (98.64% hors ignorées) | 21 | 21 | 21/21 | — | 128 (det 128 s, RSS 502 Mo) | 12.7 / 103.1 |
| 093 | 2611 | 2647 | 91.19% (99.05% hors ignorées) | 6 | 6 | 6/6 | — | 120 (det 120 s, RSS 340 Mo) | 10.9 / 98.9 |
| 122 | 30496 | 30410 | 91.16% (99.48% hors ignorées) | 0 | 0 | 0/0 | — | 120 (det 120 s, RSS 356 Mo) | 13.1 / 101.0 |
| 125 | 1263 | 1262 | 99.92% (99.92% hors ignorées) | 47 | 47 | 47/47 | — | 119 (det 118 s, RSS 331 Mo) | 12.9 / 102.1 |
| 126 | 323 | 324 | 99.38% (99.38% hors ignorées) | 12 | 13 | 11/13 | fast 01:32.45 (11); hors tol. py 01:31.72 32/fast 27 pts | 121 (det 120 s, RSS 337 Mo) | 12.9 / 103.2 |
| 127 | 176 | 177 | 100.00% (100.00% hors ignorées) | 7 | 7 | 7/7 | — | 124 (det 124 s, RSS 334 Mo) | 12.7 / 100.9 |
| 092 | 780 | 781 | 100.00% (100.00% hors ignorées) | 14 | 14 | 14/14 | — | 119 (det 119 s, RSS 345 Mo) | 10.9 / 98.7 |
| 094 | 155182 | 155449 | 98.70% (98.52% hors ignorées) | 14 | 14 | 13/14 | py 01:42.21 (19); fast 01:43.08 (15) | 128 (det 128 s, RSS 808 Mo) | 14.2 / 124.1 |
| 095 | 5111 | 4897 | 87.60% (99.90% hors ignorées) | 78 | 78 | 78/78 | — | 121 (det 121 s, RSS 343 Mo) | 10.9 / 100.0 |
| 096 | 22603 | 22522 | 97.38% (99.84% hors ignorées) | 136 | 136 | 136/136 | — | 123 (det 122 s, RSS 389 Mo) | 10.9 / 100.3 |
| 097 | 17678 | 17630 | 97.81% (99.77% hors ignorées) | 148 | 149 | 146/149 | py 00:20.38 (77); fast 00:20.62 (76); fast 02:43.39 (61); hors tol. py 02:42.26 86/fast 24 pts | 125 (det 125 s, RSS 398 Mo) | 11.0 / 99.3 |
| 098 | 15428 | 15344 | 98.18% (99.73% hors ignorées) | 131 | 131 | 130/131 | py 00:41.46 (21); fast 00:05.86 (7) | 121 (det 121 s, RSS 354 Mo) | 11.1 / 100.8 |
| 099 | 13518 | 13567 | 93.16% (99.76% hors ignorées) | 115 | 117 | 112/117 | fast 00:19.42 (79); fast 00:27.91 (6); hors tol. py 00:14.29 75/fast 119 pts; hors tol. py 00:16.95 60/fast 16 pts; hors tol. py 00:17.92 127/fast 50 pts | 122 (det 122 s, RSS 347 Mo) | 11.5 / 103.3 |
| 100 | 7430 | 7357 | 97.55% (99.77% hors ignorées) | 96 | 98 | 96/98 | fast 01:36.25 (13); fast 01:36.38 (10) | 124 (det 123 s, RSS 353 Mo) | 11.7 / 101.5 |
| 101 | 5619 | 5570 | 94.93% (99.38% hors ignorées) | 32 | 32 | 32/32 | — — instables py 3 / fast 3 | 121 (det 120 s, RSS 343 Mo) | 11.6 / 101.6 |
| 102 | 4398 | 4407 | 93.70% (99.35% hors ignorées) | 36 | 36 | 36/36 | — | 122 (det 122 s, RSS 340 Mo) | 12.0 / 102.0 |
| 103 | 548329 | 548545 | 98.79% (98.81% hors ignorées) | 10 | 10 | 10/10 | — | 169 (det 167 s, RSS 923 Mo) | 30.1 / 258.1 |
| 120 | 15633 | 13760 | 72.07% (99.51% hors ignorées) | 0 | 0 | 0/0 | — | 120 (det 120 s, RSS 384 Mo) | 11.3 / 96.9 |
| 121 | 28596 | 28794 | 90.95% (99.56% hors ignorées) | 2 | 2 | 2/2 | — | 119 (det 119 s, RSS 361 Mo) | 12.5 / 101.7 |
| 123 | 28811 | 28331 | 89.97% (99.60% hors ignorées) | 8 | 8 | 8/8 | — | 120 (det 120 s, RSS 363 Mo) | 13.1 / 101.5 |
| 124 | 30750 | 31023 | 88.44% (100.00% hors ignorées) | 5 | 5 | 5/5 | — | 122 (det 122 s, RSS 368 Mo) | 13.5 / 101.6 |
| 128 | 3655 | 3492 | 80.33% (99.50% hors ignorées) | 5 | 5 | 5/5 | — | 119 (det 119 s, RSS 345 Mo) | 13.0 / 100.9 |
| 129 | 1629 | 1621 | 97.18% (100.00% hors ignorées) | 16 | 16 | 16/16 | — | 120 (det 120 s, RSS 339 Mo) | 12.9 / 100.4 |
| 130 | 31282 | 31302 | 99.22% (100.00% hors ignorées) | 6 | 6 | 6/6 | — | 71 (det 70 s, RSS 470 Mo) | 7.8 / 62.0 |
| **Total (26)** | 1012588 | 1010476 | 97.05% | 966 | 972 | 958 | py seules 3, fast seules 9, hors tol. 5 | 3164 | 327 / 2767 |

## Divergences de pistes (9 endroits, 6 vidéos)

Chaque cas : une seule image dont les détections diffèrent suffit à le provoquer (vérifié en substituant cette image seule, pour 099 et 100).

| Vidéo, temps | Pistes py → fast | Détection en cause |
|---|---|---|
| 094 01:42.21 | 19 pts dès l'image 3069 → 15 pts dès 3095 | image 3088 : tache py seule (787,702), 4 px, amplitude 13,6 ; sans elle les 3 premiers points restent isolés |
| 097 00:20.38 | 77 pts dès 612 → 76 pts dès 619 (début +7 > ±5) | image 612 : tache py seule (1425,566), 4 px, amplitude 16,4, rattachée en premier point |
| 097 02:42.26 | 86 pts → 24 + 61 pts (coupure) | image 4903 : tache py seule (1036,779), 4 px, amplitude 15,2, qui comble le trou 4898→4906 |
| 098 00:41.46 | 21 pts → aucune | image 1273 : tache py seule (1291,229), 4 px, amplitude 13,8, dernier point ; sans elle, corde 44,6 px < min_travel 45 |
| 098 00:05.86 | aucune → 7 pts | image 183 : tache fast seule (1337.5,250.8), 4 px, amplitude 14,0, qui comble le trou 178→187 |
| 099 00:14.29–00:22.61 | 75/60/127 pts → 119/16/50 + 79 pts (échanges d'identité) | image 519 : même tache, centroïde (562.6,995.8) 45 px contre (562.3,995.7) 43 px → échange à 523 ; image 585 : une tache py (1341,388) 16 px contre deux fast (1346,388) 9 px + (1338,392) 4 px → seconde coupure |
| 099 00:27.91 | aucune → 6 pts | image 845 : tache fast seule (1189,627), 4 px, amplitude 12,8 : 6e point = min_hits |
| 100 01:36.25 | aucune → 13 + 10 pts (grosses taches, jusqu'à 2000 px) | image 2901 : tache py seule (568.8,986.5), 4 px, amplitude 19,0 ; mécanisme de suivi non isolé |
| 126 01:31.72 | 32 pts → 27 + 11 pts | image 2776 : tache fast seule (875.5,507.2), 4 px, amplitude 12,6, prise par la piste |

Hypothèse éliminée : l'ordre des détections dans une image diffère sur 5 499 des 19 841 images à plusieurs détections appariées, mais remettre les détections fast dans l'ordre Python ne change aucune piste.
