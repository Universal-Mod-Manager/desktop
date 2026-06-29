<script lang="ts">
    import { Select } from "bits-ui";
    import { ChevronDown, Check } from "lucide-svelte";
    import { appState } from "$lib/stores/app.svelte";

    const selectedLabel = $derived(
        appState.plugins.find((p) => p.id === appState.activePluginId)?.name ??
            "Select a game...",
    );
</script>

<Select.Root
    type="single"
    value={appState.activePluginId ?? undefined}
    onValueChange={(v) => v && appState.selectPlugin(v)}
>
    <Select.Trigger class="umm-select-trigger">
        <span class="umm-select-trigger-text">{selectedLabel}</span>
        <span class="umm-select-trigger-chevron">
            <ChevronDown size={14} />
        </span>
    </Select.Trigger>
    <Select.Content class="umm-select-content" sideOffset={4}>
        {#each appState.plugins as plugin (plugin.id)}
            <Select.Item
                class="umm-select-item"
                value={plugin.id}
                label={plugin.name}
            >
                {#snippet children({ selected })}
                    <span>{plugin.name}</span>
                    {#if selected}
                        <span class="umm-select-item-check">
                            <Check size={14} />
                        </span>
                    {/if}
                {/snippet}
            </Select.Item>
        {/each}
    </Select.Content>
</Select.Root>
