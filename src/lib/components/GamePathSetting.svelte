<script lang="ts">
    import { FolderOpen } from "lucide-svelte";
    import { appState } from "$lib/stores/app.svelte";

    const currentPath = $derived(
        appState.activePluginId
            ? appState.gamePaths[appState.activePluginId]
            : undefined,
    );

    const displayPath = $derived(
        currentPath
            ? currentPath.length > 30
                ? "..." + currentPath.slice(-30)
                : currentPath
            : "No path set",
    );
</script>

{#if appState.activePluginId}
    <div class="umm-game-path">
        <button
            class="umm-game-path-btn"
            onclick={() =>
                appState.activePluginId &&
                appState.browseGamePath(appState.activePluginId)}
            title={currentPath ?? "Click to set game directory"}
        >
            <span class="umm-game-path-icon">
                <FolderOpen size={14} />
            </span>
            <span class="umm-game-path-text" class:umm-game-path-text--empty={!currentPath}>
                {displayPath}
            </span>
        </button>
    </div>
{/if}
