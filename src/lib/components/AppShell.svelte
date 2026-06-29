<script lang="ts">
    import SettingsDrawer from "./SettingsDrawer.svelte";
    import ModList from "./ModList.svelte";
    import { appState } from "$lib/stores/app.svelte";

    const hasGamePath = $derived(
        appState.activePluginId
            ? !!appState.gamePaths[appState.activePluginId]
            : false,
    );

    const activePlugin = $derived(
        appState.plugins.find((plugin) => plugin.id === appState.activePluginId),
    );
    const contentTitle = $derived(activePlugin ? `${activePlugin.name} Mods` : "Mods");
</script>

<div class="umm-app">
    <main class="umm-content">
        <header class="umm-content-header">
            <div>
                <h1 class="umm-content-title">{contentTitle}</h1>
                {#if appState.activePluginId && hasGamePath}
                    <p class="umm-content-subtitle">
                        {appState.mods.length} mod{appState.mods.length !== 1 ? "s" : ""} loaded
                    </p>
                {/if}
            </div>
            <SettingsDrawer />
        </header>
        <div class="umm-content-body">
            {#if appState.loading}
                <p class="umm-mod-list-empty">Loading...</p>
            {:else if !appState.activePluginId}
                <p class="umm-mod-list-empty">Select a game plugin to get started</p>
            {:else if !hasGamePath}
                <p class="umm-mod-list-empty">
                    Open settings to set the game directory and load mods
                </p>
            {:else}
                <ModList />
            {/if}
        </div>
    </main>
</div>
