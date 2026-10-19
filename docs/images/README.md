# Images des docs

Ces images illustrent les docs pour la naturaliste ([première soirée](../premiere-soiree.md), [lire les résultats](../lire-les-resultats.md)).
Pour les refaire toutes, on lance `tools/docs-images.sh` depuis la racine du dépôt.
Le script traite en français l'extrait de test `tests/fixtures/video_092_original_3m24-4m05.mp4` dans `out/docs-images/`.
Il dessine ensuite les repères avec `magick`.
Les coordonnées des repères et les numéros d'image valent pour cet extrait.

D'où vient chaque image :

- `resume-extrait.jpg` et `resume-frise.png` : l'image résumé de l'extrait avec l'affichage masqué (`--osd-region 0,0,1,0.07`).
- `extrait-lente.jpg`, `extrait-chauve-souris.jpg`, `extrait-zoom.jpg` et `extrait-boites.png` : des images tirées par `ffmpeg` de ses extraits et zooms (`--zoom all`).
- `premiere-soiree-*.jpg` : une fenêtre de Terminal dessinée avec les sorties de la commande.
  Ces sorties sont recopiées dans le script et sont à mettre à jour quand un texte de la commande change.
- `results-page.jpg`, en tête du README : la page des résultats de l'extrait en anglais.
  Chrome la capture sans fenêtre (`--headless=new`, 1440 × 1300) et en thème sombre, quel que soit celui du Mac (`preferredColorScheme=0`).

Les extraits normaux sont encodés par VideoToolbox, qui ne redonne pas les mêmes octets d'un passage à l'autre.
`extrait-lente.jpg`, `extrait-chauve-souris.jpg` et `extrait-boites.png` changent donc un peu à chaque passage sans que leur contenu change.
Les autres sont identiques au pixel près.
On ne les recommite que si le dessin a changé.

Les vues entières sont en JPEG q80 à 1200 px de large et les gros plans en PNG, tous hors LFS.
