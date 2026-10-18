# batdetect

## Ce que fait l'outil, et ce qu'il ne fait pas

`batdetect` cherche ce qui bouge dans les vidéos des jumelles thermiques et suit chaque trajectoire : c'est une piste.
Pour chaque vidéo, il donne une image qui montre toutes les pistes, un tableau qui les décrit et un court extrait vidéo par piste, sur lequel on juge, à l'œil, si c'est une chauve-souris.

Il ne reconnaît pas une chauve-souris, et ne compte pas les passages, les vols de chauve-souris confirmés à l'œil : une piste peut être un autre animal ou du bruit d'image, un passage peut être manqué ou coupé en deux pistes.
Il ne regarde pas les moments où les jumelles bougent.
Ce qu'il voit mal, et quoi faire, est dit dans [ce que l'outil ne voit pas](docs/lire-les-resultats.md#ce-que-loutil-ne-voit-pas).

## Installer

Sur un Mac à puce Apple (M1 ou plus récent) en macOS 14 ou plus récent, avec [Homebrew](https://brew.sh), taper dans le Terminal :

```bash
brew install zaratan/bat-tools/batdetect
```

Depuis la version 0.2.1, pour vérifier que tout est prêt avant une soirée, taper `batdetect doctor` dans le dossier où les résultats seront écrits.
Chaque ligne commence par ✓ quand tout va bien ; sous une ligne ✗, la ligne « à faire » dit quoi faire ; une ligne ⚠ ne bloque rien.

Depuis la version 0.2.1, pour que la touche Tab complète les noms de vidéos et les options, copier une fois dans le Terminal `echo 'autoload -Uz compinit && compinit' >> ~/.zshrc`, puis Entrée, puis ouvrir une nouvelle fenêtre du Terminal.
Sans Homebrew, `batdetect completion zsh` affiche les étapes à suivre.

Sous Linux et Windows, télécharger l'archive de son système sur la [page des versions](https://github.com/zaratan/chiro-dvr/releases/latest), puis la décompresser :

- Linux : `batdetect-linux-x86_64.tar.gz`, pour une glibc 2.35 ou plus récente (Ubuntu 22.04 et après) ;
- Windows 10 ou plus récent : `batdetect-windows-x86_64.zip`.
  Au premier lancement, SmartScreen peut bloquer le programme : « Informations complémentaires », puis « Exécuter quand même ».

Sous Linux et Windows, le processeur doit être de niveau x86-64-v2.

Sur Mac, l'archive `batdetect-darwin-arm64.tar.gz` remplace Homebrew si besoin. macOS bloque alors le programme téléchargé : après l'avoir décompressé, taper `xattr -d com.apple.quarantine` suivi d'une espace, glisser le programme `batdetect` dans le Terminal, puis Entrée.

## Lancer une soirée

1. Copier les vidéos des jumelles dans un dossier de l'ordinateur, par le câble USB, jamais par l'export de Stream Vision 2.
2. Ouvrir le Terminal.
3. Taper `batdetect` suivi d'une espace, puis glisser le dossier dans la fenêtre du Terminal.
4. Appuyer sur Entrée et attendre le bilan de la soirée.
5. Ouvrir le dossier `batdetect/` créé à côté des vidéos ; sur Mac, il s'ouvre tout seul.

Avec une archive plutôt qu'Homebrew, glisser d'abord le programme `batdetect` (`batdetect.exe` sous Windows) dans le Terminal, à la place de taper son nom.

Chaque étape en détail, avec ce qu'on doit voir : [première soirée](docs/premiere-soiree.md).
Le temps de calcul dépend de l'ordinateur ([mesures](docs/09-profilage.md)).

## Ce qu'on obtient

Pour chaque vidéo, un dossier à son nom dans `batdetect/` : l'image résumé, qui montre toutes les pistes sur la scène (plusieurs images pour une longue vidéo) ; le tableau des pistes, à ouvrir dans un tableur ; un extrait par piste, et un extrait ralenti et agrandi pour les pistes petites ou faibles.
À la fin, le Terminal affiche le bilan de la soirée et la page des résultats, `batdetect/resultats.html`, s'ouvre dans le navigateur : toute la soirée, images et extraits compris, sur une seule page, où l'on note les verdicts piste par piste.
Comment lire tout cela, juger un extrait et noter son verdict : [lire les résultats](docs/lire-les-resultats.md).

## Choisir un mode

Il n'y a pas de mode à choisir : l'outil est réglé pour préférer une piste de trop à un passage manqué, et ses réglages par défaut servent pour toutes les soirées.
Les quelques réglages qui changent la détection sont listés, avec leur valeur par défaut, par `batdetect --help`, sous « Réglages qui changent la détection » (« Settings that change the detection » en anglais).

## Pour aller plus loin

Construire l'outil depuis les sources, le vérifier ou publier une version : [CONTRIBUTING.md](CONTRIBUTING.md).
Ce qu'on a appris en le construisant, et pourquoi il est réglé ainsi : [la base de connaissances](docs/README.md).

Le code est sous licence MIT ([LICENSE](LICENSE)).
Le programme publié contient FFmpeg, sous LGPL 2.1 ou ultérieure, openh264 (BSD-2), la police Roboto (OFL-1.1) et des bibliothèques Rust : [packaging/NOTICE](packaging/NOTICE) les décrit, et leurs textes de licence sont dans [packaging/licenses/](packaging/licenses/).
