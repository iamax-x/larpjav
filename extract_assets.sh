#!/bin/bash
# extract_assets.sh - Extract assets from MojoLauncher repo

set -e

MOJOLAUNCHER_DIR="/tmp/MojoLauncher"
LARPJAV_DIR="$(pwd)/larp-jv-android/assets"

echo "=== Cloning MojoLauncher (v3_openjdk) ==="
if [ ! -d "$MOJOLAUNCHER_DIR" ]; then
    git clone --branch v3_openjdk --depth 1 https://github.com/MojoLauncher/MojoLauncher.git "$MOJOLAUNCHER_DIR"
else
    echo "Already cloned, pulling latest..."
    cd "$MOJOLAUNCHER_DIR" && git pull
    cd -
fi

echo "=== Creating asset directories ==="
mkdir -p "$LARPJAV_DIR"/{fonts,icons,shaders,config,components/{authlib-injector,caciocavallo,security},drawables,locales}

echo "=== Copying fonts ==="
cp -v "$MOJOLAUNCHER_DIR"/app_mojo/src/main/res/font/*.ttf "$LARPJAV_DIR/fonts/"

echo "=== Copying config files ==="
cp -v "$MOJOLAUNCHER_DIR"/app_mojo/src/main/assets/*.json "$LARPJAV_DIR/config/"
cp -v "$MOJOLAUNCHER_DIR"/app_mojo/src/main/assets/*.txt "$LARPJAV_DIR/config/"
cp -v "$MOJOLAUNCHER_DIR"/app_mojo/src/main/assets/*.pem "$LARPJAV_DIR/config/"
cp -v "$MOJOLAUNCHER_DIR"/app_mojo/src/main/assets/resolv.conf "$LARPJAV_DIR/config/"

echo "=== Copying icons/drawables ==="
cp -rv "$MOJOLAUNCHER_DIR"/app_mojo/src/main/res/drawable* "$LARPJAV_DIR/drawables/" 2>/dev/null || true
cp -v "$MOJOLAUNCHER_DIR"/app_mojo/src/main/assets/mojo.png "$LARPJAV_DIR/icons/"

echo "=== Copying components ==="
cp -rv "$MOJOLAUNCHER_DIR"/app_mojo/src/main/assets/components/* "$LARPJAV_DIR/components/" 2>/dev/null || true

echo "=== Copying strings (locales) ==="
mkdir -p "$LARPJAV_DIR/locales"
for f in "$MOJOLAUNCHER_DIR"/app_mojo/src/main/res/values*/strings.xml; do
    locale=$(basename $(dirname "$f"))
    locale=${locale#values-}
    [ "$locale" = "values" ] && locale="en"
    cp -v "$f" "$LARPJAV_DIR/locales/strings_${locale}.xml"
done

echo "=== Extracting shaders from AARs ==="
mkdir -p "$LARPJAV_DIR/shaders"
cd "$LARPJAV_DIR/shaders"
for aar in "$MOJOLAUNCHER_DIR"/app_mojo/libs/*.aar; do
    echo "Extracting $aar..."
    unzip -o "$aar" -d /tmp/aar_extract 2>/dev/null
    find /tmp/aar_extract -name "*.glsl" -o -name "*.vert" -o -name "*.frag" -o -name "*.spv" | while read f; do
        cp -v "$f" .
    done
    find /tmp/aar_extract -name "*.so" | while read f; do
        cp -v "$f" .
    done
done
cd -

echo "=== Generating Fluent localization files ==="
cd "$LARPJAV_DIR/locales"
for xml in strings_*.xml; do
    locale=$(basename "$xml" .xml | sed 's/strings_//')
    cat > "strings_${locale}.ftl" <<EOF
# Fluent localization for $locale
# Generated from strings_${locale}.xml

app-name = "larp.jv"
play-button = "Play"
instances-button = "Instances"
mods-button = "Mods"
settings-button = "Settings"
# ... (run conversion script for full conversion)
EOF
done
cd -

echo "=== Done! Assets extracted to $LARPJAV_DIR ==="
ls -la "$LARPJAV_DIR"