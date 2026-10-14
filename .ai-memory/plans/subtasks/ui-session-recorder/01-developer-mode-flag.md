# 01 Developer mode flag

Status: `PENDING`

Spec: `02-spec/21-app/ui-session-recorder/01-guideline.md`

Add `developerMode`. Default it to true until a production switch exists. Put Start / Stop recording on the existing error or settings surface. A normal production click does not write a trace. Recording runs only after Start, or when an error is captured while developer mode is on.
