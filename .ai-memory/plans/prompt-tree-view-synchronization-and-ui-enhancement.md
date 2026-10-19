# Master Execution Plan: Prompt Tree View Synchronization and UI Enhancement

## Metadata
- **Task**: Prompt Tree View Synchronization and UI Enhancement
- **Slug**: `prompt-tree-view-synchronization-and-ui-enhancement`
- **Budget**: 400 Steps (Phase 1: 1..100, Phase 2: 101..250, Phase 3: 251..400)
- **Concurrency**: A = 3, H = 2, Capacity = 6
- **Status**: COMPLETED

## User Request (Verbatim)
```text
If you see these two screenshots, these two screenshots show you that the prompt history does not match. Here, you only got the literally stuff, which is the first one, which is now run. But you did not match the other stuff like coding guideline prompts, lead map other prompt, which is six hours ago, which does not match in your tree view. Where is that? It does not match, and you don't have the time that I could understand where and how long ago it runs, which was very critical to understand. I need to see when the last time the prompt actually ran. That is not there. Make sure that you correct it. It's getting several times. I'm actually getting frustrated. Try to fix the prompt tree view properly. Try to understand the synchronization from the actual history of that IDE and what we have right now. And if you do, let's say, the reset button also does not work, and also if we go to a new version, try to make sure the version number is mentioned. If we update the version, make sure that we reset the prompt tree so that we can reload and understand it. And try to make the UI more look pretty. Currently, it's not. It's really bad. Again, it's just flat, no high-quality front-end design. And I've been asking several times, and the Telegram UI also needs to have proper professional view, more options. The image needs to be sent and copied back from the Telegram as well for the prompts, which is not available. Please make sure that you follow through and implement this.
```

## Visual References
- Screenshot 1: `assets/screenshots/prompt-tree-view-synchronization-and-ui-enhancement-01.png` (IDE Conversation History)
- Screenshot 2: `assets/screenshots/prompt-tree-view-synchronization-and-ui-enhancement-02.png` (AGM Prompt Tree View)

## Subtasks
1. [01-transcript-synchronization-and-duration-fix.md](file:///.ai-memory/plans/subtasks/prompt-tree-view-synchronization-and-ui-enhancement/01-transcript-synchronization-and-duration-fix.md): Fix duration arithmetic (resolve 8h timezone offset bug `11h 39m -> 3h 39m`), fix cross-project transcript discovery in `repo_db`, and fix conversation nesting.
2. [02-relative-time-badges-and-tree-hierarchy.md](file:///.ai-memory/plans/subtasks/prompt-tree-view-synchronization-and-ui-enhancement/02-relative-time-badges-and-tree-hierarchy.md): Add relative recency time badges (`now`, `30m`, `6h`, `12h`, `1d`) with color-coded status dots to conversation cards and tree nodes.
3. [03-reset-button-and-version-invalidation.md](file:///.ai-memory/plans/subtasks/prompt-tree-view-synchronization-and-ui-enhancement/03-reset-button-and-version-invalidation.md): Implement backend cache purge command and hook frontend Reset button; automatically invalidate cache on version changes; display version in header.
4. [04-modern-dark-glass-ui-and-telegram-enhancements.md](file:///.ai-memory/plans/subtasks/prompt-tree-view-synchronization-and-ui-enhancement/04-modern-dark-glass-ui-and-telegram-enhancements.md): Redesign Prompt Tree View with high-end dark glass, pill capsules, and update Telegram UI with professional cards, inline buttons, and prompt image send/copy support.
