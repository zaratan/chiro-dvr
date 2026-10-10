#!/bin/sh
here=$(cd "$(dirname "$0")" && pwd)
DYLD_LIBRARY_PATH="$here/../.venv/lib/python3.14/site-packages/cv2/.dylibs" exec "$here/target/ff711/release/detect" "$@"
