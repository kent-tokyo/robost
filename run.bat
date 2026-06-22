@echo off
setlocal

set "PORT=9921"
set "URL=http://localhost:%PORT%"
set "SCRIPT_DIR=%~dp0"

:: If agent is already running, just open browser
netstat -an | findstr /C:":%PORT% " | findstr /C:"LISTENING" >nul 2>&1
if %errorlevel% == 0 (
    echo robost agent is already running ^(%URL%^)
    start "" "%URL%"
    exit /b 0
)

:: Look for rpa.exe
if exist "%SCRIPT_DIR%rpa.exe" (
    set "RPA=%SCRIPT_DIR%rpa.exe"
    goto :run
)
if exist "%SCRIPT_DIR%target\release\rpa.exe" (
    set "RPA=%SCRIPT_DIR%target\release\rpa.exe"
    goto :run
)
if exist "%SCRIPT_DIR%target\debug\rpa.exe" (
    set "RPA=%SCRIPT_DIR%target\debug\rpa.exe"
    goto :run
)
where rpa.exe >nul 2>&1
if %errorlevel% == 0 (
    set "RPA=rpa.exe"
    goto :run
)

echo ERROR: rpa.exe not found.
echo Please build first with: cargo build --release -p robost-cli
pause
exit /b 1

:run
echo Starting robost agent... ^(%URL%^)
echo Press Ctrl+C to stop
echo.
"%RPA%" agent --port %PORT%
