; Inno Setup Script for LightRaw 1.0.0
; Builds LightRaw-1.0.0-Setup.exe for Windows 10 & 11 (64-bit)

#define MyAppName "LightRaw"
#define MyAppVersion "1.0.0"
#define MyAppPublisher "Moestret"
#define MyAppURL "https://github.com/moestret/LightRaw"
#define MyAppExeName "lightraw.exe"

[Setup]
AppId={{9C52E58A-5D1F-4B2E-8E0A-1B7A382C1F90}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
OutputDir=..\..\dist
OutputBaseFilename=LightRaw-{#MyAppVersion}-Windows-Setup
SetupIconFile=..\..\assets\icon.ico
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64compatible
ChangesAssociations=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "associate_dng"; Description: "Associate with Adobe Digital Negative (.dng)"; GroupDescription: "File Associations:"
Name: "associate_lrcat"; Description: "Associate with LightRaw Catalog (.lrcat)"; GroupDescription: "File Associations:"

[Files]
Source: "..\..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\assets\icon.ico"; DestDir: "{app}\assets"; Flags: ignoreversion
Source: "..\..\assets\icon.png"; DestDir: "{app}\assets"; Flags: ignoreversion
Source: "..\..\README.md"; DestDir: "{app}"; Flags: ignoreversion isreadme

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\assets\icon.ico"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\assets\icon.ico"; Tasks: desktopicon

[Registry]
; .lrcat File Association
Root: HKA; Subkey: "Software\Classes\.lrcat"; ValueType: string; ValueName: ""; ValueData: "LightRaw.Catalog"; Flags: uninsdeletevalue; Tasks: associate_lrcat
Root: HKA; Subkey: "Software\Classes\LightRaw.Catalog"; ValueType: string; ValueName: ""; ValueData: "LightRaw Catalog Database"; Flags: uninsdeletekey; Tasks: associate_lrcat
Root: HKA; Subkey: "Software\Classes\LightRaw.Catalog\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\assets\icon.ico,0"; Tasks: associate_lrcat
Root: HKA; Subkey: "Software\Classes\LightRaw.Catalog\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: associate_lrcat

; .dng File Association
Root: HKA; Subkey: "Software\Classes\.dng"; ValueType: string; ValueName: ""; ValueData: "LightRaw.DNG"; Flags: uninsdeletevalue; Tasks: associate_dng
Root: HKA; Subkey: "Software\Classes\LightRaw.DNG"; ValueType: string; ValueName: ""; ValueData: "Adobe Digital Negative RAW"; Flags: uninsdeletekey; Tasks: associate_dng
Root: HKA; Subkey: "Software\Classes\LightRaw.DNG\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\assets\icon.ico,0"; Tasks: associate_dng
Root: HKA; Subkey: "Software\Classes\LightRaw.DNG\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: associate_dng

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
