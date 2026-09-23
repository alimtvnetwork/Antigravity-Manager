!macro NSIS_HOOK_POSTINSTALL
    WriteRegStr SHCTX "${UNINSTKEY}" "DisplayName" "Antigravity Manager Tools"
    WriteRegStr SHCTX "${UNINSTKEY}" "Publisher" "Maintained by Alim, Sponsored by RISEUP ASIA LLC"
!macroend
