# Dispatch menu hover stays inside the menu

Pointer to `.ai-memory/rca/20-dispatch-menu-hover-clipped.md`.

The Destination hover that reads `Destination conversation mo` is the browser `title` tooltip. The Send menu is `absolute right-0`, so the tooltip opens past the window edge. Use `DispatchFieldHint` anchored `right-0` inside the popover. Do not put `title` back on those controls, and do not put the hint element inside a `<select>`.
