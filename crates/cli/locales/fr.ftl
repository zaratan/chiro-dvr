too-many-tracks = { $video } : { $count } pistes, au-delà de { $limit } : { $annotated ->
        [yes] ni extraits, ni image résumé, ni vidéo annotée
       *[no] ni extraits ni image résumé
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
announce = { $videos ->
        [one] { $videos } vidéo
       *[other] { $videos } vidéos
    }, { $length } de vidéo{ $done ->
        [0] {""}
        [one] , dont { $done } déjà faite
       *[other] , dont { $done } déjà faites
    }
finish = { $line } · fin vers { $time }
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
        [disk-full] libérer de la place sur le disque qui contient { $folder }, puis relancer
        [forbidden] vérifier les droits de ce dossier, ou en choisir un autre avec -o
        [in-use] le fermer, puis relancer
        [encoding] relancer ; si cela recommence, le signaler avec le détail ci-dessous
        [too-many-tracks] ouvrir le tableau des pistes et regarder la vidéo à quelques-uns de ses temps ; si ce sont de vrais passages, relancer avec un --max-tracks plus grand
        [input-missing] vérifier le chemin { $folder } ; le dossier ou la vidéo a peut-être été déplacé ou renommé
        [input-read-refused] vérifier les droits de { $folder } ; sur Mac, autoriser le Terminal dans Réglages Système, Confidentialité et sécurité, Fichiers et dossiers
        [unreachable] vérifier que le disque est toujours branché, puis relancer ; si cela recommence, le signaler
        [not-a-video-name] ne donner que des vidéos, ou des dossiers de vidéos
        [no-video] vérifier que { $folder } est le bon dossier : seules les vidéos posées directement dedans sont traitées, pas celles de ses sous-dossiers
        [shared-folder] renommer l'une de ces vidéos, ou les traiter en deux fois avec un -o différent
        [read-only] copier les vidéos sur le Mac, ou choisir un autre dossier avec -o
        [other-folder] choisir un autre dossier avec -o
        [disk-short] libérer de la place, ou écrire les résultats sur un autre disque avec -o ; --ignore-disk lance quand même le calcul
        [disk-short-annotated] libérer de la place, retirer --annotated, ou écrire les résultats sur un autre disque avec -o ; --ignore-disk lance quand même le calcul
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
region-refused = la zone n° { $index } de --osd-region ne vérifie pas 0 ≤ x0 < x1 ≤ 1 et 0 ≤ y0 < y1 ≤ 1 : { $region }
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
disk-short = le disque qui contient { $path } : { $needed } Go à prévoir, { $free } Go libres
disks-short = { $count ->
        [one] le disque des résultats semble trop petit
       *[other] les disques des résultats semblent trop petits
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
tally-too-many-heading = Trop de pistes (tableau écrit, ni extraits ni image résumé) :
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
skip-refused-help = Traite les vidéos valides sans poser la question quand des entrées sont refusées ou que le disque des résultats semble trop petit
no-notify-help = Ne signale pas la fin du lot et n'ouvre pas la page de résultats (macOS)
ignore-disk-help = Lance le calcul même quand le disque des résultats semble trop petit ; chaque vidéo vérifie encore la place avant ses extraits
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
page-refused-heading = { $count ->
        [one] Entrée refusée, non traitée
       *[other] Entrées refusées, non traitées
    }
page-refused-when = lancement du { $date }
page-refused-consequence = { $count ->
        [one] Rien n'a été fait pour elle. La corriger comme indiqué ci-dessous, puis relancer batdetect : les vidéos déjà faites ne seront pas refaites.
       *[other] Rien n'a été fait pour elles. Corriger chacune comme indiqué ci-dessous, puis relancer batdetect : les vidéos déjà faites ne seront pas refaites.
    }
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
page-scene-alt = Image résumé de { $video } : { $tracks ->
        [one] { $tracks } piste
       *[other] { $tracks } pistes
    }, chaque trajectoire numérotée et colorée comme dans le tableau
page-scene-part-alt = Image résumé de { $video }, partie { $part } sur { $parts } : pistes { $first } à { $last }, chaque trajectoire numérotée et colorée comme dans le tableau
page-scene-window = De { $start } à { $end }
page-scene-span = { $start } – { $end }
page-scene-nothing-not-analysed = rien hors analyse
page-scene-nothing-unreadable = rien d'illisible
page-scene-not-analysed = hors analyse { $periods }
page-scene-unreadable = illisible { $periods }
page-scene-more = + { $count } { $count ->
        [one] autre
       *[other] autres
    }
page-scene-complete = Ouvrir l'image complète
page-scene-new-tab = (nouvel onglet)
page-scene-complete-new-tab = Ouvrir l'image complète (nouvel onglet)
page-strip-start = Piste { $track } · { $start }
page-part = Partie
page-parts = Parties de l'image résumé de { $video }
page-part-tab = Partie { $part } sur { $parts }, pistes { $first } à { $last }
page-column-track = Piste
page-column-start = Début
page-column-length = Durée
page-column-verdict = Verdict
page-table-caption = Pistes de { $video }
page-player = Lecteur des extraits de { $video }
page-table-link = Tableau des pistes, pour un tableur
page-annotated-link = Vidéo entière avec les pistes dessinées
page-js-bar = Verdicts de la soirée
page-js-yes = Oui
page-js-no = Non
page-js-doubt = Doute
page-js-key-yes = O
page-js-key-no = N
page-js-key-doubt = D
page-js-value-yes = oui
page-js-value-no = non
page-js-value-doubt = ?
page-js-to-see = à voir
page-js-other = autre : { $value }
page-js-counts = Verdicts : { $yes } oui · { $no } non · { $doubt } doute · { $todo } à voir
page-js-judged = jugées : { $done } sur { $total }
page-js-pending = ● Changements non téléchargés : { $count }
page-js-downloaded = Téléchargé à { $time } : à glisser à côté de cette page
page-js-current = Verdicts à jour
page-js-download = Télécharger les verdicts
page-js-reload = Recharger un fichier de verdicts…
page-js-read-from = Verdicts lus dans { $name }, du { $date } ; fichiers de verdicts dans le dossier : { $count }
page-js-read-file = Verdicts lus dans { $name }, du { $date }
page-js-steps-heading = Dernière étape : poser le fichier à côté de la page
page-js-step-one = Le fichier verdicts.csv est dans Téléchargements ; le navigateur l'a peut-être appelé verdicts (1).csv ou verdicts-2.csv. Si le navigateur demande où l'enregistrer, choisir le dossier { $folder }.
page-js-step-two = Le glisser dans le dossier { $folder }, à côté de cette page. S'il y en a plusieurs, la page lit le plus récent.
page-js-understood = Compris
page-js-replace-unsaved = Des changements ne sont pas téléchargés. Les remplacer par ce fichier ?
page-js-not-verdicts = { $name } n'est pas un fichier de verdicts.
page-js-draft-same = Verdicts non téléchargés depuis { $time }, repris dans la page.
page-js-draft-other = Des verdicts n'ont pas été téléchargés (depuis { $time }), sur un autre fichier de verdicts que celui que la page vient de lire.
page-js-draft-keep = Les reprendre
page-js-draft-drop = Les abandonner
page-js-orphans = Verdicts sans piste : { $count }
page-js-orphans-why = Ces verdicts ne correspondent plus à aucune piste depuis le dernier calcul. Ils restent dans le fichier de verdicts ; vérifier sur l'extrait la piste la plus proche.
page-js-orphan = { $start }, ancienne piste { $track } : { $verdict }
page-js-nearest = Voir la piste la plus proche (piste { $track }, { $start })
page-js-absent = Verdicts de vidéos absentes de la page, gardés dans le fichier : { $count }
page-js-track-of = Piste { $track } sur { $total } · { $start }
page-js-judge-track = Juger la piste { $track }
page-js-announce = Piste { $track } : { $verdict }
page-js-announce-species = Piste { $track } : { $verdict }, { $species }
page-js-species = espèce
page-js-species-then-enter = espèce, puis Entrée
page-js-species-of = Espèce de la piste { $track }
page-js-verdict-of = Verdict de la piste { $track }
page-js-view = Vue
page-js-clip = Extrait
page-js-zoom = Zoom
page-js-no-zoom = pas de zoom
page-js-no-clip = Pas d'extrait pour cette piste.
page-js-picture = Image résumé
page-js-chain = Enchaîner
page-js-previous-track = Piste précédente
page-js-next-track = Piste suivante
page-js-next-to-see = À voir »
page-js-blocked = Le navigateur a bloqué la lecture : cliquer sur ▶
page-js-unreadable = Extrait illisible.
page-js-changed = modifié
page-js-column-video = vidéo
page-js-column-track = piste
page-js-column-start = début
page-js-column-start-seconds = début (s)
page-js-column-verdict = verdict
page-js-column-species = espèce
page-js-column-start-x = départ x (px)
page-js-column-start-y = départ y (px)
page-js-separator = ;
page-js-decimal-mark = ,
page-js-key-species = E
page-js-draft-drop-confirm = Abandonner ces verdicts non téléchargés ? Ils seront perdus.
page-js-download-now = Télécharger maintenant
page-js-cleared = Piste { $track } : verdict effacé
page-js-value-yes-other = yes
page-js-value-no-other = no
other-commands = Autres commandes : « batdetect doctor » vérifie que tout est prêt avant une soirée, « batdetect completion zsh » écrit la complétion de la touche Tab. Un dossier nommé doctor ou completion se donne en ./doctor ou ./completion.
completion-zsh-only = seul zsh est pris en charge : batdetect completion zsh
completion-with-homebrew = Avec Homebrew, le script est déjà installé : ajouter à ~/.zshrc la ligne
completion-without-homebrew = Sans Homebrew : enregistrer le script par
completion-then-profile = puis ajouter à ~/.zshrc ces deux lignes, dans cet ordre :
doctor-no-argument = doctor ne prend aucun argument : le lancer dans le dossier à vérifier
doctor-version = batdetect { $version } — FFmpeg { $ffmpeg } inclus
doctor-hardware = ✓ encodeur : VideoToolbox (matériel)
doctor-software = ✓ encodeur : openh264 (logiciel)
doctor-fallback = ⚠ encodeur : openh264 seulement, VideoToolbox ne répond pas
doctor-broken = ✗ encodeur : aucun ne fonctionne
doctor-space = ✓ place libre : { $gigabytes } Go dans { $folder }
doctor-space-short = ✗ place libre : { $gigabytes } Go dans { $folder }, moins de { $least } Go
doctor-space-unknown = ⚠ place libre : non mesurée dans { $folder }
doctor-writable = ✓ écriture : permise dans { $folder }
doctor-unwritable = ✗ écriture : refusée dans { $folder } ({ $access ->
        [refused] droits insuffisants
        [not-a-folder] pas un dossier
        [missing] introuvable
        [read-only] lecture seule
        [disk-full] disque plein
       *[unavailable] inaccessible
    })
doctor-todo-fallback = {"  "}à faire : rien d'obligatoire, les extraits seront plus lents à écrire ; redémarrer le Mac peut rétablir VideoToolbox
doctor-todo-broken = {"  "}à faire : réinstaller batdetect
doctor-todo-space = {"  "}à faire : libérer de la place, ou choisir un autre dossier avec -o
doctor-todo-writing = {"  "}à faire : choisir un dossier où l'écriture est permise, avec -o
doctor-ready = Tout est prêt.
doctor-problems = { $count ->
        [one] 1 point à corriger.
       *[other] { $count } points à corriger.
    }
doctor-untried = ✗ encodeur : non essayé, le dossier temporaire du système refuse l'écriture
doctor-todo-untried = {"  "}à faire : libérer de la place sur le disque du système, puis relancer batdetect doctor
completion-commands-group = commande
completion-doctor = vérifie que tout est prêt avant une soirée
completion-completion = écrit la complétion de la touche Tab
