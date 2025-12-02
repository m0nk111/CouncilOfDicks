<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { pohvGetStatus, pohvHeartbeat, type PoHVState } from "./api";

  let state: PoHVState | null = null;
  let interval: any;
  let loading = false;

  async function updateStatus() {
    try {
      state = await pohvGetStatus();
    } catch (e) {
      console.error("Failed to get PoHV status:", e);
    }
  }

  async function sendHeartbeat() {
    loading = true;
    try {
      state = await pohvHeartbeat();
    } catch (e) {
      console.error("Failed to send heartbeat:", e);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    updateStatus();
    interval = setInterval(updateStatus, 1000);
  });

  onDestroy(() => {
    if (interval) clearInterval(interval);
  });

  function formatTime(seconds: number): string {
    const m = Math.floor(seconds / 60);
    const s = seconds % 60;
    return `${m}:${s.toString().padStart(2, "0")}`;
  }
</script>

{#if state}
  <div class="pohv-indicator" class:warning={state.status === "Warning"} class:locked={state.status === "Locked"}>
    <div class="status-row">
      <span class="icon">❤️</span>
      <span class="timer">{formatTime(state.seconds_remaining)}</span>
    </div>
    
    {#if state.status === "Locked"}
      <div class="lock-overlay">
        <h1>⚠️ SYSTEM LOCKED ⚠️</h1>
        <p>Proof of Human Value required.</p>
        <button class="heartbeat-btn large" on:click={sendHeartbeat} disabled={loading}>
          {loading ? "Verifying..." : "I AM HUMAN"}
        </button>
      </div>
    {:else}
      <button class="heartbeat-btn" on:click={sendHeartbeat} disabled={loading} title="Send Heartbeat">
        Verify
      </button>
    {/if}
  </div>
{/if}

<style>
  .pohv-indicator {
    position: fixed;
    bottom: 20px;
    right: 20px;
    background: rgba(0, 0, 0, 0.8);
    border: 1px solid #333;
    border-radius: 30px;
    padding: 8px 16px;
    display: flex;
    align-items: center;
    gap: 12px;
    z-index: 1000;
    transition: all 0.3s;
    backdrop-filter: blur(10px);
  }

  .pohv-indicator.warning {
    background: rgba(255, 152, 0, 0.2);
    border-color: #ff9800;
    animation: pulse 1s infinite;
  }

  .status-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .icon {
    font-size: 1.2rem;
  }

  .timer {
    font-family: monospace;
    font-size: 1.1rem;
    font-weight: bold;
    color: #e0e0e0;
  }

  .heartbeat-btn {
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    color: white;
    border-radius: 20px;
    padding: 4px 12px;
    cursor: pointer;
    font-size: 0.8rem;
    transition: all 0.2s;
  }

  .heartbeat-btn:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.2);
  }

  /* Lock Overlay */
  .lock-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    background: rgba(0, 0, 0, 0.95);
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    z-index: 2000;
  }

  .lock-overlay h1 {
    color: #ff4444;
    font-size: 3rem;
    margin-bottom: 1rem;
    text-transform: uppercase;
    letter-spacing: 4px;
  }

  .lock-overlay p {
    color: #888;
    margin-bottom: 2rem;
    font-size: 1.2rem;
  }

  .heartbeat-btn.large {
    font-size: 1.5rem;
    padding: 1rem 3rem;
    background: #ff4444;
    border: none;
    font-weight: bold;
    letter-spacing: 2px;
    animation: pulse 2s infinite;
  }

  .heartbeat-btn.large:hover {
    background: #ff6666;
    transform: scale(1.05);
  }

  @keyframes pulse {
    0% { box-shadow: 0 0 0 0 rgba(255, 68, 68, 0.4); }
    70% { box-shadow: 0 0 0 20px rgba(255, 68, 68, 0); }
    100% { box-shadow: 0 0 0 0 rgba(255, 68, 68, 0); }
  }
</style>
