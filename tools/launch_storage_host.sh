#!/bin/bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Start an isolated Logos Storage module host with a caller-controlled token.
# Useful when the Basecamp-managed storage instance's capability token is not
# available to a headless client.
set -euo pipefail

APPIMAGE_MOUNT="${APPIMAGE_MOUNT:-/tmp/.mount_LogosBobefME/usr}"
INSTANCE_DIR="${INSTANCE_DIR:-$HOME/proclaim-storage-instance}"
TOKEN_FILE="${TOKEN_FILE:-/tmp/proclaim_storage_token}"
TOKEN="${TOKEN:-proclaim-registry-upload-token-2026}"

mkdir -p "$INSTANCE_DIR"
printf '%s' "$TOKEN" > "$TOKEN_FILE"
chmod 600 "$TOKEN_FILE"

export QT_QPA_PLATFORM=offscreen
export LD_LIBRARY_PATH="$APPIMAGE_MOUNT/lib:${LD_LIBRARY_PATH:-}"
export QT_PLUGIN_PATH="$APPIMAGE_MOUNT/plugins"
export QML2_IMPORT_PATH="$APPIMAGE_MOUNT/qml"

exec "$APPIMAGE_MOUNT/bin/logos_host" \
  --name storage_module \
  --path "$APPIMAGE_MOUNT/modules/storage_module/storage_module_plugin.so" \
  --instance-persistence-path "$INSTANCE_DIR" \
  --token-source "file:$TOKEN_FILE"
