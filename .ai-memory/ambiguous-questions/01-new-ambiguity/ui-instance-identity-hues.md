# Ambiguity: INSTANCE_THEMES identity hues (data-ink exception)

Status: proceeding by best judgment.

## Question

`src/pages/Instances.tsx` `INSTANCE_THEMES` (7 entries: Indigo/Emerald/Cyan/
Amber/Sky/Teal/Slate, lines ~130-180) assigns each profile a distinct identity
hue used for accent bars, badges, email pills, and dots. Plan-T1 demands zero
literal Tailwind colors, but daisyUI semantics offer only ~6 distinct hues —
and on `riseup-dark`, `primary`/`accent`/`warning` (`#FFAD01`/`#FFC24A`/`#FF9346`)
collide into one amber family, while `info` renders violet (`#9B3FD1`),
violating the taste ban and single-accent discipline. Collapsing 7 functional
identity hues onto semantics destroys at-a-glance profile distinction.

## Decision taken

Keep the 7 literal hues as a **data-ink exception** (same class as chart series
colors and the spec-02 code-block exception: data-ink, not theme-ink), with an
in-code comment marking the block. Only the theme-coupled glow shadows on the
accent bar (`rgba(59,130,246,…)` blue, wrong for 6 of 7 themes and above the
spec-02 `/0.25` hover-glow ceiling) are removed; the opacity/scale hover motion
stays.

## If overturned

Map the 7 entries onto `primary/secondary/accent/info/success/warning/neutral`
(or a cited 7-step categorical ramp), accept the RiseUp amber collision, and
re-verify profile distinguishability on both RiseUp themes plus stock dark.
