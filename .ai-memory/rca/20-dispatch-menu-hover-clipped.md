# RCA 20: Dispatch menu hover is clipped at the window edge

Read this before adding a tooltip, `title`, or hover label on the prompt-tree header, the Send menu, or any right-aligned popover.

## Symptom

Hovering Destination in Dispatch Configuration shows a white browser tooltip that reads `Destination conversation mo` and runs off the right edge of the window. The last word, `mode`, is cut. The same clip happens to any other native tooltip in that menu.

## Root cause

`src/components/instances/PromptTreeView/DetailHeader.tsx` put the explanation on the `<select>` with `title="Destination conversation mode"`. The Send menu is `absolute right-0` inside the header capsule, which sits on the right edge of the window. The operating-system tooltip opens to the right of the cursor, so the window clips it. A `title` is not a positioned element. `overflow-visible`, `z-50`, and a wider menu do not move it.

## Fix

The hint is an in-menu element, `DispatchFieldHint`, anchored `right-0` on the row (`group/field`). It grows to the left, wraps inside `max-w-full`, and stays inside the popover. The `<select>` and the copy-menu rows no longer use `title`. The same hint covers Destination, Suffix, and the three Copy AI rows.

## Mistakes that bring the clip back

- Putting `title=` back on a control inside the Send menu, the Copy menu, or any `absolute right-0` popover.
- Rendering the hint with `left-0` or `left-full` so it grows toward the window edge.
- Putting the hint element inside the `<select>`. A `<span>` there is not shown.
- Using the native white tooltip and calling the dark menu “fixed” because the menu itself is on screen. The menu can be fully visible while the tooltip is not.
- Closing the menu from `onMouseLeave` and then drawing the hint outside the menu. The hint must be a child of the popover.

## Must-follow

- Explanations for controls in a right-edge popover are in-menu text anchored to the right edge of the row.
- Do not use the browser `title` attribute in `DetailHeader.tsx` popovers.
- A new hover string must be readable in full without leaving the popover. If it does not fit, wrap it. Do not let it cross the window edge.
