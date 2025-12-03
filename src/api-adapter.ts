// Environment detection and API adapter for dual-mode deployment
// Supports both Tauri native app and web browser access

// Detect if running in Tauri (native app) or web browser
export function isTauriEnvironment(): boolean {
  return typeof window !== 'undefined' && 
         (window as any).__TAURI_INTERNALS__ !== undefined &&
         typeof (window as any).__TAURI_INTERNALS__.invoke === 'function';
}

// Lazy import for Tauri API (only when needed)
async function getTauriInvoke() {
  if (!isTauriEnvironment()) {
    throw new Error("Tauri not available in web mode");
  }
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke;
}

// Get API base URL for web mode
function getApiBaseUrl(): string {
  if (typeof window !== 'undefined') {
    // Use current origin in production, localhost:8080 in development
    return window.location.hostname === 'localhost' 
      ? 'http://localhost:8080'
      : `${window.location.protocol}//${window.location.hostname}:8080`;
  }
  return 'http://localhost:8080';
}

// Unified API call that works in both Tauri and web modes
export async function apiCall<T>(
  tauriCommand: string,
  httpEndpoint: string,
  params?: Record<string, any>
): Promise<T> {
  // Defensive check: Ensure we don't try to use Tauri if internals are missing
  const canUseTauri = isTauriEnvironment() && 
                      typeof window !== 'undefined' && 
                      (window as any).__TAURI_INTERNALS__ !== undefined;

  if (canUseTauri) {
    // Native app: use Tauri invoke (lazy import)
    try {
      const invoke = await getTauriInvoke();
      return await invoke<T>(tauriCommand, params || {});
    } catch (e) {
      console.warn("Tauri invoke failed, falling back to HTTP", e);
      // Fall through to HTTP
    }
  }
  
  // Web browser: use fetch
  const baseUrl = getApiBaseUrl();
    const method = httpEndpoint.startsWith('GET ') ? 'GET' : 'POST';
    const path = httpEndpoint.replace(/^(GET|POST) /, '');
    
    const options: RequestInit = {
      method,
      headers: {
        'Content-Type': 'application/json',
      },
    };
    
    if (method === 'POST' && params) {
      options.body = JSON.stringify(params);
    }
    
    const url = method === 'GET' && params
      ? `${baseUrl}${path}?${new URLSearchParams(params as any).toString()}`
      : `${baseUrl}${path}`;
    
    const response = await fetch(url, options);
    
    if (!response.ok) {
      const error = await response.json().catch(() => ({ error: response.statusText }));
      throw new Error(error.error || `HTTP ${response.status}`);
    }
    
    const result = await response.json();

    // Automatically unwrap ApiResponse if present
    // This handles the { success: true, data: T, error: null } wrapper from the Rust backend
    if (result && typeof result === 'object' && 'success' in result && 'data' in result) {
      if (!result.success) {
        throw new Error(result.error || "API Error");
      }
      return result.data;
    }

    return result;
}

// Log current mode on startup
if (typeof window !== 'undefined') {
  console.log(
    `🔧 Running in ${isTauriEnvironment() ? 'NATIVE' : 'WEB'} mode`,
    isTauriEnvironment() ? '(Tauri invoke)' : `(HTTP API: ${getApiBaseUrl()})`
  );
}
