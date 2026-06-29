<script lang="ts">
    import PluginSelector from "./PluginSelector.svelte";
    import GamePathSetting from "./GamePathSetting.svelte";
    import ThemeSelector from "./ThemeSelector.svelte";
</script>

<aside class="umm-sidebar">
    <div class="umm-sidebar-header">
        <h2 class="umm-sidebar-title">UMM</h2>
    </div>
    <div class="umm-sidebar-section">
        <p class="umm-sidebar-section-label">Game</p>
        <PluginSelector />
        <GamePathSetting />
    </div>
    <div class="umm-sidebar-footer">
        <p class="umm-sidebar-section-label">Theme</p>
        <ThemeSelector />
    </div>
</aside>
