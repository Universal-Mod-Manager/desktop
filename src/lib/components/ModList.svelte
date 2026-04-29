<script lang="ts">
    import { appState } from "$lib/stores/app.svelte";
    import { useDragReorder } from "$lib/hooks/useDragReorder.svelte";
    import ModListItem from "./ModListItem.svelte";

    const dragReorder = useDragReorder({
        onReorder: handleReorder,
    });

    $effect(() => {
        dragReorder.setItems(appState.mods.map((mod) => mod.id));
    });

    function handleReorder(fromIndex: number, toIndex: number) {
        if (fromIndex === toIndex) return;

        const ids = [...appState.mods.map((m) => m.id)];
        const [removed] = ids.splice(fromIndex, 1);
        ids.splice(toIndex, 0, removed);
        appState.reorderMods(ids);
    }
</script>

<div
    class="umm-mod-list"
    role="list"
    use:dragReorder.list
    data-dragging={dragReorder.isDragging || undefined}
    data-drop-committing={dragReorder.isDropCommitting || undefined}
>
    {#each appState.mods as mod, i (mod.id)}
        <ModListItem
            {mod}
            index={i}
            total={appState.mods.length}
            onToggle={(enabled) => appState.toggleMod(mod.id, enabled)}
            onReorder={(newIndex) => handleReorder(i, newIndex)}
            dragRow={dragReorder.row}
            onDragStart={(event: PointerEvent) => dragReorder.startDrag(mod.id, event)}
            dragStyle={dragReorder.getRowStyle(mod.id)}
            isDragging={dragReorder.isDraggingRow(mod.id)}
        />
    {/each}
    {#if appState.mods.length === 0}
        <p class="umm-mod-list-empty">No mods found</p>
    {/if}
</div>
