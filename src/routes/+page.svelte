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

    $effect(() => {
      listen<PublishEvent>("mnemnk-publish", (event) => {
        // console.log("Received event", event);
        events.push(event.payload);
        // console.log("Events", events);
      }).then((unlistenFn) => {
        unlisten = unlistenFn;
      });
      return () => {
        unlisten?.();
      }
    });
</script>

<main class="container">
  {#each events as event}
    <Card>
      <h2>{event.agent}</h2>
      <p>{event.channel}</p>
      <p>{event.value}</p>
      <p>{new Date(event.time)}</p>
    </Card>
  {/each}
</main>
