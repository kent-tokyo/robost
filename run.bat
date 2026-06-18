@echo off
setlocal

set "PORT=9921"
set "URL=http://localhost:%PORT%"
set "SCRIPT_DIR=%~dp0"

:: すでにエージェントが起動中ならブラウザを開くだけ
netstat -an | findstr /C:":%PORT% " | findstr /C:"LISTENING" >nul 2>&1
if %errorlevel% == 0 (
    echo robost agent はすでに起動しています ^(%URL%^)
    start "" "%URL%"
    exit /b 0
)

:: rpa.exe を探す
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

echo ERROR: rpa.exe が見つかりません。
echo 先に "cargo build --release -p robost-cli" でビルドしてください。
pause
exit /b 1

:run
echo robost agent を起動しています... ^(%URL%^)
echo 終了するには Ctrl+C を押してください
echo.
"%RPA%" agent --port %PORT%
