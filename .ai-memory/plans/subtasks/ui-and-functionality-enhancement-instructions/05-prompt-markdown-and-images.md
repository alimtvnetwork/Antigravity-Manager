# Subtask 05: Prompt Markdown, Compact Toolbar, Reduced Rounding & Details Modal

Parent Task: `ui-and-functionality-enhancement-instructions`
Task-ID: `Task-06` & `Task-07`
Owner: Worker 02

## Objectives
1. Button Rounding Audit: Replace `rounded-full` with `rounded-md` across `ModalHeader.tsx`, `InspectorModal.tsx`, `PromptTreeExportModal.tsx`, `PromptTreeImportModal.tsx`, `ResultsView.tsx`, `VerificationModal.tsx`, and `PromptImagePreviewModal.tsx`.
2. Compact Toolbar: Position Input/Output buttons on the right-hand side adjacent to Expand Full.
3. Details Modal Restoration: Pass `setIsDetailsModalOpen` to `<DetailHeader />` and runtime instance props (`instanceExeName`, `instancePid`, etc.) from `PromptTreeViewModal.tsx` to `<DetailsModal />`.
4. Multimodal Clipboard & Dual Copy: Ensure header has "Copy (includes image)" and "Copy without image". Ensure image context menu has "Copy Image to Clipboard" and "Copy All Attached Images".
