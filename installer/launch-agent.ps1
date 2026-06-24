# Launch rpa.exe agent without showing a console window.
# If the agent is already running on port 9921, open the browser to the
# existing instance instead of spawning a duplicate process.
$port    = 9921
$appDir  = Split-Path -Parent $MyInvocation.MyCommand.Path
$rpaExe  = Join-Path $appDir 'rpa.exe'
$url     = "http://localhost:$port"

$isRunning = [bool](Get-NetTCPConnection -LocalPort $port -State Listen -ErrorAction SilentlyContinue)
if ($isRunning) {
    Start-Process $url
} else {
    Start-Process -FilePath $rpaExe -ArgumentList 'agent' -WindowStyle Hidden
    Start-Sleep -Milliseconds 800
    Start-Process $url
}
