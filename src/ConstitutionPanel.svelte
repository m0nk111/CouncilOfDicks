<script lang="ts">
  import { onMount } from "svelte";
  import { getConstitution, setConstitution } from "./api";

  let content = "Loading constitution...";
  let isEditing = false;
  let editContent = "";

  onMount(async () => {
    await loadConstitution();
  });

  async function loadConstitution() {
    try {
      content = await getConstitution();
      editContent = content;
    } catch (error) {
      content = `Error loading constitution: ${error}`;
    }
  }

  async function handleSave() {
    try {
      await setConstitution(editContent);
      content = editContent;
      isEditing = false;
    } catch (error) {
      alert(`Failed to save: ${error}`);
    }
  }

  // Simple Markdown to HTML converter for display
  function parseMarkdown(text: string): string {
    return text
      .replace(/^# (.*$)/gim, '<h1>$1</h1>')
      .replace(/^## (.*$)/gim, '<h2>$1</h2>')
      .replace(/^### (.*$)/gim, '<h3>$1</h3>')
      .replace(/\*\*(.*)\*\*/gim, '<strong>$1</strong>')
      .replace(/\*(.*)\*/gim, '<em>$1</em>')
      .replace(/^- (.*$)/gim, '<li>$1</li>')
      .replace(/\n/gim, '<br />');
  }
</script>

<div class="constitution-panel">
  <div class="header">
    <div class="title-group">
      <h2 class="constitution-title">📜 The Constitution</h2>
      <span class="subtitle">Rules of Engagement & Deliberation Flow</span>
    </div>
    <div class="actions">
      {#if isEditing}
        <button class="btn-cancel" on:click={() => (isEditing = false)}>Cancel</button>
        <button class="btn-save" on:click={handleSave}>Save Changes</button>
      {:else}
        <button class="btn-edit" on:click={() => (isEditing = true)}>Edit (Admin)</button>
      {/if}
    </div>
  </div>

  <div class="content-area">
    {#if isEditing}
      <textarea bind:value={editContent} spellcheck="false"></textarea>
    {:else}
      <div class="markdown-view">
        {@html parseMarkdown(content)}
      </div>
    {/if}
  </div>
</div>

<style>
  .constitution-panel {
    color: #e0e0e0;
    height: 100%;
    display: flex;
    flex-direction: column;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
    padding-bottom: 1rem;
    border-bottom: 1px solid #333;
  }

  .constitution-title {
    margin: 0;
    color: #ffd700;
  }

  .subtitle {
    font-size: 0.9rem;
    color: #888;
  }

  .content-area {
    flex: 1;
    overflow-y: auto;
    background: #0f172a;
    border-radius: 8px;
    padding: 1.5rem;
    border: 1px solid #333;
  }

  textarea {
    width: 100%;
    height: 100%;
    background: #0f172a;
    color: #e0e0e0;
    border: none;
    font-family: 'Fira Code', monospace;
    font-size: 0.9rem;
    resize: none;
    outline: none;
  }

  .markdown-view {
    line-height: 1.6;
  }

  /* Markdown Styles */
  :global(.markdown-view h1) { color: #ffd700; border-bottom: 1px solid #444; padding-bottom: 0.5rem; }
  :global(.markdown-view h2) { color: #64b5f6; margin-top: 1.5rem; }
  :global(.markdown-view h3) { color: #81c784; margin-top: 1rem; }
  :global(.markdown-view li) { margin-left: 1.5rem; }
  :global(.markdown-view strong) { color: #fff; }

  button {
    padding: 8px 16px;
    border-radius: 4px;
    border: none;
    cursor: pointer;
    font-weight: 600;
    transition: all 0.2s;
  }

  .btn-edit { background: #333; color: #fff; }
  .btn-edit:hover { background: #444; }

  .btn-save { background: #4caf50; color: white; }
  .btn-save:hover { background: #45a049; }

  .btn-cancel { background: transparent; color: #888; margin-right: 8px; }
  .btn-cancel:hover { color: #fff; }
</style>
