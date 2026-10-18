# Images des docs

Ces images illustrent les docs pour la naturaliste ([première soirée](../premiere-soiree.md), [lire les résultats](../lire-les-resultats.md)) ; pour les refaire toutes, lancer `tools/docs-images.sh` depuis la racine du dépôt.
Le script traite l'extrait de test `tests/fixtures/video_092_original_3m24-4m05.mp4` dans `out/docs-images/`, en français, puis dessine les repères avec `magick`.
`resume-extrait.jpg` et `resume-frise.png` viennent de son image résumé, affichage masqué (`--osd-region 0,0,1,0.07`) ; `extrait-lente.jpg`, `extrait-chauve-souris.jpg`, `extrait-zoom.jpg` et `extrait-boites.png` sont des images tirées par `ffmpeg` de ses extraits et zooms (`--zoom all`) ; `premiere-soiree-*.jpg` dessinent une fenêtre de Terminal à partir des sorties de la commande, recopiées dans le script, à mettre à jour quand un texte de la commande change.
`results-page.jpg`, en tête du README, est la page des résultats de l'extrait en anglais, capturée par Chrome sans fenêtre (`--headless=new`, 1440 × 1300), en thème sombre quel que soit celui du Mac (`preferredColorScheme=0`).
Les coordonnées des repères et les numéros d'image valent pour l'extrait de test.
Les extraits normaux sont encodés par VideoToolbox, qui ne redonne pas les mêmes octets d'un passage à l'autre : `extrait-lente.jpg`, `extrait-chauve-souris.jpg` et `extrait-boites.png` changent un peu à chaque passage, sans que leur contenu change, alors que les autres sont identiques au pixel près ; ne les recommiter que si le dessin a changé.
Vues entières en JPEG q80 à 1200 px de large, gros plans en PNG, hors LFS.
