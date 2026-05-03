import React from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import { applyDesktopDesignTokens } from "./designTokens";
import "./styles.css";

const rootElement = document.getElementById("root");

if (!rootElement) {
  throw new Error("Desktop root element was not found.");
}

applyDesktopDesignTokens();

createRoot(rootElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
