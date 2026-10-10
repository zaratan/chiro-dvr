# Lot 3 : ergonomie pour la naturaliste — interview et décisions (9 octobre 2026)

Lot sans code : interview de l'utilisateur, audit du code et des docs par agents de lecture, rédaction
d'issues nouvelles. Aucune issue existante n'est traitée. Label `ergonomie`, jalon « Prise en main
naturaliste », toutes en Backlog.

## État mesuré (main 742668b, extrait `tests/fixtures/video_092_original_3m24-4m05.mp4`, mode normal)

- 22,45 s réels, 65 s CPU, 702 Mo résidents ; 4 pistes.
- Écran pendant l'exécution : rien pendant 22 s, puis 5 lignes (bilan de la vidéo + une par piste).
- `--help` : 31 options, en anglais, 7 sans description, aucune valeur par défaut affichée, aucune
  mention des fichiers produits. `batdetect` sans argument : erreur argparse.
- Dossier produit : `<nom>/<nom>.tracks.png`, `<nom>.tracks.csv`, `params.json`, `split/NN_MmSSsCC.mp4`.
- README : installation Homebrew, sources, utilisation, modes avec tableau de mesures par vidéo.

## Interview (réponses de l'utilisateur)

**Qui.** Manon avec l'utilisateur au début, puis seule ; d'autres naturalistes ensuite. Elle tape et
adapte une commande si l'aide le lui dit.

**Séance.** Une soirée entière d'un coup (10 à 15 clips de 5 min), parfois plusieurs soirées
accumulées ; lancé et laissé tourner. Premier regard : l'image résumé, puis les extraits un par un.

**Pendant le calcul.** Vidéo en cours + barre + temps restant, avec estimation globale pour la soirée ;
étapes nommées (lecture, détection, suivi, extraits). Pas de pistes au fil de l'eau.

**Bilan.** Les quatre : tableau final dans le terminal (par vidéo : pistes, durée, échec, chemin) ;
fichier bilan dans le dossier de sortie, **page HTML** sans serveur (tableau par vidéo, image résumé,
liens vers les extraits) ; notification macOS à la fin ; le dossier de sortie s'ouvre dans le Finder.

**Erreurs.** Une vidéo en échec n'arrête pas la soirée : continuer, et le dire clairement à la fin
(vidéo, raison en clair, quoi faire).

**Relance.** Sauter les vidéos déjà traitées (sortie existante avec les mêmes réglages) ; une option
force le recalcul.

**Langue.** Français ou anglais selon la locale, pour l'aide, les messages, le bilan, **et les noms de
fichiers et dossiers produits**.

**Aide.** Chaque option a une description et sa valeur par défaut. Plusieurs sections : un tl;dr au
début pour les usages courants (sur le modèle de la commande `tldr`), puis l'aide complète.
`batdetect` sans argument affiche le tl;dr.

**Complétion.** zsh installée par Homebrew ; valeurs complétées (`--mode`, `--zoom`, chemins vidéo) ;
sous-commande `completion` pour l'installer soi-même. bash/fish non retenus.

**Santé.** `batdetect doctor` (ffmpeg/ffprobe, encodeur, espace disque, quoi faire) ; contrôle de
l'espace disque avant de lancer. Pas d'avertissement de nouvelle version.

**Fichiers produits.** Noms parlants, selon la locale (`split/` → `extraits/`, `tracks.csv` →
`pistes.csv`, `01_0m04s80.mp4` → nom avec « passage ») ; sortie par défaut à côté des vidéos (dossier
`batdetect/` ou équivalent) plutôt que dans le dossier courant ; CSV lisible dans un tableur
(en-têtes selon la locale, temps en mm:ss, colonne verdict vide, point-virgule en FR). Pas de dossier
par soirée.

**README et docs.** Naturaliste d'abord, développeur ailleurs (sources, tests, licences dans docs/ ou
CONTRIBUTING.md) ; parcours « première soirée » pas à pas (jumelles, USB, Terminal, lancer, attendre,
ouvrir) ; tableaux de mesures sortis du README vers docs/ ; une section « techniques utilisées » en
mode simple dans docs/10. Lecture des résultats : lire l'image résumé, valider un extrait (faux
positifs typiques, où noter le verdict), glossaire, ce que l'outil ne voit pas.

**Ordre.** 1. Avancement et bilan. 2. Aide, langue, complétion, doctor. 3. Fichiers produits.
4. README et docs.

## Issues existantes voisines (ne pas dupliquer)

#6 (feuille de verdict par vidéo), #10 (temps sur la machine de Manon), #13 (couleurs des extraits
= image résumé), #28 (vol rapide vs lent dans le livrable), #3 (fait : `--max-tracks`), #38 (fait :
`--mode`), #30 (fait : Homebrew).

## Décisions après audit (9 octobre 2026, 38 questions des agents)

**Transverses.** Entrée fautive au départ (chemin absent, non-vidéo) : refus avant tout calcul ; une
vidéo qui casse en cours de calcul n'arrête pas la soirée. Aucune nouvelle dépendance Python : barre à
la main, table déclarative des options pour la complétion, gettext de la bibliothèque standard avec un
script du dépôt pour compiler le `.mo`. `doctor` et `completion` par aiguillage dans `main`,
`batdetect <vidéos>` inchangé. Vocabulaire : **piste** = sortie de l'outil, **passage** = confirmé par
Manon ; le bandeau de l'image résumé dit « pistes » ; point = une détection sur une image.

**Langue.** Anglais quand aucune variable de locale n'est posée. Virgule décimale dans les messages en
français. Docs naturaliste en français seulement.

**Fin de soirée.** Notification et ouverture du dossier dans le Finder actifs par défaut, sur macOS et
quand la sortie est un terminal ; `--no-notify` pour les éteindre. Le dossier s'ouvre, pas la page HTML.

**Reprise.** Recalcul si : version de batdetect différente, réglages de détection ou de suivi
différents (réglage d'origine comparé, `auto` et non `videotoolbox`), même nom mais taille différente,
ou réglage de rendu seul différent (tout recalculer, pas de rendu seul). `params.json` écrit en dernier
avec version et état : il marque le dossier terminé. Page HTML : vidéos en échec incluses avec la raison
(état gardé à la racine de la sortie) ; un bilan par dossier source, le tableau du terminal couvre tout.

**CSV.** `pistes.csv` devient la feuille de verdict de #6 (#6 se resserre sur relecture et taux). Au
recalcul, un CSV qui porte des verdicts est mis de côté sous un nom daté et signalé. Temps `mm:ss` sans
centièmes dans début/fin (`start_s` garde les centièmes). Colonnes techniques gardées en fin de tableau,
en-têtes traduits avec l'unité.

**Noms.** Extraits `piste_01_0m04s.mp4`, sans le nom de la vidéo. Image résumé et CSV préfixés du nom
de la vidéo. `params.json` et le dossier `batdetect/` ne se traduisent pas. La langue du dernier calcul
fait foi (renommage et ménage des deux jeux de noms).

**Refus.** Source non inscriptible : arrêt avant calcul, message qui dit `-o` ou de copier. Disque :
refus si l'estimation dépasse la place libre (1,05× l'entrée, +1,7× avec `--annotated`), option pour
passer outre, avertissement sous une marge.

**Aide et complétion.** `batdetect completion zsh` affiche les lignes à ajouter et ne touche pas
`~/.zshrc`. `batdetect` sans argument : tl;dr, code 0.

**Docs.** Mesures du README : tableau des pistes vers docs/08, temps vers docs/09 (écart 437 s contre
373–386 s signalé). `CONTRIBUTING.md` à la racine pour le développeur. Pages naturaliste
`docs/premiere-soiree.md` et `docs/lire-les-resultats.md`, sans numéro, en tête du sommaire sous « Pour
utiliser l'outil ». Section « techniques en simple » en tête de docs/10, après « L'idée en une phrase ».
Captures : extrait de test, affichage des jumelles masqué, accord de Manon à demander, `docs/images/`
hors LFS. Faux positifs typiques : premier jet d'après docs/03 et docs/06, relu par Manon.

**Détails fixés après alignement.** `_zoom` gardé, `_boxes` → `_annotee` (fr) / `_annotated` (en). Disque :
marge 10 % ou 1 Go (le plus grand), option `--ignore-disk`. `doctor` sans vidéo mesure le dossier courant.
`completion zsh` : script sur stdout, lignes de profil sur stderr. Issue ajoutée : bandeau de l'image résumé
en « pistes » (`fichiers-06`, 1 point).

## Issues créées (9 octobre 2026)

Label `ergonomie`, jalon « Prise en main naturaliste » (n° 1), toutes en Backlog avec leurs points.
Aide : #54 options décrites, #55 messages d'erreur, #56 tl;dr, #57 langue, #58 doctor, #59 complétion zsh,
#60 formule, #61 espace disque. Avancement : #62 annonce de départ, #63 vidéo et étape, #64 raisons en
clair, #65 bilan terminal, #66 dossier terminé, #67 barre, #68 heure de fin, #69 notification et Finder,
#70 relance, #71 page HTML. Docs : #72 sommaire, #73 mesures, #74 CONTRIBUTING, #75 techniques en simple,
#76 glossaire, #77 image résumé, #78 limites, #79 valider un extrait, #80 première soirée, #81 README.
Fichiers : #82 écriture vérifiée, #83 colonne verdict, #84 CSV tableur, #85 noms selon la langue,
#86 sortie à côté des vidéos, #87 bandeau « pistes ». Textes sources : `~/Projects/dvr-wt/rapports/lot3-issues/`.

**Suite (9 octobre 2026, soir).** Accord de Manon reçu pour les captures (#77, #79, #80 commentées). Priorités
posées par valeur perçue pour Manon : P1 #57 #63 #64 #65 #67 #69 #70 #71 #77 #79 #80 #81 #86 ; P2 #54 #55 #56 #58 #59
#61 #62 #66 #68 #75 #76 #78 #82 #83 #84 #85 #87 ; P3 #60 #72 #73 #74.

**Verdict de Manon sur la 092 (9 octobre 2026, soir).** 13 chauves-souris sur 14 pistes ; seule la piste lente
3:29 → 3:39 n'en est pas une, et le « 3:36 » de septembre était cette chose ; 0:51 est une chauve-souris en bordure
droite. Règle : mieux vaut un faux positif facile à écarter qu'un passage manqué (docs/02, CLAUDE.md). Manon n'a
rien de formel pour compter : la colonne de #83 devient une commodité de comptage, #6 fermée, docs/03 et docs/06
mis à jour, commentaires sur #25, #78, #79.
