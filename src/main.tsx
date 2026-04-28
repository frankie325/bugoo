import React from "react";
import ReactDOM from "react-dom/client";
import { HeroUIProvider } from "@heroui/react";
import App from "./App";
import { FloatWindowPage } from "./FloatWindowPage";
import "./styles/globals.css";

function Root() {
  const params = new URLSearchParams(window.location.search);
  const textParam = params.get('text');

  if (textParam) {
    return <FloatWindowPage text={decodeURIComponent(textParam)} />;
  }

  return <App />;
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <HeroUIProvider>
      <Root />
    </HeroUIProvider>
  </React.StrictMode>
);
