<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import {
    topicEnqueue,
    topicGetStatus,
    topicHistory,
    topicPreviewFeed,
    topicSet,
    topicStop,
    type TopicFeedItem,
    type TopicFeedPreview,
  } from "./api";

  let topic = "";
  let interval = 300; // 5 minutes
  let status: any = null;
  let timer: any;
  let history: Array<[string, number]> = [];
  let feedName = "";
  let feedUrl = "";
  let feedPreview: TopicFeedPreview | null = null;
  let feedLoading = false;
  let feedError = "";

  type FeedSource = {
    id: string;
    name: string;
    url: string;
  };

  const FEED_STORAGE_KEY = "tcod-topic-feeds";
  const defaultFeeds: FeedSource[] = [
    { id: "bbc-world", name: "World News", url: "https://feeds.bbci.co.uk/news/world/rss.xml" },
    { id: "hn-frontpage", name: "Tech Frontpage", url: "https://hnrss.org/frontpage" },
    { id: "hn-ai", name: "AI News", url: "https://hnrss.org/newest?q=AI" },
  ];
  let feedSources: FeedSource[] = [];

  function persistFeeds() {
    localStorage.setItem(FEED_STORAGE_KEY, JSON.stringify(feedSources));
  }

  function loadStoredFeeds() {
    const raw = localStorage.getItem(FEED_STORAGE_KEY);
    if (!raw) {
      feedSources = defaultFeeds;
      persistFeeds();
      return;
    }

    try {
      const parsed = JSON.parse(raw);
      if (Array.isArray(parsed) && parsed.length > 0) {
        feedSources = parsed;
      } else {
        feedSources = defaultFeeds;
        persistFeeds();
      }
    } catch {
      feedSources = defaultFeeds;
      persistFeeds();
    }
  }

  function selectFeed(source: FeedSource) {
    feedName = source.name;
    feedUrl = source.url;
  }

  function buildTopicLabel(sourceName: string, title: string): string {
    const prefix = sourceName.trim() ? `${sourceName.trim()}: ` : "";
    const maxLength = 100;
    const available = Math.max(12, maxLength - prefix.length);
    const trimmedTitle = title.trim();

    if ((prefix + trimmedTitle).length <= maxLength) {
      return `${prefix}${trimmedTitle}`;
    }

    return `${prefix}${trimmedTitle.slice(0, available - 1).trim()}…`;
  }

  async function previewFeed() {
    if (!feedUrl.trim()) return;

    try {
      feedLoading = true;
      feedError = "";
      feedPreview = await topicPreviewFeed(feedUrl.trim(), 8, feedName.trim() || undefined);
      if (!feedName.trim()) {
        feedName = feedPreview.source_name;
      }
    } catch (e) {
      feedError = `Failed to preview feed: ${e}`;
      feedPreview = null;
    } finally {
      feedLoading = false;
    }
  }

  async function addFeedSource() {
    if (!feedUrl.trim()) return;

    const source: FeedSource = {
      id: crypto.randomUUID(),
      name: feedName.trim() || "Custom Feed",
      url: feedUrl.trim(),
    };

    const existingIndex = feedSources.findIndex((item) => item.url === source.url);
    if (existingIndex >= 0) {
      feedSources[existingIndex] = source;
      feedSources = [...feedSources];
    } else {
      feedSources = [source, ...feedSources];
    }
    persistFeeds();
  }

  function removeFeedSource(sourceId: string) {
    feedSources = feedSources.filter((source) => source.id !== sourceId);
    persistFeeds();
  }

  async function queueFeedItem(item: TopicFeedItem) {
    const sourceName = feedPreview?.source_name || feedName || "Feed";
    const topicLabel = buildTopicLabel(sourceName, item.title);
    status = await topicEnqueue(topicLabel);
    await updateStatus();
  }

  async function startFeedItem(item: TopicFeedItem) {
    const sourceName = feedPreview?.source_name || feedName || "Feed";
    const topicLabel = buildTopicLabel(sourceName, item.title);
    status = await topicSet(topicLabel, parseInt(interval.toString(), 10));
    topic = topicLabel;
    await loadHistory();
  }

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
    loadStoredFeeds();
    if (feedSources.length > 0) {
      selectFeed(feedSources[0]);
    }
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

        {#if status.topic_queue_length && status.topic_queue_length > 0}
          <div class="mt-2">
            <div class="text-xs text-gray-400 font-bold mb-1">Queued topics ({status.topic_queue_length})</div>
            <ul class="text-xs text-gray-500 space-y-1">
              {#each status.queued_topics ?? [] as t}
                <li>{t}</li>
              {/each}
            </ul>
          </div>
        {/if}
      {/if}
    </div>
  {/if}

  <div class="feed-panel">
    <div class="feed-panel-header">
      <h4>📰 Feed Sources</h4>
      <span>Preview headlines and push them into the topic queue</span>
    </div>

    <div class="feed-form">
      <input
        type="text"
        bind:value={feedName}
        placeholder="Feed name (e.g. World News)"
        class="feed-input"
      />
      <input
        type="url"
        bind:value={feedUrl}
        placeholder="https://example.com/rss.xml"
        class="feed-input feed-url"
      />
      <button type="button" class="feed-action secondary" on:click={previewFeed} disabled={feedLoading}>
        {#if feedLoading}Loading…{:else}Preview{/if}
      </button>
      <button type="button" class="feed-action" on:click={addFeedSource}>Save Feed</button>
    </div>

    {#if feedSources.length > 0}
      <div class="feed-source-list">
        {#each feedSources as source (source.id)}
          <div class="feed-source-card">
            <button type="button" class="feed-source-main" on:click={() => selectFeed(source)}>
              <strong>{source.name}</strong>
              <span>{source.url}</span>
            </button>
            <button type="button" class="feed-source-remove" on:click={() => removeFeedSource(source.id)}>✕</button>
          </div>
        {/each}
      </div>
    {/if}

    {#if feedError}
      <div class="feed-error">{feedError}</div>
    {/if}

    {#if feedPreview}
      <div class="feed-preview">
        <div class="feed-preview-header">
          <div>
            <strong>{feedPreview.source_name}</strong>
            <div class="feed-preview-url">{feedPreview.resolved_url}</div>
          </div>
          <button type="button" class="feed-action secondary" on:click={previewFeed}>Refresh</button>
        </div>

        {#if feedPreview.items.length === 0}
          <div class="feed-empty">No usable items found in this feed.</div>
        {:else}
          <div class="feed-items">
            {#each feedPreview.items as item}
              <div class="feed-item">
                <div class="feed-item-body">
                  <div class="feed-item-title">{item.title}</div>
                  {#if item.published}
                    <div class="feed-item-meta">{new Date(item.published).toLocaleString()}</div>
                  {/if}
                  {#if item.link}
                    <a href={item.link} target="_blank" rel="noreferrer" class="feed-item-link">Open article</a>
                  {/if}
                </div>
                <div class="feed-item-actions">
                  <button type="button" class="feed-action secondary" on:click={() => queueFeedItem(item)}>Queue</button>
                  <button type="button" class="feed-action" on:click={() => startFeedItem(item)}>Start</button>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {/if}
  </div>

  {#if history.length > 0}
    <div class="mt-4 border-t border-gray-700 pt-2">
      <h4 class="text-sm font-bold text-gray-400 mb-2">Recent Topics</h4>
      <ul class="space-y-1">
        {#each history as [hTopic, timestamp]}
          <li>
            <button
              type="button"
              class="w-full text-xs text-gray-500 flex justify-between hover:text-gray-300"
              on:click={() => (topic = hTopic)}
            >
              <span>{hTopic}</span>
              <span>{new Date(timestamp * 1000).toLocaleString()}</span>
            </button>
          </li>
        {/each}
      </ul>
    </div>
  {/if}
</div>

<style>
  .feed-panel {
    margin-top: 1rem;
    padding-top: 1rem;
    border-top: 1px solid rgba(148, 163, 184, 0.2);
  }

  .feed-panel-header h4 {
    margin: 0;
    color: #f8fafc;
  }

  .feed-panel-header span {
    display: block;
    margin-top: 0.25rem;
    font-size: 0.8rem;
    color: #94a3b8;
  }

  .feed-form {
    display: grid;
    grid-template-columns: 1fr 2fr auto auto;
    gap: 0.6rem;
    margin-top: 1rem;
  }

  .feed-input {
    background: #1f2937;
    color: #fff;
    border: 1px solid #475569;
    border-radius: 10px;
    padding: 0.75rem 0.9rem;
  }

  .feed-url {
    min-width: 0;
  }

  .feed-action {
    border: none;
    border-radius: 10px;
    padding: 0.75rem 1rem;
    background: #0ea5e9;
    color: #fff;
    font-weight: 600;
    cursor: pointer;
  }

  .feed-action.secondary {
    background: #334155;
  }

  .feed-action:disabled {
    opacity: 0.6;
    cursor: wait;
  }

  .feed-source-list {
    display: grid;
    gap: 0.5rem;
    margin-top: 1rem;
  }

  .feed-source-card {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 0.5rem;
    align-items: center;
  }

  .feed-source-main {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.2rem;
    width: 100%;
    text-align: left;
    border: 1px solid rgba(14, 165, 233, 0.2);
    background: rgba(15, 23, 42, 0.7);
    color: #e2e8f0;
    border-radius: 10px;
    padding: 0.8rem 0.9rem;
    cursor: pointer;
  }

  .feed-source-main span {
    font-size: 0.75rem;
    color: #94a3b8;
    word-break: break-all;
  }

  .feed-source-remove {
    border: none;
    border-radius: 10px;
    padding: 0.7rem 0.9rem;
    background: #7f1d1d;
    color: #fecaca;
    cursor: pointer;
  }

  .feed-error,
  .feed-empty {
    margin-top: 0.9rem;
    padding: 0.8rem 0.9rem;
    border-radius: 10px;
    background: rgba(127, 29, 29, 0.2);
    color: #fecaca;
  }

  .feed-preview {
    margin-top: 1rem;
    border: 1px solid rgba(14, 165, 233, 0.2);
    border-radius: 12px;
    padding: 1rem;
    background: rgba(15, 23, 42, 0.7);
  }

  .feed-preview-header {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    align-items: flex-start;
    margin-bottom: 0.8rem;
  }

  .feed-preview-url {
    margin-top: 0.2rem;
    font-size: 0.75rem;
    color: #94a3b8;
    word-break: break-all;
  }

  .feed-items {
    display: grid;
    gap: 0.75rem;
  }

  .feed-item {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 1rem;
    align-items: start;
    padding: 0.8rem 0;
    border-top: 1px solid rgba(148, 163, 184, 0.12);
  }

  .feed-item:first-child {
    border-top: none;
    padding-top: 0;
  }

  .feed-item-title {
    color: #f8fafc;
    font-weight: 600;
    line-height: 1.35;
  }

  .feed-item-meta {
    margin-top: 0.25rem;
    color: #94a3b8;
    font-size: 0.75rem;
  }

  .feed-item-link {
    display: inline-block;
    margin-top: 0.4rem;
    color: #7dd3fc;
    font-size: 0.8rem;
  }

  .feed-item-actions {
    display: flex;
    gap: 0.5rem;
  }

  @media (max-width: 900px) {
    .feed-form {
      grid-template-columns: 1fr;
    }

    .feed-item {
      grid-template-columns: 1fr;
    }

    .feed-item-actions,
    .feed-preview-header {
      flex-direction: column;
      align-items: stretch;
    }
  }
</style>