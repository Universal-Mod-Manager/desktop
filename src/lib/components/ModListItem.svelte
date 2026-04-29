<script lang="ts">
    import { Switch } from "bits-ui";
    import type { ModInfo } from "$lib/bindings";
    import type { Action } from "svelte/action";

    type DragRowAction = Action<HTMLElement, string>;

    let {
        mod,
        index,
        total,
        onToggle,
        onReorder,
        dragRow,
        onDragStart,
        dragStyle,
        isDragging = false,
    }: {
        mod: ModInfo;
        index: number;
        total: number;
        onToggle: (enabled: boolean) => void;
        onReorder: (newIndex: number) => void;
        dragRow: DragRowAction;
        onDragStart: (event: PointerEvent) => void;
        dragStyle?: string;
        isDragging?: boolean;
    } = $props();

    let inputValue = $state("");

    $effect(() => {
        inputValue = String(index + 1);
    });

    function commitReorder() {
        const newPos = parseInt(inputValue, 10);
        if (isNaN(newPos) || newPos < 1 || newPos > total || newPos === index + 1) {
            inputValue = String(index + 1);
            return;
        }
        onReorder(newPos - 1);
    }
</script>

<div
    class="umm-mod-list-item"
    data-disabled={!mod.enabled || undefined}
    data-dragging={isDragging || undefined}
    role="listitem"
    style={dragStyle}
    use:dragRow={mod.id}
>
    <button
        class="umm-mod-list-item-drag-handle"
        type="button"
        aria-label={`Drag ${mod.name} to reorder`}
        aria-pressed={isDragging}
        onpointerdown={onDragStart}
    >
        <span class="umm-mod-list-item-drag-handle-icon" aria-hidden="true"></span>
    </button>
    <input
        class="umm-mod-list-item-priority"
        type="number"
        min="1"
        max={total}
        bind:value={inputValue}
        onblur={commitReorder}
        onkeydown={(e) => {
            if (e.key === "Enter") {
                e.currentTarget.blur();
            }
        }}
    />
    <div class="umm-mod-list-item-info">
        <div class="umm-mod-list-item-name">{mod.name}</div>
        <div class="umm-mod-list-item-description">{mod.description}</div>
    </div>
    <span class="umm-mod-list-item-version">{mod.version}</span>
    <div class="umm-mod-list-item-toggle">
        <Switch.Root
            class="umm-toggle"
            checked={mod.enabled}
            onCheckedChange={(checked) => onToggle(checked)}
        >
            <Switch.Thumb class="umm-toggle-thumb" />
        </Switch.Root>
    </div>
</div>
