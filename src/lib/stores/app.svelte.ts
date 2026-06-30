import { commands, type GamePathRoot, type ModInfo, type PluginInfo, type ThemeInfo, type Result } from "$lib/bindings";
import { open } from "@tauri-apps/plugin-dialog";

function unwrap<T>(result: Result<T, string>): T {
    if (result.status === "ok") return result.data;
    throw new Error(result.error);
}

function compactPluginPaths(
    record: Partial<Record<string, Partial<Record<string, string>>>>,
): Record<string, Record<string, string>> {
    const pluginEntries = Object.entries(record).map(([pluginId, rootPaths]) => {
        const pathEntries = Object.entries(rootPaths ?? {}).filter(
            (entry): entry is [string, string] => typeof entry[1] === "string",
        );

        return [pluginId, Object.fromEntries(pathEntries)] as const;
    });

    return Object.fromEntries(pluginEntries);
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
    pluginPaths = $state<Record<string, Record<string, string>>>({});
    pluginPathRoots = $state<Record<string, GamePathRoot[]>>({});
    loading = $state(true);
    modLoadError = $state<string | null>(null);

    async initialize() {
        try {
            this.plugins = unwrap(await commands.listPlugins());
            this.themes = unwrap(await commands.listThemes());
            this.pluginPaths = compactPluginPaths(unwrap(await commands.getPluginPaths()));
            this.activeThemeName = unwrap(await commands.getActiveTheme());
            if (this.activeThemeName) {
                this.themeCss = unwrap(await commands.getThemeCss(this.activeThemeName));
            }

            const active = unwrap(await commands.getActivePlugin());
            if (active) {
                this.activePluginId = active;
                try {
                    await this.loadPluginPathRoots(active);
                    if (this.hasConfiguredPluginPaths(active)) {
                        this.mods = unwrap(await commands.listMods());
                    }
                } catch (error) {
                    this.mods = [];
                    this.modLoadError = error instanceof Error ? error.message : String(error);
                }
            }
        } finally {
            this.loading = false;
        }
    }

    async selectPlugin(pluginId: string) {
        this.activePluginId = pluginId;
        this.modLoadError = null;

        try {
            await this.loadPluginPathRoots(pluginId);
        } catch (error) {
            this.mods = [];
            this.modLoadError = error instanceof Error ? error.message : String(error);
            return;
        }

        if (!this.hasConfiguredPluginPaths(pluginId)) {
            this.mods = [];
            return;
        }

        await this.loadSelectedPluginMods(pluginId);
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

    async browsePluginPath(pluginId: string, rootId: string) {
        const roots = await this.loadPluginPathRoots(pluginId);
        const root = roots.find((pathRoot) => pathRoot.id === rootId);
        const selected = await open({
            directory: true,
            multiple: false,
            title: `Select ${root?.name ?? "plugin path"}`,
        });
        if (!selected || Array.isArray(selected)) return;

        this.modLoadError = null;
        unwrap(await commands.setPluginPath(pluginId, rootId, selected));
        this.pluginPaths = {
            ...this.pluginPaths,
            [pluginId]: {
                ...(this.pluginPaths[pluginId] ?? {}),
                [rootId]: selected,
            },
        };

        if (this.activePluginId === pluginId) {
            if (this.hasConfiguredPluginPaths(pluginId)) {
                await this.loadSelectedPluginMods(pluginId);
            } else {
                this.mods = [];
            }
        }
    }

    async loadPluginPathRoots(pluginId: string): Promise<GamePathRoot[]> {
        const cached = this.pluginPathRoots[pluginId];
        if (cached) return cached;

        const roots = unwrap(await commands.getPluginPathRoots(pluginId));
        this.pluginPathRoots = { ...this.pluginPathRoots, [pluginId]: roots };
        return roots;
    }

    hasConfiguredPluginPaths(pluginId: string): boolean {
        const roots = this.pluginPathRoots[pluginId] ?? [];
        const paths = this.pluginPaths[pluginId] ?? {};

        return roots.length > 0 && roots.every((root) => !!paths[root.id]);
    }


    async loadSelectedPluginMods(pluginId: string) {
        try {
            this.mods = unwrap(await commands.selectPlugin(pluginId));
        } catch (error) {
            this.mods = [];
            this.modLoadError = error instanceof Error ? error.message : String(error);
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
