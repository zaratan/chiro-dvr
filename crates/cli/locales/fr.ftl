too-many-tracks = { $video } : { $count } pistes, au-delà de { $limit } : { $annotated ->
        [yes] extraits et vidéo annotée non écrits
       *[no] extraits non écrits
    }
clips-redone = { $video } : { $names } { $count ->
        [one] réécrit
       *[other] réécrits
    } avec openh264 après une erreur de VideoToolbox : { $causes }
video-step = [{ $position }/{ $total }] { $video } — { $step ->
        [detection] détection
        [tracking] suivi
        [summary] image résumé
       *[clips] extraits
    }
video-skipped = [{ $position }/{ $total }] { $video } — déjà faite, { $tracks ->
        [one] { $tracks } piste
       *[other] { $tracks } pistes
    }
video-done = [{ $position }/{ $total }] { $video } — { $tracks ->
        [one] { $tracks } piste
       *[other] { $tracks } pistes
    }
video-failed = [{ $position }/{ $total }] { $video } — { $too-many ->
        [yes] trop de pistes
       *[no] échec
    }
failure = { $video } : { $cause ->
        [missing] fichier introuvable
        [read-refused] le système refuse la lecture de ce fichier
        [not-a-video] ce fichier n'est pas une vidéo lisible, ou sa copie est incomplète
        [unusable-name] le nom de ce fichier ne peut pas être lu par FFmpeg
        [damaged] la vidéo est abîmée ou incomplète
        [unsupported] le format de cette vidéo n'est pas géré
        [settings] les réglages ne conviennent pas à cette vidéo
        [disk-full] le disque est plein
        [forbidden] écriture refusée dans { $place }
        [in-use] { $place } est ouvert dans un autre programme
        [encoding] les extraits n'ont pas pu être écrits
        [internal] erreur interne
       *[unexpected] erreur inattendue
    }
failure-todo = {"  "}à faire : { $cause ->
        [missing] vérifier le chemin ; la vidéo a peut-être été déplacée ou renommée
        [read-refused] vérifier ses droits ; sur Mac, autoriser le Terminal dans Réglages Système, Confidentialité et sécurité, Fichiers et dossiers
        [not-a-video] s'il vient des jumelles, le recopier par le câble USB, sans passer par Stream Vision 2 ; sinon, le retirer du dossier
        [unusable-name] renommer la vidéo avec des lettres, des chiffres et des tirets
        [damaged] la recopier depuis les jumelles par le câble USB, sans passer par Stream Vision 2, puis relancer ; si elle reste abîmée, le signaler
        [unsupported] le signaler, avec le détail ci-dessous
        [settings] relancer sans --work-width ni --osd-region
        [disk-full] libérer de la place sur le disque de { $folder }, puis relancer
        [forbidden] vérifier les droits de ce dossier, ou en choisir un autre avec -o
        [in-use] le fermer, puis relancer
        [encoding] relancer ; si cela recommence, le signaler avec le détail ci-dessous
        [too-many-tracks] regarder l'image résumé ; si ce sont de vrais passages, relancer avec un --max-tracks plus grand
       *[unexpected] le signaler avec le détail ci-dessous
    }
failure-detail = {"  "}détail : { $detail }
summary-part = partie { $number } / { $count }
summary-tracks = { $count ->
        [one] { $count } piste
       *[other] { $count } pistes
    }
summary-not-analysed = hors analyse { $start } - { $end }
summary-more = + { $count } { $count ->
        [one] autre
       *[other] autres
    }
summary-unreadable = illisible { $start } - { $end }
summary-more-unreadable = + { $count } { $count ->
        [one] autre
       *[other] autres
    }, { $duration } { $seconds ->
        [one] illisible
       *[other] illisibles
    } en tout
band-not-analysed = hors analyse
band-unreadable = illisible
about = Détecte et suit les chauves-souris dans des vidéos de jumelles thermiques
inputs-help = Vidéos, ou dossiers dont toutes les vidéos sont traitées
out-dir-help = Dossier qui reçoit un dossier de résultats par vidéo
zoom-help = Extrait ralenti et agrandi : auto pour les pistes petites ou faibles seulement, all pour toutes, none pour aucune
annotated-help = Écrit aussi la vidéo entière avec toutes les pistes dessinées
max-tracks-help = Au-delà de ce nombre de pistes, la vidéo est comptée en échec et n'a ni extraits ni vidéo annotée ; 0 désactive
clip-margin-help = Temps gardé avant et après chaque piste dans son extrait ; secondes
osd-region-help = Cache à la détection un affichage à l'écran qui bouge, en fractions de l'image ; à répéter pour plusieurs zones
help-help = Affiche l'aide : -h l'essentiel, --help toutes les options
version-help = Affiche la version
no-region = aucune
beside-the-videos = batdetect/ à côté de chaque vidéo
threshold-help = Plus petit écart au fond qui compte comme une cible ; plus bas, il trouve des cibles plus faibles et plus de bruit ; niveaux de gris
min-hits-help = Les pistes qui ont moins de détections sont écartées ; plus bas, on garde des pistes plus courtes et plus de brefs scintillements
max-blobs-help = Les images qui ont au moins ce nombre de taches de plus que six fois le nombre habituel sont ignorées, comme quand les jumelles bougent ; 0 désactive
work-width-help = Largeur à laquelle les images sont réduites pour la détection ; plus large, on voit des cibles plus petites mais c'est plus long ; pixels
value-video = VIDÉO
value-folder = DOSSIER
value-which = CHOIX
value-count = NOMBRE
value-seconds = SECONDES
value-level = NIVEAU
value-pixels = PIXELS
usage-heading = Utilisation :
arguments-heading = Arguments{"\u00A0"}
options-heading = Options{"\u00A0"}
detection-heading = Réglages qui changent la détection{"\u00A0"}
default-value = [par défaut : { $value }]
possible-values = [valeurs possibles : { $values }]
error-label = erreur :
tip-label = conseil :
unexpected-argument = argument non reconnu : '{ $argument }'
similar-argument = un argument proche existe : '{ $suggestion }'
similar-value = une valeur proche existe : '{ $suggestion }'
argument-as-value = pour passer '{ $argument }' comme valeur, écrire '{ $escaped }'
invalid-value = valeur invalide '{ $value }' pour '{ $argument }'
region-shape = --osd-region doit être quatre fractions X0,Y0,X1,Y1 séparées par des virgules, pas { $value }
value-required = une valeur est attendue pour '{ $argument }', mais aucune n'est donnée
missing-arguments = arguments obligatoires manquants :
used-twice = l'argument '{ $argument }' ne peut être donné qu'une fois
unexpected-value = valeur inattendue '{ $value }' pour '{ $argument }' : aucune n'était attendue
invalid-utf8 = UTF-8 invalide dans un ou plusieurs arguments
arguments-refused = les arguments sont refusés
decimal-point = écrire { $written }, avec un point
more-information = Pour en savoir plus, essayez '{ $flag }'.
setting-refused = { $option } doit être { $bound ->
        [positive] un nombre fini > 0
        [not-negative] un nombre fini ≥ 0
        [at-least] un entier ≥ { $limit }
       *[up-to] un nombre fini entre 0 et { $limit }
    }, pas { $value }
region-refused = --osd-region { $index } ne vérifie pas 0 ≤ x0 < x1 ≤ 1 et 0 ≤ y0 < y1 ≤ 1 : { $region }
clip-margin-refused = --clip-margin doit être un nombre fini de secondes, au moins 0, pas { $value }
settings-refused = les réglages par défaut sont refusés ; le signaler
option-refused = { $option } ne peut pas s'appliquer ; le signaler
inputs-refused = aucune vidéo n'est traitée :
input-refused = { $path } : { $reason ->
        [missing] introuvable
        [refused] lecture refusée
        [not-a-folder] un élément du chemin n'est pas un dossier
       *[unavailable] inaccessible
    }
input-without-video = { $path } : { $folder ->
        [yes] aucune vidéo ({ $listed } ou { $last }) dans ce dossier
       *[no] pas une vidéo ({ $listed } ou { $last })
    }
shared-folder = { $path } : dossier de résultats commun à { $videos }
no-video = aucune vidéo ({ $listed } ou { $last }) trouvée dans { $count ->
        [one] ce dossier
       *[other] ces dossiers
    } :
folders-refused = aucune vidéo n'est traitée, les résultats ne peuvent pas s'écrire dans { $count ->
        [one] ce dossier
       *[other] ces dossiers
    } :
folder-refused = { $path } : { $access ->
        [refused] écriture refusée
        [not-a-folder] n'est pas un dossier
        [missing] introuvable
        [read-only] disque en lecture seule
        [disk-full] disque plein
       *[unavailable] inaccessible
    }
tally-title = Bilan : { $total ->
        [one] { $total } vidéo
       *[other] { $total } vidéos
    }
tally-already-done = { $count ->
        [one] { $count } déjà faite
       *[other] { $count } déjà faites
    }
tally-done = { $count ->
        [one] { $count } réussie
       *[other] { $count } réussies
    }
tally-too-many-count = { $count } avec trop de pistes
tally-failed-count = { $count } en échec
tally-video = vidéo
tally-length = durée
tally-tracks = pistes
tally-not-analysed = non analysé
tally-compute = calcul
tally-failed-cell = échec
tally-too-many-cell = { $count }, trop
tally-too-many-heading = Trop de pistes (tableau et image résumé écrits, pas d'extraits) :
tally-too-many = { $video } : { $count } pistes, au-delà de { $limit }
tally-failed-heading = { $failed ->
        [one] { $failed } vidéo
       *[other] { $failed } vidéos
    } sur { $total } en échec :
tally-kept-heading = { $count ->
        [one] Tableau laissé tel quel :
       *[other] Tableaux laissés tels quels :
    }
tally-kept = { $video } : { $table } n'a pas la forme d'un tableau des pistes{ $written ->
        [no] {"\u00A0"}; le nouveau tableau n'est pas écrit
       *[yes] {""}
    }
tally-unwritten-heading = { $count ->
        [one] Fichier non écrit, la page de résultats peut être incomplète :
       *[other] Fichiers non écrits, la page de résultats peut être incomplète :
    }
tally-set-aside-heading = { $count ->
        [one] Tableau mis de côté :
       *[other] Tableaux mis de côté :
    }
tally-set-aside = { $video } : { $table } { $reason ->
        [verdicts] porte des verdicts
       *[unknown] n'a pas la forme d'un tableau des pistes
    }, mis de côté sous { $dated }
tally-unwritten = { $taken ->
        [yes] { $path } existe déjà et n'est pas une page de batdetect : la renommer ou la déplacer pour avoir la page
       *[no] { $path } : { $detail }
    }
tally-changed = Réglages changés : { $settings }
tally-regions = { $count ->
        [one] { $count } zone
       *[other] { $count } zones
    }
tally-failure-todo = {"  "}à faire : { $cause ->
        [unsupported] le signaler, avec le détail écrit pendant le traitement
        [encoding] relancer ; si cela recommence, le signaler avec le détail écrit pendant le traitement
       *[unexpected] le signaler avec le détail écrit pendant le traitement
    }
common-uses = Usages courants :
use-folder = Traiter tout un dossier de soirée, avec les résultats dans { $folder }/ :
use-nights = Traiter plusieurs soirées d'un coup :
use-video = Traiter une seule vidéo :
use-out-dir = Écrire les résultats dans un autre dossier :
use-zoom = Faire un extrait ralenti et agrandi de chaque piste :
use-annotated = Écrire aussi la vidéo entière, avec toutes les pistes dessinées :
example-night = nuit-12-oct
example-next-night = nuit-13-oct
example-results = resultats
all-options = Toutes les options et les fichiers produits :
files-heading = Fichiers produits, dans { $folder }/<vidéo>/ à côté des vidéos, ou dans <dossier de -o>/<vidéo>/ :
file-video = <vidéo>
file-summary = L'image résumé : toutes les pistes de la vidéo sur une image
file-table = Une ligne par piste (début, fin, durée, vitesse, taille), pour un tableur
file-clip = L'extrait de chaque piste, nommé par son numéro et son début
file-zoom = Le même, ralenti et agrandi, pour les pistes petites ou faibles
file-annotated = La vidéo entière avec les pistes dessinées, avec --annotated
file-params = Les réglages et le bilan, écrit en dernier : sans lui, la vidéo est refaite à la relance
files-page = La page des résultats, à ouvrir dans le navigateur, est à côté des dossiers des vidéos : { $page }
files-long =
    Une vidéo longue ou chargée a plusieurs images résumé, par tranche de 10 min ({ $period })
    ou par partie ({ $part }).
guide = Mode d'emploi complet :
inputs-skipped = { $count ->
        [one] une entrée est refusée
       *[other] { $count } entrées sont refusées
    } :
skip-refused-question = Traiter quand même { $count ->
        [one] la vidéo valide
       *[other] les { $count } vidéos valides
    } ? (O/n)
skip-refused-declined = aucune vidéo n'est traitée
skip-refused-help = Traite les vidéos valides sans poser la question quand des entrées sont refusées
no-notify-help = Ne signale pas la fin du lot et n'ouvre pas la page de résultats (macOS)
force-help = Refait toutes les vidéos, même celles déjà faites avec les mêmes réglages
tally-refused-count = { $count ->
        [one] { $count } entrée refusée
       *[other] { $count } entrées refusées
    }
tally-refused-heading = { $count ->
        [one] Entrée refusée, non traitée
       *[other] Entrées refusées, non traitées
    } :
interrupted = lot interrompu à la vidéo { $position }/{ $total } ; relancer la même commande { $resumes ->
        [yes] reprend là où le lot s'est arrêté
       *[no] refait le lot
    }
table-track = piste
table-start = début
table-end = fin
table-duration = durée (s)
table-verdict = verdict
table-hits = points
table-chord = corde (px)
table-path = chemin (px)
table-speed = vitesse (px/s)
table-max-area = surface max (px)
table-amplitude = amplitude (niveaux de gris)
table-filled-frames = images comblées
table-start-seconds = début (s)
page-heading = Résultats de batdetect
page-folder = Dossier { $name }
page-videos = { $count ->
        [one] { $count } vidéo
       *[other] { $count } vidéos
    }
page-unfinished-count = { $count ->
        [one] { $count } inachevée
       *[other] { $count } inachevées
    }
page-waiting-count = { $count } en attente
page-reminder = Une piste est ce que l'outil a trouvé ; c'est à l'œil, sur l'extrait, que vous dites si c'est une chauve-souris.
page-expand = Tout déplier
page-collapse = Tout replier
page-date = { $day } { $month ->
        [1] janvier
        [2] février
        [3] mars
        [4] avril
        [5] mai
        [6] juin
        [7] juillet
        [8] août
        [9] septembre
        [10] octobre
        [11] novembre
       *[12] décembre
    } { $year }, { $hour }:{ $minute }
page-computed = calculée le { $date }
page-tried = essayée le { $date }
page-not-analysed = { $duration } non analysées
page-unfinished-badge = inachevée
page-too-many-badge = trop de pistes
page-older-badge = ancienne version
page-older = calculée par une version plus ancienne de l'outil ; relancer la même commande la refait
page-waiting-badge = en attente
page-unfinished = le calcul s'est arrêté avant la fin de cette vidéo ; relancer la même commande la refait
page-waiting = pas encore traitée par le calcul en cours
page-previous-results = résultats du calcul précédent
page-summary-alt = Image résumé de { $video } : { $tracks ->
        [one] { $tracks } piste
       *[other] { $tracks } pistes
    }, chaque trajectoire numérotée et colorée comme dans le tableau. Ouvrir en taille réelle (nouvel onglet)
page-part-alt = Image résumé de { $video }, partie { $part } sur { $parts } : pistes { $first } à { $last }, chaque trajectoire numérotée et colorée comme dans le tableau. Ouvrir en taille réelle (nouvel onglet)
page-part = Partie
page-parts = Parties de l'image résumé de { $video }
page-part-tab = Partie { $part } sur { $parts }, pistes { $first } à { $last }
page-column-track = Piste
page-column-start = Début
page-column-length = Durée
page-column-verdict = Verdict
page-column-watch = Voir
page-table-caption = Pistes de { $video }
page-no-verdict = à noter
page-clip = Extrait
page-zoom = Zoom
page-play-clip = Lire l'extrait de la piste { $track }
page-play-zoom = Lire le zoom ralenti de la piste { $track }
page-show-track = Montrer la piste { $track } sur l'image résumé
page-playing = Piste { $track } sur { $total } · début { $start } · { $zoom ->
        [yes] zoom ralenti
       *[no] extrait
    }
page-unreadable = extrait illisible
page-back = Revenir à l'image résumé
page-previous = Précédente
page-next = Suivante
page-previous-clip = Extrait de la piste précédente
page-next-clip = Extrait de la piste suivante
page-player = Lecteur des extraits de { $video }
page-table-link = Tableau des pistes, pour un tableur
page-annotated-link = Vidéo entière avec les pistes dessinées
