!macro NSIS_HOOK_POSTINSTALL
    WriteRegStr SHCTX "${UNINSTKEY}" "DisplayName" "Antigravity Manager Tools by MD Alim Ul Karim and sponsored by RISE UP ASIA LLC"
    WriteRegStr SHCTX "${UNINSTKEY}" "Publisher" "Antigravity Manager Tools by MD Alim Ul Karim and sponsored by RISE UP ASIA LLC"
!macroend
