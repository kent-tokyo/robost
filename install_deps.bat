@echo off
setlocal enabledelayedexpansion
chcp 65001 > nul

echo ================================================================
echo  robost 依存ライブラリ インストーラー
echo  (Tesseract OCR + Visual C++ 再配布パッケージ)
echo ================================================================
echo.

:: 管理者権限チェック
net session >nul 2>&1
if %errorlevel% neq 0 (
    echo [エラー] 管理者として実行してください。
    echo このファイルを右クリック → "管理者として実行" を選んでください。
    pause
    exit /b 1
)

:: ---- 1. Visual C++ 再配布パッケージ (2015-2022) ----
echo [1/2] Visual C++ 再配布パッケージを確認中...
reg query "HKLM\SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64" >nul 2>&1
if %errorlevel% == 0 (
    echo       → すでにインストール済みです。
) else (
    echo       → ダウンロード中...
    curl -L -o "%TEMP%\vc_redist.x64.exe" ^
        "https://aka.ms/vs/17/release/vc_redist.x64.exe"
    if %errorlevel% neq 0 (
        echo [エラー] ダウンロードに失敗しました。インターネット接続を確認してください。
        pause
        exit /b 1
    )
    echo       → インストール中 (サイレント)...
    "%TEMP%\vc_redist.x64.exe" /install /quiet /norestart
    echo       → 完了
)

:: ---- 2. Tesseract OCR ----
echo.
echo [2/2] Tesseract OCR を確認中...

:: インストール済みか確認 (レジストリ)
set "TESS_EXE="
for %%D in (
    "C:\Program Files\Tesseract-OCR\tesseract.exe"
    "C:\Program Files (x86)\Tesseract-OCR\tesseract.exe"
) do (
    if exist %%D set "TESS_EXE=%%~D"
)

if defined TESS_EXE (
    echo       → すでにインストール済みです: %TESS_EXE%
) else (
    echo       → ダウンロード中 (Tesseract 5.x + 日本語モデル)...
    :: UB-Mannheim の公式インストーラー (日本語含む全言語パック)
    set "TESS_INSTALLER=%TEMP%\tesseract-ocr-setup.exe"
    curl -L -o "!TESS_INSTALLER!" ^
        "https://github.com/UB-Mannheim/tesseract/releases/download/v5.5.0.20241111/tesseract-ocr-w64-setup-5.5.0.20241111.exe"
    if %errorlevel% neq 0 (
        echo [エラー] Tesseract のダウンロードに失敗しました。
        echo 手動でインストールしてください:
        echo   https://github.com/UB-Mannheim/tesseract/wiki
        pause
        exit /b 1
    )
    echo       → インストール中 (日本語・英語モデル選択あり)...
    echo         ※ インストール画面で "Japanese" にチェックを入れてください
    "!TESS_INSTALLER!" /S /D="C:\Program Files\Tesseract-OCR"
    echo       → 完了
)

:: Tesseract を PATH に追加 (ユーザー環境変数)
echo.
echo PATH に Tesseract を追加中...
for %%D in (
    "C:\Program Files\Tesseract-OCR"
    "C:\Program Files (x86)\Tesseract-OCR"
) do (
    if exist "%%~D\tesseract.exe" (
        setx PATH "%%~D;%PATH%" >nul 2>&1
        echo       → %%~D を PATH に追加しました。
    )
)

echo.
echo ================================================================
echo  インストール完了！
echo  robost を使うには run.bat をダブルクリックしてください。
echo  (新しいコマンドプロンプトを開いてから有効になります)
echo ================================================================
echo.
pause
