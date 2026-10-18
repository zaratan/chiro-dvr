# Exploration WebAssembly (#347)

Prototype de la version web d'essai (méta #346), gardé hors de `main` : rien ici n'est livré ni testé par la CI. Le compte rendu complet est dans `RAPPORT.md`, le découpage en issues dans `issues/` (créées en #349 à #360).

- `ws.patch` et `ws-render.patch` : les changements du dépôt faits pour la compilation en wasm, contre le commit 9172af1 (`git apply -p1` sur une copie de cet arbre). `ws.patch` sépare `media` et `engine` derrière une feature `native` et ajoute la crate `webproto` ; `ws-render.patch` y ajoute le rendu (`render`) sans FFmpeg.
- `scripts/build.sh` : la construction des trois variantes (simple, `simd128`, fils par `atomics` sous nightly) et le passage par `wasm-bindgen` 0.2.129 ; les chemins y sont ceux du poste d'origine (`out/347/ws`).
- `web/` : les pages d'essai (décodage par WebCodecs et Mediabunny, encodage, zip) et le banc lancé par `node web/run.mjs` ; `pnpm install` dans `web/` avant.
- `zipweb/` et `zipprobe/` : le zip en mode Stored dans le navigateur, et les sondes de taille du wasm.
- `mac/` : le banc à jouer sur le Mac (QuickTime, VLC, Utilitaire d'archive, Safari, Chrome, Firefox), voir `mac/README.md` ; l'extrait `extract.mp4` n'est pas gardé : c'est `tests/fixtures/video_092_original_3m24-4m05.mp4`.
- `vercel.json` : les en-têtes COOP et COEP qu'il faut pour les fils.
