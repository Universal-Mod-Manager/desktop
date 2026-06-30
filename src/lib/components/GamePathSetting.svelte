<script lang="ts">
    import { FolderOpen } from "lucide-svelte";
    import { appState } from "$lib/stores/app.svelte";

    const activePluginId = $derived(appState.activePluginId);
    const pathRoots = $derived(
        activePluginId ? (appState.pluginPathRoots[activePluginId] ?? []) : [],
    );

    function displayPath(path: string | undefined): string {
        if (!path) return "No path set";
        return path.length > 30 ? "..." + path.slice(-30) : path;
    }
</script>

{#if activePluginId && pathRoots.length > 0}
    <div class="umm-game-path">
        {#each pathRoots as pathRoot (pathRoot.id)}
            {@const currentPath = appState.pluginPaths[activePluginId]?.[pathRoot.id]}
            <div class="umm-game-path-item">
                <span class="umm-game-path-label">{pathRoot.name}</span>
                <button
                    class="umm-game-path-btn"
                    onclick={() => appState.browsePluginPath(activePluginId, pathRoot.id)}
                    title={currentPath ?? pathRoot.description}
                >
                    <span class="umm-game-path-icon">
                        <FolderOpen size={14} />
                    </span>
                    <span class="umm-game-path-text" class:umm-game-path-text--empty={!currentPath}>
                        {displayPath(currentPath)}
                    </span>
                </button>
                {#if pathRoot.description}
                    <span class="umm-game-path-description">{pathRoot.description}</span>
                {/if}
            </div>
        {/each}
    </div>
{/if}
