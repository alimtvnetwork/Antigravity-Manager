# Subtask 03: Frontend Split Restart Button & Distinct Sync Icons

- **Objective**: Implement contiguous split Stop/Restart capsule in Table and Card modes, update sync icons to `Cpu`/`FolderSync`, and preserve Switch button.
- **Details**:
  - `InstanceTable.tsx`: Split button when running (`Square` + `RotateCcw`), `Play` when stopped.
  - `Instances.tsx`: Card footer split button, `handleRestart` with toast and refresh, `Cpu` icon for Sync PIDs.
  - Switch button kept dedicated to selecting/switching account.
