<script lang="ts">
    import { appState } from "$lib/stores/app.svelte";
    import ModListItem from "./ModListItem.svelte";

    function handleReorder(fromIndex: number, toIndex: number) {
        const ids = [...appState.mods.map((m) => m.id)];
        const [removed] = ids.splice(fromIndex, 1);
        ids.splice(toIndex, 0, removed);
        appState.reorderMods(ids);
    }
</script>

<div class="umm-mod-list" role="list">
    {#each appState.mods as mod, i (mod.id)}
        <ModListItem
            {mod}
            index={i}
            total={appState.mods.length}
            onToggle={(enabled) => appState.toggleMod(mod.id, enabled)}
            onReorder={(newIndex) => handleReorder(i, newIndex)}
        />
    {/each}
    {#if appState.mods.length === 0}
        <p class="umm-mod-list-empty">No mods found</p>
    {/if}
</div>
