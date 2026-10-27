Unicode true
!include "MUI2.nsh"
!include "LogicLib.nsh"
!include "x64.nsh"
!include "FileFunc.nsh"
!include "Sections.nsh"
!define DESKTOP_SECTION_ID 1 ; Core=0, optional Desktop=1
!ifndef APPDIR
!error "APPDIR must point to the verified release folder"
!endif
!ifndef OUTFILE
!error "OUTFILE is required"
!endif
!ifndef APPICON
!error "APPICON is required"
!endif
Name "$(AppName)"
OutFile "${OUTFILE}"
InstallDir "$LOCALAPPDATA\Programs\Kartoteka"
InstallDirRegKey HKCU "Software\Popovantondev\Kartoteka" "InstallDir"
RequestExecutionLevel user
SetCompressor /SOLID lzma
SetDatablockOptimize on
BrandingText "popovantondev · Картотека 0.5.0"
VIProductVersion "0.5.0.0"
VIAddVersionKey /LANG=1049 "ProductName" "Одноклассники — Картотека"
VIAddVersionKey /LANG=1049 "FileDescription" "Установка Картотеки для текущего пользователя"
VIAddVersionKey /LANG=1049 "FileVersion" "0.5.0"
VIAddVersionKey /LANG=1049 "ProductVersion" "0.5.0"
VIAddVersionKey /LANG=1049 "LegalCopyright" "popovantondev"
!define MUI_ICON "${APPICON}"
!define MUI_UNICON "${APPICON}"
!define MUI_ABORTWARNING
!define MUI_WELCOMEPAGE_TITLE "$(Welcome)"
!define MUI_WELCOMEPAGE_TEXT "$(WelcomeText)"
!define MUI_FINISHPAGE_RUN "$INSTDIR\Kartoteka.exe"
!define MUI_FINISHPAGE_RUN_NOTCHECKED
!define MUI_FINISHPAGE_RUN_TEXT "$(RunApp)"
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!define MUI_UNCONFIRMPAGE_TEXT_TOP "$(UninstallText)"
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!define MUI_LANGDLL_REGISTRY_ROOT "HKCU"
!define MUI_LANGDLL_REGISTRY_KEY "Software\Popovantondev\Kartoteka"
!define MUI_LANGDLL_REGISTRY_VALUENAME "InstallerLanguage"
!insertmacro MUI_LANGUAGE "English"
!insertmacro MUI_LANGUAGE "German"
!insertmacro MUI_LANGUAGE "Russian"
!include "kartoteka-languages.nsh"
FileErrorText "$(FileBusy)"
Function .onInit
  !insertmacro MUI_LANGDLL_DISPLAY
  SetShellVarContext current
  ${GetParameters} $R0
  ${GetOptions} $R0 "/LANGUAGE=" $R2
  ${If} $R2 == "1031"
    StrCpy $LANGUAGE 1031
  ${ElseIf} $R2 == "1049"
    StrCpy $LANGUAGE 1049
  ${ElseIf} $R2 == "1033"
    StrCpy $LANGUAGE 1033
  ${EndIf}
  ${GetOptions} $R0 "/DESKTOP=" $R1
  ${If} $R1 == "1"
    SectionSetFlags ${DESKTOP_SECTION_ID} ${SF_SELECTED}
  ${EndIf}
  ${IfNot} ${RunningX64}
    MessageBox MB_ICONSTOP "$(Need64)"
    SetErrorLevel 2
    Abort
  ${EndIf}
FunctionEnd
Section "$(CoreName)" Core
  SectionIn RO
  SetShellVarContext current
  SetOutPath "$INSTDIR"
  ClearErrors
  File "${APPDIR}\Kartoteka.exe"
  File "${APPDIR}\libc++.dll"
  File "${APPDIR}\libunwind.dll"
  File "${APPDIR}\libwinpthread-1.dll"
  IfErrors 0 +3
    SetErrorLevel 1
    Abort "$(WriteError)"
  File /r "${APPDIR}\licenses"
  WriteUninstaller "$INSTDIR\Uninstall.exe"
  CreateDirectory "$SMPROGRAMS\Kartoteka"
  CreateShortcut "$SMPROGRAMS\Kartoteka\Kartoteka.lnk" "$INSTDIR\Kartoteka.exe" "" "$INSTDIR\Kartoteka.exe" 0
  CreateShortcut "$SMPROGRAMS\Kartoteka\Uninstall.lnk" "$INSTDIR\Uninstall.exe"
  WriteRegStr HKCU "Software\Popovantondev\Kartoteka" "InstallDir" "$INSTDIR"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Popovantondev.Kartoteka" "DisplayName" "$(AppName)"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Popovantondev.Kartoteka" "DisplayVersion" "0.5.0"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Popovantondev.Kartoteka" "Publisher" "popovantondev"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Popovantondev.Kartoteka" "UninstallString" '$\"$INSTDIR\Uninstall.exe$\"'
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Popovantondev.Kartoteka" "QuietUninstallString" '$\"$INSTDIR\Uninstall.exe$\" /S'
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Popovantondev.Kartoteka" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Popovantondev.Kartoteka" "DisplayIcon" "$INSTDIR\Kartoteka.exe,0"
  WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Popovantondev.Kartoteka" "NoModify" 1
  WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Popovantondev.Kartoteka" "NoRepair" 1
SectionEnd
Section /o "$(DesktopName)" Desktop
  CreateShortcut "$DESKTOP\Kartoteka.lnk" "$INSTDIR\Kartoteka.exe" "" "$INSTDIR\Kartoteka.exe" 0
SectionEnd
Function un.onInit
  !insertmacro MUI_UNGETLANGUAGE
FunctionEnd
Section "Uninstall"
  SetShellVarContext current
  Delete "$INSTDIR\Kartoteka.exe"
  Delete "$INSTDIR\libc++.dll"
  Delete "$INSTDIR\libunwind.dll"
  Delete "$INSTDIR\libwinpthread-1.dll"
  Delete "$INSTDIR\Uninstall.exe"
!include "license-uninstall.nsh"
  Delete "$SMPROGRAMS\Kartoteka\Kartoteka.lnk"
  Delete "$SMPROGRAMS\Kartoteka\Uninstall.lnk"
  RMDir "$SMPROGRAMS\Kartoteka"
  Delete "$SMPROGRAMS\Одноклассники — Картотека\Картотека.lnk"
  Delete "$SMPROGRAMS\Одноклассники — Картотека\Удалить программу.lnk"
  RMDir "$SMPROGRAMS\Одноклассники — Картотека"
  Delete "$DESKTOP\Картотека.lnk"
  Delete "$DESKTOP\Kartoteka.lnk"
  DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Popovantondev.Kartoteka"
  DeleteRegValue HKCU "Software\Popovantondev\Kartoteka" "InstallDir"
  DeleteRegValue HKCU "Software\Popovantondev\Kartoteka" "InstallerLanguage"
  DeleteRegKey /ifempty HKCU "Software\Popovantondev\Kartoteka"
  RMDir "$INSTDIR"
SectionEnd

