<script lang="ts">
    import { Card } from 'flowbite-svelte';
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";

    type PublishEvent = {
      agent: string;
      channel: string;
      value: any;
      time: number;
    };

    let unlisten: UnlistenFn | null = null;
    let events = $state<PublishEvent[]>([]);
    let height = $state(0);

    const MAX_EVENTS = 100;

    $effect(() => {
      listen<PublishEvent>("mnemnk-publish", (event) => {
        events.push(event.payload);
        if (events.length > MAX_EVENTS) {
          events = events.slice(events.length - MAX_EVENTS);
        }
      }).then((unlistenFn) => {
        unlisten = unlistenFn;
      });
      return () => {
        unlisten?.();
      }
    });

    $effect(() => {
      window.scrollTo(0, height);
    })

    function formatTime(timestamp: number): string {
      const date = new Date(timestamp);
      return date.toLocaleTimeString('en-US', { hour12: false });
    }
</script>

<main class="container m-4" bind:clientHeight={height}>
  <div class="grid grid-column grid-cols-12 gap-4">
    {#each events as event}
      <div class=""></div>
      <div class="col-span-10 border border-gray-200 p-4 rounded-lg drop-shadow-sm">
        <div class="drop-shadow-none">
          <h3>
            <span class="mr-4 mb-2">{formatTime(event.time)}</span>
            <span class="font-bold">{event.channel} ({event.agent})</span>
          </h3>
        </div>
        <p>{JSON.stringify(event.value, null, 2)}</p>
      </div>
      <div class=""></div>
    {/each}
  </div>
</main>
