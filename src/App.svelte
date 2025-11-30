<script lang="ts">
  import ChatInterface from "./ChatInterface.svelte";
  import ProvidersPanel from "./ProvidersPanel.svelte";

  let showSettings = false;
</script>

{#if showSettings}
  <!-- Settings Modal -->
  <div class="modal-overlay" on:click={() => (showSettings = false)} role="dialog">
    <div class="modal-content" on:click|stopPropagation role="document">
      <div class="modal-header">
        <h2>⚙️ Settings</h2>
        <button class="close-btn" on:click={() => (showSettings = false)}>✕</button>
      </div>
      <ProvidersPanel />
    </div>
  </div>
{/if}

<!-- Main Chat Interface -->
<ChatInterface on:showSettings={() => (showSettings = true)} />

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
  }

  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.8);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal-content {
    background: #16213e;
    border-radius: 12px;
    padding: 2rem;
    max-width: 800px;
    max-height: 90vh;
    overflow-y: auto;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
  }

  .modal-header h2 {
    margin: 0;
    color: #00d4ff;
    font-size: 1.5rem;
  }

  .close-btn {
    background: none;
    border: none;
    font-size: 1.5rem;
    color: #999;
    cursor: pointer;
    padding: 0.25rem 0.5rem;
    transition: color 0.2s;
  }

  .close-btn:hover {
    color: #fff;
  }
</style>
