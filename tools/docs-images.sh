#!/usr/bin/env bash
set -euo pipefail

repository=$(cd "$(dirname "$0")/.." && pwd -P)
cd "$repository"
export LC_ALL=fr_FR.UTF-8

images=docs/images
work=out/docs-images
font=crates/draw/fonts/Roboto.ttf
extract=tests/fixtures/video_092_original_3m24-4m05.mp4
mkdir -p "$images" "$work"

batdetect() {
  LANGUAGE=fr cargo run --quiet --release -p batdetect -- "$@" --no-notify
}

reperes=()
width=0
height=0
repere() {
  local n=$1 x=$2 y=$3 tx=$4 ty=$5
  reperes+=(-stroke black -strokewidth 8 -draw "line $x,$y $tx,$ty"
    -stroke white -strokewidth 3 -draw "line $x,$y $tx,$ty"
    -stroke black -strokewidth 3 -fill white
    -draw "roundrectangle $((x - 22)),$((y - 22)) $((x + 22)),$((y + 22)) 6,6"
    -stroke none -fill black -gravity Center
    -annotate "$(printf '%+d%+d' $((x - width / 2)) $((y - height / 2)))" "$n" -gravity NorthWest)
}

summary_images() {
  batdetect "$extract" --osd-region 0,0,1,0.07 -o "$work/resume"
  local resume=$work/resume/video_092_original_3m24-4m05/video_092_original_3m24-4m05_resume.png
  reperes=() width=1949 height=1215
  repere 1 1915 44 1866 44
  repere 2 1635 82 1562 82
  repere 3 1650 120 1594 120
  repere 4 1660 250 1612 250
  repere 5 560 560 600 628
  repere 6 170 700 106 772
  repere 7 990 375 918 428
  repere 8 720 38 720 38
  repere 9 1500 1140 1426 1130
  magick "$resume" -background 'rgb(32,32,32)' -gravity East -splice 70x0 -gravity NorthWest \
    -stroke white -strokewidth 3 -draw 'line 1612,148 1612,352' \
    -font "$font" -pointsize 32 "${reperes[@]}" \
    -resize 1200x -quality 80 "$images/resume-extrait.jpg"
  reperes=() width=1520 height=215
  repere 10 800 30 800 76
  repere 11 1480 115 1424 115
  repere 12 1040 172 977 137
  repere 13 290 170 226 168
  magick "$resume" -crop 1440x135+0+1080 +repage -background 'rgb(32,32,32)' \
    -gravity North -splice 0x60 -gravity South -splice 0x20 -gravity East -splice 80x0 \
    -gravity NorthWest -font "$font" -pointsize 32 "${reperes[@]}" \
    "$images/resume-frise.png"
}

clip_frame() {
  local clips=$work/extraits/video_092_original_3m24-4m05/extraits
  ffmpeg -v error -y -i "$clips/$1.mp4" -vf "select=eq(n\,$2)" -frames:v 1 "$work/$1_$2.png"
}

clip_images() {
  batdetect "$extract" --zoom all -o "$work/extraits"
  clip_frame piste_01_0m04s 250
  clip_frame piste_01_0m04s_zoom 250
  local n
  for n in 52 53 54 56; do
    clip_frame piste_02_0m23s "$n"
  done
  magick "$work/piste_01_0m04s_250.png" -resize 1200x -quality 80 "$images/extrait-lente.jpg"
  magick "$work/piste_01_0m04s_zoom_250.png" -resize 1200x -quality 80 "$images/extrait-zoom.jpg"
  reperes=() width=1440 height=1080
  repere 1 1160 720 1083 660
  repere 2 960 580 1040 610
  repere 3 1320 470 1236 440
  repere 4 240 110 290 58
  magick "$work/piste_02_0m23s_56.png" -font "$font" -pointsize 32 "${reperes[@]}" \
    -resize 1200x -quality 80 "$images/extrait-chauve-souris.jpg"
  magick \( "$work/piste_02_0m23s_52.png" -crop 130x120+1160+390 \) \
    \( "$work/piste_02_0m23s_53.png" -crop 130x120+1115+445 \) \
    \( "$work/piste_02_0m23s_54.png" -crop 130x120+1075+500 \) \
    +repage -filter point -resize 300% -bordercolor black -border 3 +append \
    "$images/extrait-boites.png"
}

lignes=()
ligne() {
  lignes+=("$1" "$2" "${3:-}")
}

terminal() {
  local out=$1 cw=14 lh=30 top=82 left=24 y=82 width=1198 args=() i style text rest x indent color needed
  local count=$((${#lignes[@]} / 3))
  for ((i = 0; i < ${#lignes[@]}; i += 3)); do
    needed=$((2 * left + (${#lignes[i + 1]} + ${#lignes[i + 2]}) * cw))
    if ((needed > width)); then
      width=$needed
    fi
  done
  for ((i = 0; i < ${#lignes[@]}; i += 3)); do
    style=${lignes[i]} text=${lignes[i + 1]} rest=${lignes[i + 2]}
    indent=${text%%[! ]*}
    x=$((left + ${#indent} * cw))
    text=${text#"$indent"}
    case $style in
      n) color='#1d1d1f' ;;
      t) color='#0a3cc2' ;;
      c) color='#007f8c' ;;
      r) color='#c41a16' ;;
    esac
    if [ "$style" = n ]; then
      args+=(-fill "$color" -stroke none -annotate "+$x+$y" "${text//%/%%}")
    else
      args+=(-fill "$color" -stroke "$color" -strokewidth 0.7 -annotate "+$x+$y" "${text//%/%%}")
    fi
    if [ -n "$rest" ]; then
      args+=(-fill '#1d1d1f' -stroke none -annotate "+$((x + ${#text} * cw))+$y" "${rest//%/%%}")
    fi
    y=$((y + lh))
  done
  magick -size "${width}x$((top + count * lh - 4))" xc:white \
    -fill '#e8e8e8' -draw "rectangle 0,0 $width,44" -fill '#c8c8c8' -draw "line 0,44 $width,44" \
    -fill '#ff5f57' -draw 'circle 26,22 33,22' -fill '#febc2e' -draw 'circle 50,22 57,22' \
    -fill '#28c840' -draw 'circle 74,22 81,22' \
    -font /System/Library/Fonts/Menlo.ttc -pointsize 24 \
    -fill '#4d4d4d' -annotate "+$((width / 2 - 56))+30" 'Terminal' \
    "${args[@]}" -resize '1198x>' -bordercolor '#b0b0b0' -border 1 -quality 80 "$out"
  lignes=()
}

terminal_images() {
  ligne n '~ % batdetect'
  ligne n 'Détecte et suit les chauves-souris dans des vidéos de jumelles thermiques'
  ligne n ''
  ligne t 'Usages courants :'
  ligne n ''
  ligne n '  - Traiter tout un dossier de soirée, avec les résultats dans nuit-12-oct/batdetect/ :'
  ligne c '      batdetect nuit-12-oct'
  ligne n ''
  ligne n "  - Traiter plusieurs soirées d'un coup :"
  ligne c '      batdetect nuit-12-oct nuit-13-oct'
  ligne n ''
  ligne n '  - Traiter une seule vidéo :'
  ligne c '      batdetect nuit-12-oct/video_092.mp4'
  ligne n ''
  ligne n '  - Écrire les résultats dans un autre dossier :'
  ligne c '      batdetect nuit-12-oct -o resultats'
  ligne n ''
  ligne n '  - Faire un extrait ralenti et agrandi de chaque piste :'
  ligne c '      batdetect nuit-12-oct --zoom all'
  ligne n ''
  ligne n '  - Écrire aussi la vidéo entière, avec toutes les pistes dessinées :'
  ligne c '      batdetect nuit-12-oct --annotated'
  ligne n ''
  ligne n 'Toutes les options et les fichiers produits : batdetect --help'
  ligne n '~ % '
  terminal "$images/premiere-soiree-04-lancer.jpg"
  ligne n '~ % batdetect /Users/naturaliste/Desktop/nuit-12-oct'
  ligne n '[1/1] video_092_original_3m24-4m05 — 4 pistes'
  ligne n 'Bilan : 1 vidéo, 1 réussie'
  ligne n ''
  ligne n '/Users/naturaliste/Desktop/nuit-12-oct/batdetect'
  ligne n '  vidéo                          durée   pistes   non analysé   calcul'
  ligne n '  video_092_original_3m24-4m05    40 s        4           0 s      4 s'
  ligne n '~ % '
  terminal "$images/premiere-soiree-05-attendre.jpg"
}

results_page() {
  LANGUAGE=en cargo run --quiet --release -p batdetect -- "$extract" -o "$work/page" --no-notify
  "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" --headless=new --hide-scrollbars --blink-settings=preferredColorScheme=0 \
    --window-size=1440,1300 --screenshot="$work/page.png" "file://$repository/$work/page/results.html"
  magick "$work/page.png" -resize 1200x -quality 80 "$images/results-page.jpg"
}

summary_images
clip_images
terminal_images
results_page
