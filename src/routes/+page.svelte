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

    $effect(() => {
      listen<PublishEvent>("mnemnk-publish", (event) => {
        events.push(event.payload);
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
</script>

<main class="container" bind:clientHeight={height}>
  {#each events as event}
    <Card>
      <h2>{event.agent}</h2>
      <p>{event.channel}</p>
      <p>{JSON.stringify(event.value, null, 2)}</p>
      <p>{new Date(event.time)}</p>
    </Card>
  {/each}
</main>
