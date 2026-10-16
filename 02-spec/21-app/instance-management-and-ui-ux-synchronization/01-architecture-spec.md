# Instance list, stop, switch, and audit

The list status badge shows the OS process id as `PID <n>`. The data-path cell and the executable cell show the ending of each path on one row.

Stop and account switch call a verified close. A saved Antigravity PID is included when its executable matches that instance. If the PID is still alive after `taskkill`, stop and switch return an error and do not rewrite credentials.

`agm instances current` walks the parent process chain and prints the instance that owns this command. Stop, switch, restart, and delete refuse that instance.

Switch history is returned as `recent_switches` and `total_switches`, which is what the audit dialog reads.
