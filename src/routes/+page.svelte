<script lang="ts">
    import { Button, DarkMode, Input } from "flowbite-svelte";
    import { ArrowDownOutline, ArrowUpDownOutline } from "flowbite-svelte-icons";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import { invoke } from "@tauri-apps/api/core";

    type InputEvent = {
      kind: string;
      value: any;
      time: number;
    };

    let unlisten: UnlistenFn | null = null;
    let events = $state<InputEvent[]>([]);
    let height = $state(0);
    let message = $state("");
    let keepScrolled = $state(true);

    const MAX_EVENTS = 100;

    $effect(() => {
      listen<InputEvent>("mnemnk-input", (event) => {
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
      if (keepScrolled) {
        window.scrollTo(0, height);
      }
    })

    function formatTime(timestamp: number): string {
      const date = new Date(timestamp);
      return date.toLocaleTimeString('en-US', { hour12: false });
    }

    async function onsubmit(event: Event) {
      event.preventDefault();
      events.push({
        kind: "user_message",
        value: { "message": message}, 
        time: Date.now(),
      });
      if (message) {
        await invoke("send_message", { message });
      }
      message = "";
    }

    function toggleKeepScrolled() {
      keepScrolled = !keepScrolled;
    }
</script>

<div class="container relative flex flex-col min-h-[100vh] max-w-full p-4 bg-white dark:bg-gray-900">
  <div class="fixed top-0 left-0 right-0 z-50 flex gap-1 bg-transparent">
    <div class="grow">&nbsp;</div>
    <Button size="xs" onclick={toggleKeepScrolled} color="light" class="border-none opacity-50">
      {#if keepScrolled}
        <ArrowDownOutline size="sm" />
      {:else}
        <ArrowUpDownOutline size="sm" />
      {/if}
    </Button>
    <DarkMode size="sm" />
  </div>
  <div class="grow" bind:clientHeight={height}>
    {#each events as event}
      <div class="border border-gray-200 m-2 p-2 rounded-lg drop-shadow-sm">
        <div class="text-gray-700 dark:text-gray-300 drop-shadow-none">
          <h3>
            <span class="mr-4 mb-2">{formatTime(event.time)}</span>
            <span class="font-bold">{event.kind}</span>
          </h3>
        </div>
        <p>{JSON.stringify(event.value, null, 2)}</p>
      </div>
    {/each}
  </div>
  <div class="ml-2 mr-2">
    <form {onsubmit}>
      <Input placeholder="message..." bind:value={message} />
    </form>
  </div>
</div>
