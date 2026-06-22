@echo off
setlocal enabledelayedexpansion
chcp 65001 > nul

echo ================================================================
echo  robost Dependency Installer
echo  (Tesseract OCR + Visual C++ Redistributable)
echo ================================================================
echo.

:: Check for administrator privileges
net session >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERROR] Please run as administrator.
    echo Right-click this file and select "Run as administrator".
    pause
    exit /b 1
)

:: ---- 1. Visual C++ Redistributable (2015-2022) ----
echo [1/2] Checking Visual C++ Redistributable...
reg query "HKLM\SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64" >nul 2>&1
if %errorlevel% == 0 (
    echo       -> Already installed.
) else (
    echo       -> Downloading...
    curl -L -o "%TEMP%\vc_redist.x64.exe" ^
        "https://aka.ms/vs/17/release/vc_redist.x64.exe"
    if %errorlevel% neq 0 (
        echo [ERROR] Download failed. Please check your internet connection.
        pause
        exit /b 1
    )
    echo       -> Installing (silent)...
    "%TEMP%\vc_redist.x64.exe" /install /quiet /norestart
    echo       -> Done
)

:: ---- 2. Tesseract OCR ----
echo.
echo [2/2] Checking Tesseract OCR...

:: Check if already installed
set "TESS_EXE="
for %%D in (
    "C:\Program Files\Tesseract-OCR\tesseract.exe"
    "C:\Program Files (x86)\Tesseract-OCR\tesseract.exe"
) do (
    if exist %%D set "TESS_EXE=%%~D"
)

if defined TESS_EXE (
    echo       -> Already installed: %TESS_EXE%
) else (
    echo       -> Downloading Tesseract 5.x with Japanese language model...
    set "TESS_INSTALLER=%TEMP%\tesseract-ocr-setup.exe"
    curl -L -o "!TESS_INSTALLER!" ^
        "https://github.com/UB-Mannheim/tesseract/releases/download/v5.5.0.20241111/tesseract-ocr-w64-setup-5.5.0.20241111.exe"
    if %errorlevel% neq 0 (
        echo [ERROR] Tesseract download failed.
        echo Please install manually from:
        echo   https://github.com/UB-Mannheim/tesseract/wiki
        pause
        exit /b 1
    )
    echo       -> Installing (check "Japanese" language during setup if prompted)...
    "!TESS_INSTALLER!" /S /D="C:\Program Files\Tesseract-OCR"
    echo       -> Done
)

:: Add Tesseract to PATH (user environment variable)
echo.
echo Adding Tesseract to PATH...
for %%D in (
    "C:\Program Files\Tesseract-OCR"
    "C:\Program Files (x86)\Tesseract-OCR"
) do (
    if exist "%%~D\tesseract.exe" (
        setx PATH "%%~D;%PATH%" >nul 2>&1
        echo       -> Added %%~D to PATH.
    )
)

echo.
echo ================================================================
echo  Installation complete!
echo  Double-click run.bat to start robost.
echo  (Open a new command prompt for PATH changes to take effect)
echo ================================================================
echo.
pause
