# Banc #347 sur le Mac

Au calme, sans autre calcul lourd. Le serveur pose COOP et COEP.

1. Lance le serveur : `cd /Users/zaratan/Projects/dvr-wt/rapports/347/mac && node server.mjs . 8347 isolated`
2. Ouvre `http://127.0.0.1:8347/index.html` dans Safari, puis dans Chrome, puis dans Firefox.
3. La ligne du haut dit le navigateur et `crossOriginIsolated`. Le bouton « threads » est grisé s'il vaut `false`.
4. Clique les cinq boutons l'un après l'autre, en attendant chaque résultat.
5. Copie le texte gris de chaque navigateur dans `rapports/mac-347-resultats.md`. Les lignes « identiques » comparent l'extrait au binaire et à FFmpeg.
6. Extrait du navigateur, image par image avec les flèches : `open -a "QuickTime Player" ../browser-clip.mp4 && open -a VLC ../browser-clip.mp4`
7. Zip : `open -a "Archive Utility" ../batdetect.zip`, puis ouvre `resultats.html` du dossier dézippé dans Safari.
8. Dans cette page, clique une piste, tape « o », puis « Télécharger les verdicts ».
