<script lang="ts">
  import { onMount, onDestroy } from "svelte";

  export let text: string = "";
  export let speed: number = 15; // Slightly faster
  export let timestamp: string = "";

  let displayedText = "";
  let isAnimating = false;
  let interval: any;

  const ANIMATION_THRESHOLD_MS = 30000;

  function shouldAnimate(): boolean {
    if (!timestamp) return true;
    const msgTime = new Date(timestamp).getTime();
    const now = Date.now();
    return (now - msgTime) < ANIMATION_THRESHOLD_MS;
  }

  function startAnimation() {
    if (interval) clearInterval(interval);
    
    isAnimating = true;
    displayedText = "";
    let i = 0;

    interval = setInterval(() => {
      if (i < text.length) {
        displayedText += text[i];
        i++;
      } else {
        stopAnimation();
      }
    }, speed);
  }

  function stopAnimation() {
    if (interval) clearInterval(interval);
    interval = null;
    isAnimating = false;
    displayedText = text;
  }

  // Reactive logic to handle text updates and initialization
  $: if (text !== displayedText) {
    // If we are currently animating, don't interrupt unless text completely changed
    if (!isAnimating) {
      if (displayedText === "" && shouldAnimate()) {
        startAnimation();
      } else {
        displayedText = text;
      }
    }
  }

  onDestroy(() => {
    if (interval) clearInterval(interval);
  });
</script>

<div class="typewriter">
  {displayedText}
  {#if isAnimating}
    <span class="cursor">▍</span>
  {/if}
</div>

<style>
  .typewriter {
    white-space: pre-wrap;
    word-break: break-word;
    font-size: 0.85em;
    line-height: 1.4;
  }
  
  .cursor {
    display: inline-block;
    width: 0.5em;
    animation: blink 1s step-end infinite;
    color: #00d4ff;
    vertical-align: baseline;
  }

  @keyframes blink {
    0%, 100% { opacity: 1; }
    50% { opacity: 0; }
  }
</style>