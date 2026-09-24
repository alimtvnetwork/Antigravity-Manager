# Specification: Instance Lifecycle, Adaptive Quota Ladder, Navbar Mutual Exclusion, and Plaintext Emails (v4.70.0)

- **Spec ID:** `02-spec/21-app/70-instance-lifecycle-switcher-navbar-and-plaintext-email-v470.md`
- **Release Version:** `v4.70.0`
- **Status:** `active`
- **Author:** Antigravity AI Orchestrator

---

## 1. Domain Architecture & Objectives

This specification defines the architectural standards, data contracts, and verification gates for release `v4.70.0`, addressing six core operational observations:
1. **Instance Deletion Lifecycle & Native Sandboxing**: Eliminate directory locking crashes and infinite open/close restart loops by terminating child processes prior to directory removal and launching native `Antigravity.exe` with `--user-data-dir` parameter isolation.
2. **Auto-Switcher 3-Tier Interval Ladder & Gemini 3.8 Flash High**: Implement dynamic polling (300s safe -> 60s caution <15% -> 40s critical <=12%), default model evaluation to `gemini-3.8-flash-high`, and persist prompt snapshots to SQLite upon rotation.
3. **Navbar Collision Eradication & Toolbar Consolidation**: Implement `agm:dropdown-open` custom event broadcast for mutual exclusion between Accounts and Instances dropdowns; relocate `Bug` button to brand logo area; consolidate Theme + Language into a single dropdown leaving only Quick Clean; consolidate Import + Export into a single icon dropdown with `z-[9999]` tooltips.
4. **Window Focus Stealing Eradication**: Set `auto_focus_window: false` by default; ensure crash watchdog `check_and_recover_crashed_instance` is strictly silent (cleans lockfiles without popping up foreground windows).
5. **100% Plaintext Subject-Driven Emails & Telegram Alignment**: Strip all HTML tags from outgoing emails; enforce command in Subject and pure prompt in Body; strip inbound footers and terminal prefixes; horizontally align Telegram inputs (`items-end`) and provide `03-ai-scripts/telegram-bot-helper.ps1`.
6. **Isolated 4-Block Installer Commands**: Guarantee 4 separate, distinct code blocks for Windows Direct, Windows Pinned, Linux/macOS Direct, and Linux/macOS Pinned across all release documentation.

---

## 2. User Request (Verbatim)

```text
is it done properly and released??


https://prnt.sc/2IgVGqVvdOAo
https://prnt.sc/ydgq-Y5feY5i
https://prnt.sc/3PFE9zj0cykl
https://prnt.sc/F_11aCEaoMCb
https://prnt.sc/ncs3mfil2FAk
https://prnt.sc/GSP_AluULzuX
https://prnt.sc/FDOMrN6T63P9
https://prnt.sc/dDUKpPnBaJ37
https://prnt.sc/mDxMODvqzp-h
https://prnt.sc/VjAqwmFIFY_b

Okay. So far, I have lots of observation. Let's start with the important ones. First, the instance section, the delete button does not work, so you need to fix it immediately. The delete button is a chaotic situation, okay? And it does not work at all. Okay, so that actually gives a error trace. I am adding the error trace. And also the default or the, let's say, instance creation, has a serious issue. I'm going to explain that. The issue with the instance creation is that once the instance is created, it cannot manage itself. So it just closes and open and closes and open. So that's a terrible thing to look at, okay? So in this case, what I want from you is to test it. Test it here with end-to-end testing locally. You can run it, try to create a instance, delete it. Okay? And also, you should have these, let's say, command line commands to do that. And at the end, you show me these command line commands. Okay? Also, another point which I have, let's say, reported several times. The Anti-Gravity release page combines the installation several times, which I asked you several times not to do, and you're still doing it. Does not make any sense. Okay? There are other errors, which I will give you the screenshots and things, okay, which basically you can look at. The next problem that we have is that the account switch, auto switch section, we have the settings, right? So it should automatically check in every-- So by default, it should automatically check in every 300 milliseconds. That means five minutes, right? Only if it goes below the, let's say, 12% or 10%, then it will check every... Sorry, under 15%, it will check every 60 seconds. Under, let's say, 12%, it will check every 40 seconds. Okay? So basically, once it goes below 15%, so put that as a default, it should automatically switch to the next workspace and also push the current running prompt to send now to that project repo. Okay? So that is kind of must. We have been discussing with this several times, and you are not fixing it. It's not polite. I want you to test this. Probably what you could do is put the threshold to 90% to testing, and then you do this here in this machine. Do everything you want. No worries on this, okay? Because this is snapshot VM I could restore. So don't take any problems, but you commit. Every code you write, you must commit as a group so that before testing, we must commit. Because otherwise, any crash could be fatal. The code can be lost. Remember that. Also, here in the image, you can see Gemini Pro is primary evaluated model. No. The default one would be Gemini Flash-8 High. That would be the default model, which I will give you the screenshot as well. So these are on top of my head, these are really creating big issues. The account switching is not working. Also, there are issues if I click on the accounts and click on the instances. It just looks like colliding with each other, where the other dropdown should be closed automatically. Okay? That's kind of the standard feeling which I do not get here. I think you need to fix that. So if we go inside now, and right-hand side, we have lots of button. I don't find a reason to have this debug button where we could move all the debug things to the left-hand icon. Again, I'm going to give you the screenshot, so you look into all these screenshots, okay, so that nothing remains unsolved. All right. And also the theme color and language, both of those can be on a dropdown, okay, rather than two buttons. Only the recycle button could be there. That's all right. I appreciate that. And also here, the import/export should be combined to one dropdown button that is icon. And when I hover over, I should be seeing the icon tooltip, which I do not see. I think you have integrated, but the tooltip goes below the other stuff, which is also very wrong. You do not do the proper designing and color stuff, which is absolutely bad. You need to focus on this. Now, if we go into the email section, the email formatting and the dispatch, as I mentioned, these are not yet fixed The PowerShell section is also not improved. Okay. And also the Telegram section also has some alignment issues. Okay. And also I want you to create a Telegram that's a bot, step by step using PowerShell. So Telegram account is integrated here in desktop. You could do end-to-end testing to make sure that you can create the bot and things like that. Okay. Look into all the image so that you understand the image, make sure of that. And also Telegram needs to be connected in order to check how this will work. For example, the interactive remote control section, you don't need to pass PowerShell clone to the terminal. That would actually give an error. Okay, so you just pass the command to PowerShell and that would work. Currently, in your case, it is not happening and creating more issues. I hope you understand and respect that. So here, one important factor is the account switching that needs to be happening automatically and email needs to be sent when this happens. These are not happening yet. You need to send me all the sample email that will contain all this email formatting. How I can interact with this email. So this needs to be there, which you didn't do it yet. Okay. In your formatting, it is not there, which is also very bad. Disrespectful. Okay, one more thing. Sometimes when you're working with, let's say, Windows Explorer or other place, it immediately switch backs to Antigravity IDE. Why it happens, find the root cause, and don't do it. Okay? Probably you do it because I said sometimes Antigravity becomes blank. I think this is why you are doing it. But I think just switching to that does not solve the problem. Okay? So you could try to find the root cause of it in the future. Okay? But not this way. And I want you to write this into the documents. Okay, so these are the factors I think you need to work on. Okay, now I received the emails as a format, which is also very terrible, as I mentioned several times. In your email, there is no need to have any HTML formatting nicely. You just put the text that I can understand and reply back. Okay? So most of the things would be done in the subject. Only the prompt section would be sent into the body. Okay? So I think we have discussed this in other prompts. You can look into your specs properly. I hope it's clear, right? If you have any question, confusion, let me know.
```

---

## 3. Visual Assets

All user screenshots are decoded and permanently preserved in the repository:
- `assets/screenshots/prnt-2IgVGqVvdOAo.png`
- `assets/screenshots/prnt-ydgq-Y5feY5i.png`
- `assets/screenshots/prnt-3PFE9zj0cykl.png`
- `assets/screenshots/prnt-F_11aCEaoMCb.png`
- `assets/screenshots/prnt-ncs3mfil2FAk.png`
- `assets/screenshots/prnt-GSP_AluULzuX.png`
- `assets/screenshots/prnt-FDOMrN6T63P9.png`
- `assets/screenshots/prnt-dDUKpPnBaJ37.png`
- `assets/screenshots/prnt-mDxMODvqzp-h.png`
- `assets/screenshots/prnt-VjAqwmFIFY_b.png`

---

## 4. Verification Gates & Invariants

| Gate ID | Area | Required Verification |
| :--- | :--- | :--- |
| **VG-01** | Instance Lifecycle | CLI `--create-profile`, `--list-profiles`, and `--delete-profile` execute with exit code 0 |
| **VG-02** | Auto-Switcher Ladder | Unit tests confirm 300s at >=15%, 60s at <15%, 40s at <=12% |
| **VG-03** | Navbar Mutual Exclusion | `agm:dropdown-open` custom event verified across `NavMenu`, `InstanceSelector`, `NavSettings` |
| **VG-04** | Plaintext Emails | Outgoing email body contains 0 HTML tags; Subject holds command syntax |
| **VG-05** | Build Integrity | `npm run build` passes with exit code 0 |
| **VG-06** | Release Gate | Version bumped to `4.70.0`, tag pushed, GitHub Actions pipeline triggered |
