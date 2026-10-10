import React from "react";
import ReactDOM from "react-dom/client";
import App from './App';
import './i18n'; // Import i18n config
import "./App.css";
import "./styles/ui-tokens.css"; // Semantic UI tokens (refined dark product UI)

import { invoke } from "@tauri-apps/api/core";
import { isTauri } from "./utils/env";
import { ErrorBoundary } from "./components/common/ErrorBoundary";
import { useErrorStore } from "./stores/error-store";

if (isTauri()) {
  invoke("show_main_window").catch((e) => {
    // Tracked in the error module; startup window show is best-effort, OS window manager may already show it.
    useErrorStore.getState().trackWarning(e, {
      source: 'main.showMainWindow',
      endpoint: 'show_main_window',
      triggerAction: 'show_main_window',
    });
    console.error(e);
  });
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ErrorBoundary>
      <App />
    </ErrorBoundary>
  </React.StrictMode>,
);
