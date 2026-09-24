import React from "react";
import ReactDOM from "react-dom/client";
import App from './App';
import './i18n'; // Import i18n config
import "./App.css";

import { invoke } from "@tauri-apps/api/core";
import { isTauri } from "./utils/env";
import { ErrorBoundary } from "./components/common/ErrorBoundary";

if (isTauri()) {
  invoke("show_main_window").catch(console.error);
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ErrorBoundary>
      <App />
    </ErrorBoundary>
  </React.StrictMode>,
);
