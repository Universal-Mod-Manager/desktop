<script lang="ts">
    import { Dialog } from "bits-ui";
    import { Settings, X } from "lucide-svelte";
    import { fade, fly } from "svelte/transition";
    import Sidebar from "./Sidebar.svelte";

    let settingsDrawerOpen = $state(false);
</script>

<Dialog.Root bind:open={settingsDrawerOpen}>
    <Dialog.Trigger
        class="umm-settings-button"
        aria-label="Open settings"
        title="Open settings"
    >
        <Settings size={18} />
    </Dialog.Trigger>
    <Dialog.Portal>
        <Dialog.Overlay forceMount>
            {#snippet child({ props, open })}
                {#if open}
                    <div
                        {...props}
                        class="umm-drawer-overlay"
                        transition:fade={{ duration: 120 }}
                    ></div>
                {/if}
            {/snippet}
        </Dialog.Overlay>
        <Dialog.Content forceMount>
            {#snippet child({ props, open })}
                {#if open}
                    <div
                        {...props}
                        class="umm-drawer"
                        transition:fly={{ x: "100%", duration: 150, opacity: 1 }}
                    >
                        <Dialog.Title class="umm-visually-hidden">Settings</Dialog.Title>
                        <Dialog.Description class="umm-visually-hidden">
                            Game, path, and theme settings.
                        </Dialog.Description>
                        <Dialog.Close
                            class="umm-drawer-close"
                            aria-label="Close settings"
                            title="Close settings"
                        >
                            <X size={16} />
                        </Dialog.Close>
                        <Sidebar />
                    </div>
                {/if}
            {/snippet}
        </Dialog.Content>
    </Dialog.Portal>
</Dialog.Root>
