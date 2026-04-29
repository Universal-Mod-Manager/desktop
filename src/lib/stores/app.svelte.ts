import { commands, type ModInfo, type PluginInfo, type ThemeInfo, type Result } from "$lib/bindings";
import { open } from "@tauri-apps/plugin-dialog";

function unwrap<T>(result: Result<T, string>): T {
    if (result.status === "ok") return result.data;
    throw new Error(result.error);
}

function compactStringRecord(record: Partial<Record<string, string>>): Record<string, string> {
    const entries = Object.entries(record).filter(
        (entry): entry is [string, string] => typeof entry[1] === "string",
    );

    return Object.fromEntries(entries);
}

function reorderModList(mods: ModInfo[], modIds: string[]): ModInfo[] {
    return modIds
        .map((id, i) => {
            const mod = mods.find((m) => m.id === id);
            return mod ? { ...mod, priority: i } : null;
        })
        .filter((m): m is ModInfo => m !== null);
}

class AppState {
    plugins = $state<PluginInfo[]>([]);
    activePluginId = $state<string | null>(null);
    mods = $state<ModInfo[]>([]);
    themes = $state<ThemeInfo[]>([]);
    activeThemeName = $state("");
    themeCss = $state("");
    gamePaths = $state<Record<string, string>>({});
    loading = $state(true);

    async initialize() {
        try {
            this.plugins = unwrap(await commands.listPlugins());
            this.themes = unwrap(await commands.listThemes());
            this.gamePaths = compactStringRecord(unwrap(await commands.getGamePaths()));
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
        this.activePluginId = pluginId;

        if (!this.gamePaths[pluginId]) {
            this.mods = [];
            return;
        }

        this.mods = unwrap(await commands.selectPlugin(pluginId));
    }

    async toggleMod(modId: string, enabled: boolean) {
        unwrap(await commands.toggleMod(modId, enabled));
        this.mods = this.mods.map((m) =>
            m.id === modId ? { ...m, enabled } : m,
        );
    }

    async reorderMods(modIds: string[]) {
        const previousMods = this.mods;
        this.mods = reorderModList(previousMods, modIds);

        try {
            unwrap(await commands.reorderMods(modIds));
        } catch (error) {
            this.mods = previousMods;
            throw error;
        }
    }

    async browseGamePath(pluginId: string) {
        const selected = await open({
            directory: true,
            multiple: false,
            title: "Select game directory",
        });
        if (!selected) return;

        unwrap(await commands.setGamePath(pluginId, selected));
        this.gamePaths = { ...this.gamePaths, [pluginId]: selected };

        if (this.activePluginId === pluginId) {
            this.mods = unwrap(await commands.selectPlugin(pluginId));
        }
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
