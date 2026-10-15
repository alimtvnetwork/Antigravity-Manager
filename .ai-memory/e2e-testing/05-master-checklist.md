# Master checklist

Copy this into the reply. Mark each line. A line left blank is not a pass.

```text
Date:
Binary version:
Commit:
Instance used:

Decisions
[ ] Binary version matches the commit
[ ] Builds were one at a time, -j 1
[ ] Switch and send used the copy instance unless the user named Default
[ ] Threshold ended at 15.0
[ ] No token, password, or prompt body in the reply

01 Prompt tree and CLI
[ ] Tree cold, cache kept, duration recorded
[ ] Tree warm, faster
[ ] Reset cleared cache, next tree cold
[ ] Send stayed running, manager stayed open
[ ] Queue fifo_ordered true
[ ] Backup exit 0
[ ] Restore exit 0 in seconds, no foreign key error
[ ] Window tree painted, or NOT SHOWN with a reason

02 Live switch and resend
[ ] ff on the copy skipped validation_blocked accounts
[ ] Default PID unchanged
[ ] Log: backed_up before switch, relaunch verified, same prompt id dispatched
[ ] agy log: silent auth succeeded, real conversation id

03 CDP and prompt in the IDE
[ ] Security Notice dismissed, share checkbox unchecked, Accept and continue
[ ] app storage: terms accepted, share data false
[ ] Second CLI start did not purge the new running_projects row
[ ] Composer in the IDE received the prompt
[ ] Prompt Tree Backup wrote a file and Restore dispatched it, or NOT CLICKED

04 Telegram, balance, SSH
[ ] agm balance table
[ ] agm balance --csv header row only on stdout
[ ] agm ssh status is the status subcommand, not an SSH login prompt
[ ] RSA test round-trip, or NOT RUN
[ ] Telegram cards have no literal nbsp, or bot NOT RUN

06 Recorder, UI, screenshots, Telegram images
[ ] agm ui record start, one action, list shows that action, then stop
[ ] agm ui open, then agm ui goto settings/ssh, no 404
[ ] agm ui ls settings/ssh before any scroll or click
[ ] agm shot sent the open window to the saved Telegram chat, or NOT RUN
[ ] Compared the new switch card to assets/screenshots/enhancing-telegram-ui-and-gitmap-integration-01.png
[ ] No literal &nbsp; on the new card
[ ] Buttons Check Balance, Export CSV, Prompt Instance, Refresh Status are under the card
[ ] Export CSV arrived as a file

Verdict
[ ] PASS or FAIL written per section
```
