import "./styles.css";
import { mount } from "svelte";
import App from "./App.svelte";

console.log("🔧 Council Of Dicks - Initializing...");
console.log('🌍 Environment:', {
  userAgent: navigator.userAgent,
  hasTauri: typeof window !== 'undefined' && (window as any).__TAURI_INTERNALS__ !== undefined,
  location: window.location.href,
});

const appElement = document.getElementById("app");
console.log("📍 App element found:", appElement);

let appInstance: any = null;

if (!appElement) {
  console.error("❌ No #app element found in DOM!");
} else {
  try {
    console.log("🚀 Mounting Svelte 5 App...");
    appInstance = mount(App, {
      target: appElement,
    });
    console.log("✅ App mounted successfully:", appInstance);
    
    (window as any).__COUNCIL_APP__ = appInstance;
  } catch (error) {
    console.error("❌ Failed to mount app:", error);
    appElement.innerHTML = `
      <div style="padding: 20px; color: red;">
        <h1>❌ Failed to mount app</h1>
        <pre>${error}</pre>
      </div>
    `;
  }
}

export default appInstance;
