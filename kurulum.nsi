; Kitapcık Yazdır kurulum betiği (NSIS 3, Türkçe, kullanici-düzeyi, yöneticisiz).
Unicode True
!include "MUI2.nsh"

Name "Kitapçık Yazdır"
OutFile "KitapcikYazdir-0.3-Kurulum.exe"
InstallDir "$LOCALAPPDATA\Programs\Kitapçık Yazdır"
RequestExecutionLevel user
SetCompressor /SOLID lzma

!define MUI_ICON "assets\icon.ico"
!define MUI_HEADERIMAGE
!define MUI_HEADERIMAGE_BITMAP "assets\ust.bmp"
!define MUI_WELCOMEFINISHPAGE_BITMAP "assets\sol.bmp"
!define MUI_FINISHPAGE_RUN "$INSTDIR\kitapcik-gui.exe"
!define MUI_FINISHPAGE_RUN_TEXT "Kitapçık Yazdır'ı çalıştır"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "Turkish"

Section "Program" SecProg
  SetOutPath $INSTDIR
  File "paket\kitapcik-gui.exe"
  File "paket\kitapcik-yazdir.exe"
  File "paket\README.md"
  File "paket\KILAVUZ.md"
  WriteUninstaller "$INSTDIR\kaldir.exe"
  CreateShortcut "$DESKTOP\Kitapçık Yazdır.lnk" "$INSTDIR\kitapcik-gui.exe" "" "$INSTDIR\kitapcik-gui.exe" 0
  CreateDirectory "$SMPROGRAMS\Kitapçık Yazdır"
  CreateShortcut "$SMPROGRAMS\Kitapçık Yazdır\Kitapçık Yazdır.lnk" "$INSTDIR\kitapcik-gui.exe" "" "$INSTDIR\kitapcik-gui.exe" 0
  CreateShortcut "$SMPROGRAMS\Kitapçık Yazdır\Kaldır.lnk" "$INSTDIR\kaldir.exe"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\KitapcikYazdir" "DisplayName" "Kitapçık Yazdır"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\KitapcikYazdir" "DisplayVersion" "0.3"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\KitapcikYazdir" "Publisher" "Kitapçık Yazdır"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\KitapcikYazdir" "UninstallString" "$INSTDIR\kaldir.exe"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\KitapcikYazdir" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\KitapcikYazdir" "DisplayIcon" "$INSTDIR\kitapcik-gui.exe"
SectionEnd

Section "Uninstall"
  Delete "$INSTDIR\kitapcik-gui.exe"
  Delete "$INSTDIR\kitapcik-yazdir.exe"
  Delete "$INSTDIR\README.md"
  Delete "$INSTDIR\KILAVUZ.md"
  Delete "$INSTDIR\kaldir.exe"
  RMDir "$INSTDIR"
  Delete "$DESKTOP\Kitapçık Yazdır.lnk"
  Delete "$SMPROGRAMS\Kitapçık Yazdır\Kitapçık Yazdır.lnk"
  Delete "$SMPROGRAMS\Kitapçık Yazdır\Kaldır.lnk"
  RMDir "$SMPROGRAMS\Kitapçık Yazdır"
  DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\KitapcikYazdir"
SectionEnd
