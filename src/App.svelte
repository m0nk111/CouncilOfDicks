<script lang="ts">
  import ChatInterface from "./ChatInterface.svelte";
  import ProvidersPanel from "./ProvidersPanel.svelte";
  import CouncilPanel from "./CouncilPanel.svelte";
  import PoHVIndicator from "./PoHVIndicator.svelte";
  import TopicControl from "./TopicControl.svelte";
  import ConstitutionPanel from "./ConstitutionPanel.svelte";

  let showSettings = false;
  let showCouncil = false;
  let showTopic = false;
  let showConstitution = false;
</script>

{#if showSettings}
  <!-- Settings Modal -->
  <div
    class="modal-overlay"
    role="dialog"
    aria-modal="true"
    aria-labelledby="settings-modal-title"
    tabindex="-1"
    on:click|self={() => (showSettings = false)}
    on:keydown={(event) => event.key === "Escape" && (showSettings = false)}
  >
    <div class="modal-content" role="document">
      <div class="modal-header">
        <h2 id="settings-modal-title">⚙️ Settings</h2>
        <button class="close-btn" on:click={() => (showSettings = false)}>✕</button>
      </div>
      <ProvidersPanel />
    </div>
  </div>
{/if}

{#if showCouncil}
  <!-- Council Modal -->
  <div
    class="modal-overlay"
    role="dialog"
    aria-modal="true"
    aria-labelledby="council-modal-title"
    tabindex="-1"
    on:click|self={() => (showCouncil = false)}
    on:keydown={(event) => event.key === "Escape" && (showCouncil = false)}
  >
    <div class="modal-content modal-wide" role="document">
      <div class="modal-header">
        <h2 id="council-modal-title">🏛️ Council</h2>
        <button class="close-btn" on:click={() => (showCouncil = false)}>✕</button>
      </div>
      <CouncilPanel />
    </div>
  </div>
{/if}

{#if showTopic}
  <!-- Topic Modal -->
  <div
    class="modal-overlay"
    role="dialog"
    aria-modal="true"
    aria-labelledby="topic-modal-title"
    tabindex="-1"
    on:click|self={() => (showTopic = false)}
    on:keydown={(event) => event.key === "Escape" && (showTopic = false)}
  >
    <div class="modal-content" role="document">
      <div class="modal-header">
        <h2 id="topic-modal-title">📢 Topic Channel</h2>
        <button class="close-btn" on:click={() => (showTopic = false)}>✕</button>
      </div>
      <TopicControl />
    </div>
  </div>
{/if}

{#if showConstitution}
  <!-- Constitution Modal -->
  <div
    class="modal-overlay"
    role="dialog"
    aria-modal="true"
    aria-labelledby="constitution-modal-title"
    tabindex="-1"
    on:click|self={() => (showConstitution = false)}
    on:keydown={(event) => event.key === "Escape" && (showConstitution = false)}
  >
    <div class="modal-content modal-wide" role="document">
      <div class="modal-header">
        <h2 id="constitution-modal-title">📜 Constitution</h2>
        <button class="close-btn" on:click={() => (showConstitution = false)}>✕</button>
      </div>
      <ConstitutionPanel />
    </div>
  </div>
{/if}

<!-- Main Chat Interface -->
<ChatInterface 
  on:showSettings={() => (showSettings = true)}
  on:showCouncil={() => (showCouncil = true)}
  on:showTopic={() => (showTopic = true)}
  on:showConstitution={() => (showConstitution = true)}
/>

<PoHVIndicator />

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

  .modal-wide {
    max-width: 1200px;
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
