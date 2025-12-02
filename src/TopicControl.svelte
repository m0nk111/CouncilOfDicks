<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";

  let topic = "";
  let interval = 300; // 5 minutes
  let status: any = null;
  let timer: any;

  async function updateStatus() {
    try {
      status = await invoke("topic_get_status");
    } catch (e) {
      console.error("Failed to get topic status", e);
    }
  }

  async function startTopic() {
    if (!topic) return;
    try {
      status = await invoke("topic_set", { topic, interval });
    } catch (e) {
      console.error("Failed to set topic", e);
    }
  }

  async function stopTopic() {
    try {
      status = await invoke("topic_stop");
    } catch (e) {
      console.error("Failed to stop topic", e);
    }
  }

  onMount(() => {
    updateStatus();
    timer = setInterval(updateStatus, 5000);
  });

  onDestroy(() => {
    if (timer) clearInterval(timer);
  });
</script>

<div class="topic-control p-4 bg-gray-800 rounded-lg mb-4">
  <h3 class="text-lg font-bold mb-2">📢 Topic Channel Control</h3>
  
  <div class="flex gap-2 mb-4">
    <input 
      type="text" 
      bind:value={topic} 
      placeholder="Enter topic (e.g. #topic Future of AI)"
      class="flex-1 p-2 rounded bg-gray-700 text-white border border-gray-600"
    />
    <input 
      type="number" 
      bind:value={interval} 
      min="60"
      class="w-24 p-2 rounded bg-gray-700 text-white border border-gray-600"
      title="Interval in seconds"
    />
    <button 
      on:click={startTopic}
      class="px-4 py-2 bg-green-600 hover:bg-green-500 rounded font-bold"
    >
      Start
    </button>
    <button 
      on:click={stopTopic}
      class="px-4 py-2 bg-red-600 hover:bg-red-500 rounded font-bold"
    >
      Stop
    </button>
  </div>

  {#if status}
    <div class="text-sm text-gray-300">
      <div class="flex justify-between items-center">
        <span>Status: <span class={status.is_running ? "text-green-400" : "text-gray-500"}>{status.is_running ? "ACTIVE" : "IDLE"}</span></span>
        {#if status.is_running}
          <span>Current Topic: <span class="font-mono text-yellow-400">{status.current_topic}</span></span>
        {/if}
      </div>
      {#if status.is_running}
        <div class="mt-2 flex gap-4">
          <span>Queue: {status.queue_length} agents</span>
          <span>Next message in: {status.next_run_in_secs}s</span>
        </div>
      {/if}
    </div>
  {/if}
</div>