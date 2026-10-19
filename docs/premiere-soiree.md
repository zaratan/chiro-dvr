# Première soirée : des jumelles aux résultats

Ce guide suit une soirée d'enregistrement sur un Mac, des jumelles jusqu'aux pistes à regarder.
Chaque étape dit ce qu'on fait et ce qu'on doit voir.
Les exemples prennent une soirée rangée dans un dossier `nuit-12-oct`, comme l'aide de la commande.

## 1. Avant la première fois

**Vérifier le Mac.**
Menu Pomme › À propos de ce Mac : la ligne « Puce » doit nommer une puce Apple (M1, M2…).
La version de macOS doit être 14 ou plus récente.

**Ouvrir le Terminal**, l'application où l'on tape les commandes.
Appuyer sur Cmd + Espace, taper `Terminal` puis Entrée.

**Installer Homebrew.** Il sert ensuite à installer l'outil.
Copier dans le Terminal la commande affichée en tête du site [brew.sh](https://brew.sh) puis appuyer sur Entrée.
L'installeur demande le mot de passe du Mac. Rien ne s'affiche pendant la frappe : c'est normal.
L'installation peut prendre plusieurs minutes.
Elle finit par « Next steps » : copier et lancer une par une les commandes affichées là.
Sans elles, le Terminal ne trouvera pas `brew`.

**Installer l'outil** : taper `brew install zaratan/bat-tools/batdetect` puis Entrée.
Taper ensuite `batdetect --version` pour vérifier : la première ligne doit dire « batdetect 0.3.0 ».

**Activer la touche Tab** (depuis la version 0.3.0). Elle complète les noms de vidéos et les options.
Copier dans le Terminal `echo 'autoload -Uz compinit && compinit' >> ~/.zshrc` puis appuyer sur Entrée.
Ouvrir ensuite une nouvelle fenêtre du Terminal (Cmd + N).

**Régler les jumelles.**
Les noms des réglages sont ceux du manuel des Pulsar Symbion DXT50.
L'outil a été mis au point sur des vidéos filmées avec ces quatre réglages :

- **Compression vidéo sur OFF** (menu Media).
  Sur une copie compressée, une chauve-souris de quelques pixels sortait souvent en deux taches.
- **Filtre de lissage sur OFF.** Il risque d'effacer les petits points.
- **Calibrage semi-automatique.**
  En automatique, l'image se fige environ une seconde à chaque calibrage. Un passage peut tomber dedans.
- **Pas de zoom numérique.**
  Les « 2.0x » et « 6.5x » affichés en bas de l'image sont les grossissements de base : ils ne signalent pas un zoom.

Trois autres conseils viennent du manuel et de la littérature. Ils n'ont pas encore été essayés sur ces jumelles :

- Régler le gain et le contraste à la main pour que le ciel ne sature pas.
  Un ciel tout noir est une zone où l'outil ne voit rien.
- Mettre la stabilisation d'image sur OFF. Elle est faite pour filmer à main levée.
- Garder la même luminosité et le même contraste d'une soirée à l'autre (option « Enregistrer les paramètres de l'image thermique »).
  Les soirées restent ainsi comparables.

Jusqu'ici, tout ne se fait qu'une fois.

**Vérifier que tout est prêt** (depuis la version 0.3.0).
La vérification porte sur le dossier où se trouve le Terminal : il faut d'abord l'y placer.
Le bon dossier est celui où les résultats seront écrits : le dossier de la soirée.
Avant la première soirée, c'est le dossier qui l'accueillera, comme le Bureau.
Taper `cd` suivi d'une espace. Glisser alors ce dossier depuis le Finder dans la fenêtre du Terminal, puis appuyer sur Entrée.
Taper ensuite `batdetect doctor` puis Entrée.
Une ligne qui commence par ✓ est bonne.
Sous une ligne ✗, la ligne « à faire » dit quoi faire.
Une ligne ⚠ ne bloque rien.
Cette vérification se refait de la même façon avant une soirée importante.

## 2. Filmer, puis brancher les jumelles

**Les jumelles restent fixes sur un trépied pendant tout l'enregistrement.**
Quand elles bougent, tout le fond bouge avec elles.
L'outil met alors ce moment hors analyse et n'y cherche rien.

Déclencher le calibrage juste avant d'enregistrer.
Cadrer de préférence la zone de passage devant la roche.
La roche garde la chaleur de la journée : une chauve-souris s'y détache en tache sombre.
Le ciel et la végétation sombre la font moins ressortir.
Commencer l'enregistrement au début de la sortie des chauves-souris : le fond change de température autour du coucher du soleil.
Ces deux derniers conseils viennent de la littérature et n'ont pas encore été essayés.

À la fin de la soirée, brancher les jumelles **allumées** au Mac par leur câble USB-C.
Sur les jumelles, choisir le mode **« Accès aux fichiers »** avec la bague puis valider par MENU.
Les jumelles apparaissent dans le Finder comme une clé USB (manuel des jumelles, section « Connexion USB »).

## 3. Copier les vidéos

Créer sur le Mac un dossier pour la soirée, par exemple `nuit-12-oct` sur le Bureau.
Dans le Finder, y glisser les vidéos de la soirée depuis les jumelles.

- **Jamais par l'application Stream Vision 2** : son export réencode les vidéos et divise leur débit par 12 environ.
  Sur une vidéo d'essai, l'export a fait perdre 2 des 14 pistes : deux chauves-souris que la copie d'origine montre.
- **Jamais directement sur les jumelles** : ne pas lancer l'outil sur leur volume.
  Les résultats s'écriraient sur leur carte.
  On travaille toujours sur la copie du Mac.

Les vidéos doivent être directement dans le dossier : l'outil ne regarde pas dans les sous-dossiers.
Il prend les fichiers `.mp4`, `.mov`, `.avi`, `.mkv` et `.m4v`.
Il laisse de côté sans rien dire les autres fichiers du dossier, comme les photos ou les notes.

**Prévoir de la place pour les résultats.**
Sauf choix d'un autre dossier, ils s'écrivent à côté des vidéos, sur le même disque.
Une soirée calme laisse des résultats plus légers que ses vidéos.
Une soirée très active peut demander jusqu'à sept fois la taille des vidéos.
La vidéo annotée (option `--annotated`) demande près de trois fois de plus.

## 4. Lancer

Dans le Terminal, **taper `batdetect` suivi d'une espace, puis glisser le dossier de la soirée** depuis le Finder dans la fenêtre du Terminal.
Son chemin s'écrit tout seul, par exemple `/Users/naturaliste/Desktop/nuit-12-oct` (le Bureau s'y écrit « Desktop »).
Appuyer sur Entrée.

Tapé seul sans dossier, `batdetect` rappelle les usages courants :

![Le Terminal après la commande batdetect seule : les usages courants](images/premiere-soiree-04-lancer.jpg)

Mieux vaut glisser le dossier que taper son nom.
Le Terminal s'ouvre dans votre dossier personnel, pas sur le Bureau.
Un nom tapé à la main y est cherché et n'est pas trouvé. Rien n'est alors traité :

> erreur : aucune vidéo n'est traitée :\
> nuit-12-oct : introuvable\
> à faire : vérifier le chemin nuit-12-oct ; le dossier ou la vidéo a peut-être été déplacé ou renommé

**La première fois, macOS peut demander** si le Terminal a le droit d'accéder au dossier : autoriser.
Si l'accès a été refusé, rien n'est traité :

> erreur : aucune vidéo n'est traitée :\
> nuit-12-oct : lecture refusée\
> à faire : vérifier les droits de /Users/naturaliste/Desktop/nuit-12-oct ; sur Mac, autoriser le Terminal dans Réglages Système, Confidentialité et sécurité, Fichiers et dossiers

Ouvrir alors Réglages Système › Confidentialité et sécurité › Fichiers et dossiers.
Y autoriser le Terminal pour ce dossier (le Bureau par exemple) puis relancer.

**Avant de calculer, l'outil ouvre chaque vidéo.**
Une vidéo illisible ou dont la copie a raté est refusée tout de suite, avec ce qu'il faut faire.
Elle ne fait donc pas échouer la soirée en son milieu :

> erreur : aucune vidéo n'est traitée :\
> video_093.mp4 : ce fichier n'est pas une vidéo lisible, ou sa copie est incomplète\
> à faire : s'il vient des jumelles, le recopier par le câble USB, sans passer par Stream Vision 2 ; sinon, le retirer du dossier

**Une entrée peut être refusée parmi d'autres**, par exemple un dossier sur plusieurs.
La commande la nomme avec la raison (introuvable, illisible…) et demande s'il faut traiter quand même le reste.
Entrée ou `o` lance le calcul et `n` l'arrête. Toute autre réponse repose la question :

> erreur : une entrée est refusée :\
> nuit-13-oct : introuvable\
> à faire : vérifier le chemin /Users/naturaliste/Desktop/nuit-13-oct ; le dossier ou la vidéo a peut-être été déplacé ou renommé\
> Traiter quand même la vidéo valide ? (O/n)

**Si le disque des résultats semble trop petit**, l'outil le dit avant de calculer.
Il nomme le dossier de résultats concerné et demande s'il faut lancer quand même :

> erreur : le disque des résultats semble trop petit :\
> le disque qui contient nuit-12-oct/batdetect : 6,4 Go à prévoir, 4,1 Go libres\
> à faire : libérer de la place, ou écrire les résultats sur un autre disque avec -o ; --ignore-disk lance quand même le calcul\
> Traiter quand même les 12 vidéos valides ? (O/n)

Répondre `o` lance la soirée.
Chaque vidéo vérifie encore la place avant d'écrire ses extraits.
S'il en manque, elle s'arrête sans remplir le disque et compte en échec. L'outil passe alors à la suivante.

## 5. Attendre

Le calcul commence par une ligne qui annonce la soirée, par exemple « 12 vidéos, 1 h 2 min 5 s de vidéo ».
Si la même commande a déjà tourné, elle dit aussi combien de vidéos sont faites : « 12 vidéos, 1 h 2 min 5 s de vidéo, dont 3 déjà faites ».

Pendant le calcul, une seule ligne se redessine sur place.
Elle donne le rang de la vidéo dans le lot, son nom puis l'étape en cours.
Par exemple `[1/1] video_092_original_3m24-4m05 — détection`, puis `suivi`, `image résumé` et `extraits`.
À la fin de chaque vidéo, sa ligne reste à l'écran avec son nombre de pistes. Puis vient le bilan de la soirée :

![Le Terminal à la fin du calcul : la dernière ligne de la vidéo et le bilan](images/premiere-soiree-05-attendre.jpg)

Pour chaque vidéo, le bilan donne sa durée, son nombre de pistes, le temps non analysé et le temps de calcul.
Viennent ensuite à part les vidéos qui ont trop de pistes, celles en échec et les entrées refusées.
Chacune porte ce qu'il faut faire.

Le Mac du développeur (un M1 Max) calcule une vidéo de 5 min en 20 s environ.
Cela fait à peu près 4 min par heure d'enregistrement.
Un Mac plus modeste met plus longtemps. Ce temps n'y a pas encore été mesuré.

Dès la première vidéo calculée, la ligne qui se redessine donne aussi l'heure à laquelle toute la soirée devrait être traitée.
Par exemple : `[3/12] video_094 — détection · fin vers 22:41`.
Cette heure est une estimation.
Elle part du temps qu'ont pris les vidéos déjà calculées pendant ce lancement et se corrige à chaque étape.
Une vidéo avec beaucoup de pistes a plus d'extraits à écrire et prend plus longtemps que les autres.

## 6. Ouvrir les résultats

À la fin du lot, une notification l'annonce et la page des résultats de chaque soirée s'ouvre dans le navigateur.
Elle montre toutes les vidéos de la soirée avec leur image résumé et leurs extraits.
Tout se regarde sans quitter la page ([La page des résultats](lire-les-resultats.md#la-page-des-résultats)).
Pour s'en passer, ajouter `--no-notify` à la fin de la commande, après le dossier.

La page s'appelle `resultats.html`. Elle se trouve dans le dossier `batdetect/`, à côté des vidéos.
Pour la rouvrir plus tard, double-cliquer dessus.
À côté d'elle, chaque vidéo a son dossier avec l'image résumé, le tableau des pistes et les extraits.
Leurs noms suivent la langue du Mac ([Où sont les résultats](lire-les-resultats.md#où-sont-les-résultats)).

Relancer la même commande sur le même dossier reprend un lot interrompu là où il s'était arrêté.
Les vidéos déjà faites restent telles quelles. Le bilan les compte « déjà faites » avec leurs chiffres d'origine.
Une vidéo est refaite si elle a changé.
Elle l'est aussi si les réglages, la version de l'outil ou la langue du Mac ne sont plus les mêmes.
`--force` refait tout.

Pour lire l'image résumé, juger un extrait et noter son verdict : [Lire les résultats](lire-les-resultats.md).
