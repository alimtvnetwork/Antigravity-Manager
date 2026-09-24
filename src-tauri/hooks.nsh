!macro NSIS_HOOK_POSTINSTALL
    ReadRegStr $0 SHCTX "${UNINSTKEY}" "DisplayVersion"
    StrCmp $0 "" 0 +2
    StrCpy $0 "4.71.4"
    WriteRegStr SHCTX "${UNINSTKEY}" "DisplayName" "Antigravity Manager Tools $0"
    WriteRegStr SHCTX "${UNINSTKEY}" "Publisher" "Maintained by Alim, Sponsored by RISEUP ASIA LLC"
!macroend
