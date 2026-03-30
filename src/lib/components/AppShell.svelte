<script lang="ts">
    import Sidebar from "./Sidebar.svelte";
    import ModList from "./ModList.svelte";
    import { appState } from "$lib/stores/app.svelte";
</script>

<div class="umm-app">
    <Sidebar />
    <main class="umm-content">
        <header class="umm-content-header">
            <div>
                <h1 class="umm-content-title">Mods</h1>
                {#if appState.activePluginId}
                    <p class="umm-content-subtitle">
                        {appState.mods.length} mod{appState.mods.length !== 1 ? "s" : ""} loaded
                    </p>
                {/if}
            </div>
        </header>
        <div class="umm-content-body">
            {#if appState.loading}
                <p class="umm-mod-list-empty">Loading...</p>
            {:else if !appState.activePluginId}
                <p class="umm-mod-list-empty">Select a game plugin to get started</p>
            {:else}
                <ModList />
            {/if}
        </div>
    </main>
</div>
