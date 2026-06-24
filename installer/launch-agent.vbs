' Launch rpa.exe agent without showing a console window.
' SW_HIDE (0) keeps the terminal invisible while the agent runs in the background.
Dim scriptDir
scriptDir = Left(WScript.ScriptFullName, InStrRev(WScript.ScriptFullName, "\"))
CreateObject("WScript.Shell").Run Chr(34) & scriptDir & "rpa.exe" & Chr(34) & " agent", 0, False
