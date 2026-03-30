import { commands, type ModInfo, type PluginInfo, type ThemeInfo, type Result } from "$lib/bindings";

function unwrap<T>(result: Result<T, string>): T {
    if (result.status === "ok") return result.data;
    throw new Error(result.error);
}

class AppState {
    plugins = $state<PluginInfo[]>([]);
    activePluginId = $state<string | null>(null);
    mods = $state<ModInfo[]>([]);
    themes = $state<ThemeInfo[]>([]);
    activeThemeName = $state("");
    themeCss = $state("");
    loading = $state(true);

    async initialize() {
        try {
            this.plugins = unwrap(await commands.listPlugins());
            this.themes = unwrap(await commands.listThemes());
            this.activeThemeName = unwrap(await commands.getActiveTheme());
            if (this.activeThemeName) {
                this.themeCss = unwrap(await commands.getThemeCss(this.activeThemeName));
            }

            const active = unwrap(await commands.getActivePlugin());
            if (active) {
                this.activePluginId = active;
                this.mods = unwrap(await commands.listMods());
            }
        } finally {
            this.loading = false;
        }
    }

    async selectPlugin(pluginId: string) {
        this.mods = unwrap(await commands.selectPlugin(pluginId));
        this.activePluginId = pluginId;
    }

    async toggleMod(modId: string, enabled: boolean) {
        unwrap(await commands.toggleMod(modId, enabled));
        this.mods = this.mods.map((m) =>
            m.id === modId ? { ...m, enabled } : m,
        );
    }

    async reorderMods(modIds: string[]) {
        unwrap(await commands.reorderMods(modIds));
        this.mods = modIds
            .map((id, i) => {
                const mod = this.mods.find((m) => m.id === id);
                return mod ? { ...mod, priority: i } : null;
            })
            .filter((m): m is ModInfo => m !== null);
    }

    async refreshThemes() {
        this.themes = unwrap(await commands.listThemes());
    }

    async setTheme(themeName: string) {
        this.themeCss = unwrap(await commands.setActiveTheme(themeName));
        this.activeThemeName = themeName;
        this.themes = this.themes.map((t) => ({
            ...t,
            is_active: t.name === themeName,
        }));
    }
}

export const appState = new AppState();
