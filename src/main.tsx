import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles/globals.css";

const savedTheme = localStorage.getItem("synk-theme") || "dark";
if (savedTheme === "dark") {
  document.documentElement.classList.add("dark");
}

if (import.meta.env.DEV) {
  import("@axe-core/react").then((axe) => {
    axe.default(React, ReactDOM, 1000);
  });
}

const rootElement = document.getElementById("root");
if (!rootElement) throw new Error("Element #root introuvable dans index.html");

ReactDOM.createRoot(rootElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
