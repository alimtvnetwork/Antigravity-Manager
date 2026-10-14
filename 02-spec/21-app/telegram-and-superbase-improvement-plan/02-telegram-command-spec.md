# Telegram window, screenshot, and aimed prompt

## Commands

| Send | Result |
| :--- | :--- |
| `/ui` or `/agm ui` | If `agm-alim` is already running, say so. Otherwise start the manager window. |
| `/shot`, `/screenshot`, or `/agm shot` | Open the manager only when it is closed. Capture its window. Send the PNG to the chat that asked. |
| `/target <instance> <conversation>` | Remember that pair for 10 minutes. Example: `/target default C001`. |
| `/target` | Show the current aim. |
| `/target clear` | Forget the aim. |
| next plain message | Insert that text as the prompt for the aimed instance and conversation. |
| `/prompt C001 --instance default <text>` | One-message form. Unchanged. |
| `agm ui` | CLI open. |
| `agm shot` | CLI capture. Prints the byte count. Exit 1 when capture fails. |

Copy buttons on `/help` use Telegram `copy_text`. The button label is short. The copied text is the command, ready to paste and edit.

## Errors

A missing window, a helper failure, or a Telegram photo rejection is returned as text. A failed screenshot does not hide a prompt-dispatch error. A successful `/ui` does not claim a picture was taken.

## Not in this change

- Speech-to-text for Telegram voice notes.
- Clicking an IDE conversation row from the chat.
- One command that screenshots every SSH node and uploads each PNG.
