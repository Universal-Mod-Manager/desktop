<script lang="ts">
    import { Switch } from "bits-ui";
    import type { ModInfo } from "$lib/bindings";

    let {
        mod,
        index,
        total,
        onToggle,
        onReorder,
    }: {
        mod: ModInfo;
        index: number;
        total: number;
        onToggle: (enabled: boolean) => void;
        onReorder: (newIndex: number) => void;
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
    role="listitem"
>
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
