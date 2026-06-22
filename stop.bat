@echo off
set "PORT=9921"

for /f "tokens=5" %%a in ('netstat -aon ^| findstr /C:":%PORT% " ^| findstr /C:"LISTENING" 2^>nul') do (
  taskkill /PID %%a /F >nul 2>&1
  echo robost agent stopped ^(PID %%a^)
  exit /b 0
)

echo robost agent is not running
