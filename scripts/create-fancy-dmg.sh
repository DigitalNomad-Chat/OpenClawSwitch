#!/usr/bin/env bash
# ============================================================================
# Clawlite Fancy DMG Builder
#
# Creates a polished macOS DMG with custom background, icon positioning,
# and Applications shortcut for drag-to-install UX.
# ============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
TAURI_DIR="$PROJECT_DIR/src-tauri"

# --- Configuration ---
VOLUME_NAME="Clawlite"
WINDOW_W=540
WINDOW_H=380
ICON_SIZE=128
TEXT_SIZE=13
APP_X=140
APP_Y=150
APPS_X=400
APPS_Y=150

# --- Resolve paths ---
APP_SOURCE="$TAURI_DIR/target/release/bundle/macos/Clawlite.app"
BACKGROUND="$TAURI_DIR/resources/dmg-background.png"
OUTPUT_DIR="$TAURI_DIR/target/release/bundle/dmg"
DMG_NAME="Clawlite_$(grep '"version"' "$TAURI_DIR/tauri.conf.json" | head -1 | sed 's/.*: *"//' | sed 's/".*//')_aarch64.dmg"
DMG_PATH="$OUTPUT_DIR/$DMG_NAME"

# --- Validate ---
if [[ ! -d "$APP_SOURCE" ]]; then
  echo "Error: $APP_SOURCE not found. Run 'npx tauri build' first."
  exit 1
fi

if [[ ! -f "$BACKGROUND" ]]; then
  echo "Error: $BACKGROUND not found."
  exit 1
fi

mkdir -p "$OUTPUT_DIR"

# --- Prepare staging ---
STAGING=$(mktemp -d)
trap "rm -rf '$STAGING'" EXIT

cp -R "$APP_SOURCE" "$STAGING/Clawlite.app"
mkdir "$STAGING/Applications"

# --- Create read-write DMG ---
RW_DMG="$OUTPUT_DIR/rw.$DMG_NAME"
rm -f "$RW_DMG"

echo "Creating disk image..."
hdiutil create -volname "$VOLUME_NAME" \
  -srcfolder "$STAGING" \
  -ov -format UDRW \
  "$RW_DMG" 2>&1 | tail -1

# --- Mount ---
echo "Mounting..."
MOUNT_POINT="/Volumes/$VOLUME_NAME"

# Unmount if already mounted
if [[ -d "$MOUNT_POINT" ]]; then
  hdiutil detach "$MOUNT_POINT" -quiet 2>/dev/null || true
  sleep 1
fi

DEV_NAME=$(hdiutil attach -readwrite -noverify -noautoopen "$RW_DMG" 2>&1 \
  | grep '^/dev/' | sed 1q | awk '{print $1}')

if [[ -z "$DEV_NAME" ]]; then
  echo "Error: Failed to mount DMG."
  exit 1
fi

# --- Copy background ---
mkdir -p "$MOUNT_POINT/.background"
cp "$BACKGROUND" "$MOUNT_POINT/.background/background.png"

# --- Configure Finder appearance via AppleScript ---
echo "Configuring Finder appearance..."
osascript <<EOF
tell application "Finder"
  tell disk "$VOLUME_NAME"
    open
    set current view of container window to icon view
    set toolbar visible of container window to false
    set statusbar visible of container window to false
    set the bounds of container window to {100, 100, 100 + $WINDOW_W, 100 + $WINDOW_H}

    set viewOptions to the icon view options of container window
    set icon size of viewOptions to $ICON_SIZE
    set text size of viewOptions to $TEXT_SIZE
    set arrangement of viewOptions to not arranged
    set background picture of viewOptions to file ".background:background.png"

    set position of item "Clawlite.app" to {$APP_X, $APP_Y}
    set position of item "Applications" to {$APPS_X, $APPS_Y}

    close
    open
    delay 1
  end tell
end tell
EOF

# --- Unmount ---
echo "Unmounting..."
hdiutil detach "$DEV_NAME" -quiet 2>/dev/null || {
  sleep 2
  hdiutil detach "$DEV_NAME" -force -quiet 2>/dev/null
}

# --- Convert to compressed read-only ---
echo "Compressing..."
rm -f "$DMG_PATH"
hdiutil convert "$RW_DMG" -format UDZO -imagekey zlib-level=9 -ov -o "$DMG_PATH" 2>&1 | tail -1
rm -f "$RW_DMG"

echo ""
echo "=========================================="
echo "  DMG created successfully"
echo "  Path: $DMG_PATH"
echo "  Size: $(du -h "$DMG_PATH" | cut -f1)"
echo "=========================================="
