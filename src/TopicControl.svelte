<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { topicGetStatus, topicSet, topicStop, topicHistory } from "./api";

  let topic = "";
  let interval = 600; // 10 minutes
  let status: any = null;
  let timer: any;
  let history: Array<[string, number]> = [];

  async function updateStatus() {
    try {
      status = await topicGetStatus();
    } catch (e) {
      console.error("Failed to get topic status", e);
    }
  }

  async function loadHistory() {
    try {
      history = await topicHistory(5);
    } catch (e) {
      console.error("Failed to load history", e);
    }
  }

  async function startTopic() {
    if (!topic) return;
    try {
      // Ensure interval is a number
      const intervalNum = parseInt(interval.toString(), 10);
      status = await topicSet(topic, intervalNum);
      await loadHistory();
    } catch (e) {
      console.error("Failed to set topic", e);
      alert("Failed to set topic: " + e);
    }
  }

  async function stopTopic() {
    try {
      status = await topicStop();
    } catch (e) {
      console.error("Failed to stop topic", e);
    }
  }

  onMount(() => {
    updateStatus();
    loadHistory();
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

  {#if history.length > 0}
    <div class="mt-4 border-t border-gray-700 pt-2">
      <h4 class="text-sm font-bold text-gray-400 mb-2">Recent Topics</h4>
      <ul class="space-y-1">
        {#each history as [hTopic, timestamp]}
          <li class="text-xs text-gray-500 flex justify-between cursor-pointer hover:text-gray-300" on:click={() => topic = hTopic}>
            <span>{hTopic}</span>
            <span>{new Date(timestamp * 1000).toLocaleString()}</span>
          </li>
        {/each}
      </ul>
    </div>
  {/if}
</div>