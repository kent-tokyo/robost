#define AppName "robost"
#define AppPublisher "kent-tokyo"
#define AppURL "https://github.com/kent-tokyo/robost"
#define AppExeName "rpa.exe"

[Setup]
AppId={{B8F1E2A3-4C5D-6E7F-8A9B-0C1D2E3F4A5B}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher={#AppPublisher}
AppPublisherURL={#AppURL}
AppSupportURL={#AppURL}
AppUpdatesURL={#AppURL}/releases
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
AllowNoIcons=yes
; Anchor source paths to the repo root (parent of installer/)
SourceDir={#SourcePath}\..
OutputDir={#SourcePath}\output
OutputBaseFilename=robost-setup
SetupIconFile=assets\icon.ico
Compression=lzma
SolidCompression=yes
WizardStyle=modern
ChangesEnvironment=yes
PrivilegesRequired=admin

[Languages]
Name: "japanese"; MessagesFile: "compiler:Languages\Japanese.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"
Name: "modifypath"; Description: "PATH 環境変数に robost を追加"; GroupDescription: "環境設定:"

[Files]
Source: "{#RpaExe}"; DestDir: "{app}"; Flags: ignoreversion
Source: "run.bat"; DestDir: "{app}"; Flags: ignoreversion
Source: "stop.bat"; DestDir: "{app}"; Flags: ignoreversion
Source: "installer\launch-agent.ps1"; DestDir: "{app}"; Flags: ignoreversion
Source: "assets\icon.ico"; DestDir: "{app}"; Flags: ignoreversion
Source: "examples\*"; DestDir: "{app}\examples"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
; Use PowerShell to launch without showing a console window.
; Get-NetTCPConnection guard prevents duplicate processes on double-click.
Name: "{group}\robost エディタを起動"; Filename: "{sys}\WindowsPowerShell\v1.0\powershell.exe"; Parameters: "-ExecutionPolicy Bypass -WindowStyle Hidden -File ""{app}\launch-agent.ps1"""; WorkingDir: "{app}"; IconFilename: "{app}\icon.ico"
Name: "{group}\{cm:UninstallProgram,{#AppName}}"; Filename: "{uninstallexe}"
Name: "{commondesktop}\robost エディタを起動"; Filename: "{sys}\WindowsPowerShell\v1.0\powershell.exe"; Parameters: "-ExecutionPolicy Bypass -WindowStyle Hidden -File ""{app}\launch-agent.ps1"""; WorkingDir: "{app}"; IconFilename: "{app}\icon.ico"; Tasks: desktopicon

[Registry]
Root: HKLM; Subkey: "SYSTEM\CurrentControlSet\Control\Session Manager\Environment"; ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{app}"; Check: NeedsAddPath(ExpandConstant('{app}')); Tasks: modifypath

[Code]
// Case-insensitive check: returns True if AppDir is NOT already in PATH.
function NeedsAddPath(AppDir: string): boolean;
var
  OrigPath: string;
begin
  if not RegQueryStringValue(HKEY_LOCAL_MACHINE,
    'SYSTEM\CurrentControlSet\Control\Session Manager\Environment',
    'Path', OrigPath)
  then begin
    Result := True;
    exit;
  end;
  Result := Pos(LowerCase(';' + AppDir + ';'),
                LowerCase(';' + OrigPath + ';')) = 0;
end;

// Remove AppDir from the system PATH during uninstall (case-insensitive).
procedure RemoveFromPath(AppDir: string);
var
  OrigPath, NewPath, AppDirLower, Segment: string;
  P, Q: Integer;
begin
  if not RegQueryStringValue(HKEY_LOCAL_MACHINE,
    'SYSTEM\CurrentControlSet\Control\Session Manager\Environment',
    'Path', OrigPath)
  then exit;
  AppDirLower := LowerCase(AppDir);
  NewPath := '';
  P := 1;
  while P <= Length(OrigPath) do begin
    Q := P;
    while (Q <= Length(OrigPath)) and (OrigPath[Q] <> ';') do
      Q := Q + 1;
    Segment := Copy(OrigPath, P, Q - P);
    if LowerCase(Segment) <> AppDirLower then begin
      if NewPath <> '' then NewPath := NewPath + ';';
      NewPath := NewPath + Segment;
    end;
    P := Q + 1;
  end;
  RegWriteExpandStringValue(HKEY_LOCAL_MACHINE,
    'SYSTEM\CurrentControlSet\Control\Session Manager\Environment',
    'Path', NewPath);
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then
    RemoveFromPath(ExpandConstant('{app}'));
end;
