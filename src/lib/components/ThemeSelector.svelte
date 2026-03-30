<script lang="ts">
    import { Select } from "bits-ui";
    import { ChevronDown, Check, Palette } from "lucide-svelte";
    import { appState } from "$lib/stores/app.svelte";

    const displayName = $derived(appState.activeThemeName || "Default");
</script>

<Select.Root
    type="single"
    value={appState.activeThemeName || "__default__"}
    onOpenChange={(open) => open && appState.refreshThemes()}
    onValueChange={(v) => v && appState.setTheme(v === "__default__" ? "" : v)}
>
    <Select.Trigger class="umm-select-trigger">
        <span class="umm-select-trigger-decoration">
            <Palette size={14} />
        </span>
        <span class="umm-select-trigger-text">{displayName}</span>
        <span class="umm-select-trigger-chevron">
            <ChevronDown size={14} />
        </span>
    </Select.Trigger>
    <Select.Content class="umm-select-content" sideOffset={4}>
        <Select.Viewport class="umm-select-viewport">
            <Select.Item
                class="umm-select-item"
                value="__default__"
                label="Default"
            >
                {#snippet children({ selected })}
                    <span>Default</span>
                    {#if selected}
                        <span class="umm-select-item-check">
                            <Check size={14} />
                        </span>
                    {/if}
                {/snippet}
            </Select.Item>
            {#each appState.themes as theme (theme.name)}
                <Select.Item
                    class="umm-select-item"
                    value={theme.name}
                    label={theme.name}
                >
                    {#snippet children({ selected })}
                        <span>{theme.name}</span>
                        {#if selected}
                            <span class="umm-select-item-check">
                                <Check size={14} />
                            </span>
                        {/if}
                    {/snippet}
                </Select.Item>
            {/each}
        </Select.Viewport>
    </Select.Content>
</Select.Root>
