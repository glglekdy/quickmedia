; QuickMedia 설치 파일. build.ps1 이 dist/QuickMedia (PyInstaller onedir) 를 만든 뒤 컴파일한다.
;   ISCC /DAppVersion=1.2.3 installer\QuickMedia.iss  ->  dist\QuickMedia-Setup-1.2.3.exe

#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif

[Setup]
; 업데이트할 때 같은 앱으로 알아보고 이전 설치 폴더를 그대로 쓴다. 바꾸지 말 것.
AppId={{8F3C2A71-5D4E-4B9A-9C61-2E7D0F4A3B85}
AppName=QuickMedia
AppVersion={#AppVersion}
AppPublisher=glglekdy
UninstallDisplayName=QuickMedia
DefaultDirName={autopf}\QuickMedia
DefaultGroupName=QuickMedia
; 관리자 권한 없이 %LOCALAPPDATA%\Programs 에 설치한다. 그래서 업데이트도 조용히 끝난다.
PrivilegesRequired=lowest
OutputDir=..\dist
OutputBaseFilename=QuickMedia-Setup-{#AppVersion}
SetupIconFile=..\assets\icon.ico
UninstallDisplayIcon={app}\QuickMedia.exe
WizardStyle=modern
Compression=lzma2
SolidCompression=yes

[Languages]
Name: "korean"; MessagesFile: "compiler:Languages\Korean.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
Source: "..\dist\QuickMedia\*"; DestDir: "{app}"; Flags: recursesubdirs ignoreversion

[Icons]
Name: "{group}\QuickMedia"; Filename: "{app}\QuickMedia.exe"
Name: "{autodesktop}\QuickMedia"; Filename: "{app}\QuickMedia.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\QuickMedia.exe"; Description: "QuickMedia 실행"; Flags: postinstall nowait skipifsilent
; 앱 안에서 업데이트(조용한 설치)한 경우 끝나면 바로 다시 띄운다.
Filename: "{app}\QuickMedia.exe"; Flags: nowait skipifnotsilent

[Code]
// 옛 '유튜브 다운로더'는 업데이트할 때 자기 파일(YoutubeDownloader.exe)을 이 설치 파일로 덮어쓰고 실행한다.
// 그 사본을 앱이 처음 켜질 때 지우도록 경로를 남긴다 (ytdl/migrate.py).
procedure CurStepChanged(CurStep: TSetupStep);
begin
  if (CurStep = ssPostInstall) and
     (CompareText(ExtractFileName(ExpandConstant('{srcexe}')), 'YoutubeDownloader.exe') = 0) then
    RegWriteStringValue(HKCU, 'Software\QuickMedia\QuickMedia', 'legacy_exe', ExpandConstant('{srcexe}'));
end;
