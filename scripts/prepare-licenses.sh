#!/bin/bash

# License Preparation Script for Commercial Distribution
# This script prepares all necessary license and attribution files for distribution

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RESOURCES_DIR="$PROJECT_ROOT/src-tauri/resources"

echo "📦 Preparing license files for distribution..."
echo "Project root: $PROJECT_ROOT"
echo "Resources dir: $RESOURCES_DIR"
echo ""

# Create resources directory if it doesn't exist
mkdir -p "$RESOURCES_DIR"

# Copy main LICENSE to resources
echo "📄 Copying LICENSE..."
cp "$PROJECT_ROOT/LICENSE" "$RESOURCES_DIR/LICENSE" 2>/dev/null || \
cp "$PROJECT_ROOT/LICENSE.COMBINED" "$RESOURCES_DIR/LICENSE"

# Copy ATTRIBUTION to resources
echo "📄 Copying ATTRIBUTION.md..."
cp "$PROJECT_ROOT/ATTRIBUTION.md" "$RESOURCES_DIR/ATTRIBUTION.md"

# Create a condensed third-party notice for distribution
echo "📄 Creating THIRD-PARTY-NOTICES.md..."
cat > "$RESOURCES_DIR/THIRD-PARTY-NOTICES.md" << 'EOF'
# Third-Party Software Notices

This software incorporates the following third-party software components:

## Vue 3 (MIT License)
Copyright (c) 2013-present, Yuxi (Evan) You
https://github.com/vuejs/core

## Tauri (MIT/Apache-2.0)
Copyright (c) 2019-2024 Tauri Programme
https://github.com/tauri-apps/tauri

## Tailwind CSS (MIT License)
Copyright (c) Tailwind Labs, Inc.
https://github.com/tailwindlabs/tailwindcss

## TypeScript (Apache-2.0)
Copyright (c) Microsoft Corporation
https://github.com/microsoft/TypeScript

## Vite (MIT License)
Copyright (c) 2019-present, Yuxi (Evan) You
https://github.com/vitejs/vite

## Lucide Icons (ISC License)
Copyright (c) 2020-2024, Lucide Contributors
https://github.com/lucide-icons/lucide

All third-party libraries are used under their respective open-source licenses.

For complete license texts and attribution, visit the project repository.
EOF

echo ""
echo "✅ License files prepared successfully!"
echo ""
echo "Files created in $RESOURCES_DIR:"
ls -la "$RESOURCES_DIR"/*.md "$RESOURCES_DIR"/LICENSE 2>/dev/null | awk '{print "  - " $NF}'
echo ""
echo "🚀 You can now build the application:"
echo "   npm run tauri:build"
echo ""
echo "⚠️  Don't forget to update the placeholder values:"
echo "   - [Your Name / Your Company]"
echo "   - [Your Repository URL]"
echo "   - [Your Brand Name]"
