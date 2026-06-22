@echo off
setlocal

echo ================================================================
echo  robost Build Environment Setup
echo  Installs: Rust (rustup) + Visual Studio Build Tools (MSVC)
echo ================================================================
echo.

:: ---- [1/2] Check Rust ----
echo [1/2] Checking Rust installation...
where rustup >nul 2>&1
if %errorlevel% == 0 (
    echo       -> Rust is already installed.
    rustup show active-toolchain 2>nul
) else (
    echo       -> Rust not found. Installing via winget...
    winget install Rustlang.Rustup --silent --accept-package-agreements --accept-source-agreements
    if %errorlevel% neq 0 (
        echo [ERROR] winget install failed. Install Rust manually from:
        echo   https://rustup.rs/
        pause
        exit /b 1
    )
    echo.
    echo Rust installed. Please close this window, open a new terminal,
    echo and run this script again to continue with MSVC setup.
    pause
    exit /b 0
)

:: ---- [2/2] Check MSVC C++ compiler ----
echo.
echo [2/2] Checking MSVC Build Tools...

:: Try to find cl.exe via vswhere
set "VSWHERE=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe"
set "CL_FOUND=0"
if exist "%VSWHERE%" (
    for /f "usebackq tokens=*" %%i in (`"%VSWHERE%" -latest -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2^>nul`) do (
        if exist "%%i\VC\Tools\MSVC" set "CL_FOUND=1"
    )
)

if "%CL_FOUND%"=="1" (
    echo       -> Visual Studio Build Tools with C++ are already installed.
) else (
    echo       -> MSVC Build Tools not found. Installing...
    echo          This may take 10-20 minutes depending on your connection.
    echo.
    winget install Microsoft.VisualStudio.2022.BuildTools ^
        --override "--add Microsoft.VisualStudio.Workload.VCTools --add Microsoft.VisualStudio.Component.Windows11SDK.22621 --includeRecommended --passive" ^
        --accept-package-agreements --accept-source-agreements
    if %errorlevel% neq 0 (
        echo.
        echo [ERROR] winget install failed. Install manually:
        echo   1. Download: https://aka.ms/vs/17/release/vs_buildtools.exe
        echo   2. Select workload: "Desktop development with C++"
        echo   3. Ensure "Windows 11 SDK" component is checked
        pause
        exit /b 1
    )
)

echo.
echo ================================================================
echo  Setup complete!
echo.
echo  Please open a NEW terminal window, then build with:
echo    cargo build --release -p robost-cli
echo.
echo  If you see linker errors, run from:
echo    Start Menu -> "x64 Native Tools Command Prompt for VS 2022"
echo ================================================================
echo.
pause
