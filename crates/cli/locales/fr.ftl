processed = { $video } : { $tracks ->
        [one] { $tracks } piste
       *[other] { $tracks } pistes
    } -> { $dest }
too-many-tracks = { $video } : { $count } pistes, au-delà de { $limit } : { $annotated ->
        [yes] extraits et vidéo annotée non écrits
       *[no] extraits non écrits
    }
clips-redone = { $video } : { $names } { $count ->
        [one] réécrit
       *[other] réécrits
    } avec openh264 après une erreur de VideoToolbox : { $causes }
table-kept = { $video } : { $table } laissé tel quel, il n'a pas la forme d'un tableau des pistes{ $written ->
        [no] {" "}; le nouveau tableau n'est pas écrit
       *[yes] {""}
    }
failed = { $video } : échec : { $causes }
internal-error = { $video } : erreur interne : { $message }
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
invalid-value-because = valeur invalide '{ $value }' pour '{ $argument }' : { $cause }
value-required = une valeur est attendue pour '{ $argument }', mais aucune n'est donnée
missing-arguments = arguments obligatoires manquants :
used-twice = l'argument '{ $argument }' ne peut être donné qu'une fois
unexpected-value = valeur inattendue '{ $value }' pour '{ $argument }' : aucune n'était attendue
invalid-utf8 = UTF-8 invalide dans un ou plusieurs arguments
arguments-refused = les arguments sont refusés : { $cause }
more-information = Pour en savoir plus, essayez '{ $flag }'.
setting-refused = { $option } doit être { $bound ->
        [positive] un nombre fini > 0
        [not-negative] un nombre fini ≥ 0
        [at-least] un entier ≥ { $limit }
       *[up-to] un nombre fini entre 0 et { $limit }
    }, pas { $value }
region-refused = --osd-region { $index } ne vérifie pas 0 ≤ x0 < x1 ≤ 1 et 0 ≤ y0 < y1 ≤ 1 : { $region }
clip-margin-refused = --clip-margin doit être un nombre fini de secondes, au moins 0, pas { $value }
settings-refused = les réglages sont refusés : { $causes }
no-video = aucune vidéo trouvée
videos-refused = vidéos refusées : { $causes }
