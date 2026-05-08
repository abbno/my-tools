; NSIS Installer Hooks for SSH Tunnel Manager
; This file customizes the installer behavior

!macro customInit
  ; Set default installation directory to D:\Programs
  ; This is executed in .onInit before the directory selection page
  ${If} $INSTDIR == "${PLACEHOLDER_INSTALL_DIR}"
    StrCpy $INSTDIR "D:\Programs\${PRODUCTNAME}"
  ${EndIf}
!macroend