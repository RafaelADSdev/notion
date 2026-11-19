Unicode true
RequestExecutionLevel user

!include "MUI2.nsh"
!include "WinVer.nsh"
!include "x64.nsh"

!define APP_VERSION "0.1.0"
!define UNINSTALL_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\Margem"

Name "Margem"
OutFile "..\target\installer\Margem-Setup-${APP_VERSION}-x64.exe"
InstallDir "$LOCALAPPDATA\Programs\Margem"
InstallDirRegKey HKCU "${UNINSTALL_KEY}" "InstallLocation"
SetCompressor /SOLID lzma

VIProductVersion "${APP_VERSION}.0"
VIAddVersionKey /LANG=1046 "ProductName" "Margem"
VIAddVersionKey /LANG=1046 "FileDescription" "Instalador do Margem para Windows x64"
VIAddVersionKey /LANG=1046 "CompanyName" "Margem"
VIAddVersionKey /LANG=1046 "LegalCopyright" "Margem"
VIAddVersionKey /LANG=1046 "FileVersion" "${APP_VERSION}"
VIAddVersionKey /LANG=1046 "ProductVersion" "${APP_VERSION}"

!define MUI_ABORTWARNING
!define MUI_ICON "Margem.ico"
!define MUI_UNICON "Margem.ico"
!define MUI_WELCOMEPAGE_TITLE "Instalar o Margem"
!define MUI_WELCOMEPAGE_TEXT "O Margem é um espaço de trabalho local para suas páginas, notas e coleções.$\r$\n$\r$\nO instalador adiciona o Margem ao menu Iniciar do seu usuário."
!define MUI_FINISHPAGE_RUN "$INSTDIR\margem-desktop.exe"
!define MUI_FINISHPAGE_RUN_TEXT "Abrir o Margem"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "PortugueseBR"

Function .onInit
  ${IfNot} ${AtLeastWin10}
  ${OrIfNot} ${IsNativeAMD64}
    MessageBox MB_OK|MB_ICONSTOP "Este instalador requer Windows 10 ou 11 x64." /SD IDOK
    SetErrorLevel 1
    Abort
  ${EndIf}
FunctionEnd

Section "Margem (obrigatório)"
  SectionIn RO
  SetShellVarContext current
  ClearErrors
  SetOutPath "$INSTDIR"
  SetOverwrite try
  File "..\target\release\margem-desktop.exe"
  WriteUninstaller "$INSTDIR\Desinstalar.exe"
  ${If} ${Errors}
    MessageBox MB_OK|MB_ICONSTOP "Não foi possível instalar o Margem. Feche o aplicativo, confira a pasta de destino e tente novamente." /SD IDOK
    SetErrorLevel 2
    Abort
  ${EndIf}
  CreateShortCut "$SMPROGRAMS\Margem.lnk" "$INSTDIR\margem-desktop.exe" "" "$INSTDIR\Desinstalar.exe"

  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayName" "Margem"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayVersion" "${APP_VERSION}"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "Publisher" "Margem"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayIcon" "$INSTDIR\Desinstalar.exe"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "UninstallString" '$\"$INSTDIR\Desinstalar.exe$\"'
  WriteRegStr HKCU "${UNINSTALL_KEY}" "QuietUninstallString" '$\"$INSTDIR\Desinstalar.exe$\" /S'
  WriteRegDWORD HKCU "${UNINSTALL_KEY}" "NoModify" 1
  WriteRegDWORD HKCU "${UNINSTALL_KEY}" "NoRepair" 1
SectionEnd

Section /o "Atalho na área de trabalho"
  SetShellVarContext current
  CreateShortCut "$DESKTOP\Margem.lnk" "$INSTDIR\margem-desktop.exe" "" "$INSTDIR\Desinstalar.exe"
SectionEnd

Section "Uninstall"
  SetShellVarContext current
  ClearErrors
  Delete "$INSTDIR\margem-desktop.exe"
  ${If} ${Errors}
    MessageBox MB_OK|MB_ICONSTOP "Feche o Margem antes de desinstalar." /SD IDOK
    SetErrorLevel 1
    Abort
  ${EndIf}
  Delete "$SMPROGRAMS\Margem.lnk"
  Delete "$DESKTOP\Margem.lnk"
  Delete "$INSTDIR\Desinstalar.exe"
  RMDir "$INSTDIR"
  DeleteRegKey HKCU "${UNINSTALL_KEY}"
SectionEnd
