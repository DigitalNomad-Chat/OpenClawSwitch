@echo off
REM License Preparation Script for Commercial Distribution (Windows)
REM This script prepares all necessary license and attribution files for distribution

setlocal enabledelayedexpansion

REM Get project root directory
set "PROJECT_ROOT=%~dp0.."
set "RESOURCES_DIR=%PROJECT_ROOT%\src-tauri\resources"

echo 📦 Preparing license files for distribution...
echo Project root: %PROJECT_ROOT%
echo Resources dir: %RESOURCES_DIR%
echo.

REM Create resources directory if it doesn't exist
if not exist "%RESOURCES_DIR%" mkdir "%RESOURCES_DIR%"

REM Copy main LICENSE to resources
echo 📄 Copying LICENSE...
if exist "%PROJECT_ROOT%\LICENSE" (
    copy /Y "%PROJECT_ROOT%\LICENSE" "%RESOURCES_DIR%\LICENSE" >nul
) else if exist "%PROJECT_ROOT%\LICENSE.COMBINED" (
    copy /Y "%PROJECT_ROOT%\LICENSE.COMBINED" "%RESOURCES_DIR%\LICENSE" >nul
)

REM Copy ATTRIBUTION to resources
echo 📄 Copying ATTRIBUTION.md...
if exist "%PROJECT_ROOT%\ATTRIBUTION.md" (
    copy /Y "%PROJECT_ROOT%\ATTRIBUTION.md" "%RESOURCES_DIR%\ATTRIBUTION.md" >nul
)

REM Create a condensed third-party notice for distribution
echo 📄 Creating THIRD-PARTY-NOTICES.md...
(
echo # Third-Party Software Notices
echo.
echo This software incorporates the following third-party software components:
echo.
echo ## Vue 3 ^(MIT License^)
echo Copyright ^(c^) 2013-present, Yuxi ^(Evan^) You
echo https://github.com/vuejs/core
echo.
echo ## Tauri ^(MIT/Apache-2.0^)
echo Copyright ^(c^) 2019-2024 Tauri Programme
echo https://github.com/tauri-apps/tauri
echo.
echo ## Tailwind CSS ^(MIT License^)
echo Copyright ^(c^) Tailwind Labs, Inc.
echo https://github.com/tailwindlabs/tailwindcss
echo.
echo ## TypeScript ^(Apache-2.0^)
echo Copyright ^(c^) Microsoft Corporation
echo https://github.com/microsoft/TypeScript
echo.
echo ## Vite ^(MIT License^)
echo Copyright ^(c^) 2019-present, Yuxi ^(Evan^) You
echo https://github.com/vitejs/vite
echo.
echo ## Lucide Icons ^(ISC License^)
echo Copyright ^(c^) 2020-2024, Lucide Contributors
echo https://github.com/lucide-icons/lucide
echo.
echo All third-party libraries are used under their respective open-source licenses.
echo.
echo For complete license texts and attribution, visit the project repository.
) > "%RESOURCES_DIR%\THIRD-PARTY-NOTICES.md"

echo.
echo ✅ License files prepared successfully!
echo.
echo 🚀 You can now build the application:
echo    npm run tauri:build
echo.
echo ⚠️  Don't forget to update the placeholder values:
echo    - [Your Name / Your Company]
echo    - [Your Repository URL]
echo    - [Your Brand Name]
echo.

pause
