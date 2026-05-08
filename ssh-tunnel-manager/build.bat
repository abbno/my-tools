@echo off
echo ========================================
echo   SSH Tunnel Manager - Build Script
echo ========================================
echo.

echo [1/4] Installing dependencies...
call pnpm install
if errorlevel 1 (
    echo.
    echo [ERROR] Install failed!
    pause
    exit /b 1
)

echo.
echo [2/4] Building...
call pnpm tauri build
if errorlevel 1 (
    echo.
    echo [ERROR] Build failed!
    pause
    exit /b 1
)

echo.
echo [3/4] Build complete, creating portable ZIP...
cd src-tauri\target\release
powershell -ExecutionPolicy Bypass -Command "Compress-Archive -Path ssh-tunnel-manager.exe -DestinationPath SSH-Tunnel-Manager-Portable.zip -Force"

echo.
echo [4/4] Output files:
echo.
echo   Portable ZIP:
echo   %cd%\SSH-Tunnel-Manager-Portable.zip
echo.
echo   MSI Installer:
echo   %cd%\bundle\msi\SSH Tunnel Manager_0.1.0_x64_en-US.msi
echo.
echo   NSIS Installer:
echo   %cd%\bundle\nsis\SSH Tunnel Manager_0.1.0_x64-setup.exe
echo.
echo ========================================
echo   Build complete!
echo ========================================
pause