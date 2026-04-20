# Universal Mod Manager: User Stories
**Duration:** 2 years  
**Goal:** Multi-platform mod manager with plugin system for multiple games
---
## Phase 1: Foundation
**US 1.1: Config manager**
- As a user, I want my settings and profiles saved automatically, so I don't lose them on restart.

**Acceptance Criteria:**
- [ ] Load/save app settings (theme, last used profile) to TOML
- [ ] Load/save mod load order and enabled state to JSON
- [ ] Config files stored in OS-appropriate app data directory (using `dirs` crate)
- [ ] Graceful handling of missing or malformed config files
- [ ] Exposed to frontend via Tauri commands
---
**US 1.2: Game discovery**
- As a user, I want the app to find my installed games automatically, so I don't search manually.

**Acceptance Criteria:**
- [ ] Detect Steam library folders from `libraryfolders.vdf` on Windows and Linux
- [ ] Return default game install candidates per platform
- [ ] Works on SteamOS / Steam Deck (Linux)
- [ ] Unit-testable with mocked paths
---
## Phase 2: Core Logic
**US 2.1: Plugin API**
- As a plugin developer, I want a clear GamePlugin trait, so I can create game-specific plugins.

**Acceptance Criteria:**
- [ ] `GamePlugin` trait defined with methods: `id()`, `display_name()`, `detect_installation()`, `get_mod_directory()`, `deploy_mod()`, `undeploy_mod()`
- [ ] Trait is documented with rustdoc
- [ ] A `MockPlugin` implementation exists for testing
- [ ] Plugin registration mechanism in place (static registry for Phase 2, dynamic loading later)
---
**US 2.2: Mod ingestion**
- As a user, I want to import mods from ZIP/7z or folders, so they're organized in my library.

**Acceptance Criteria:**
- [ ] Support importing `.zip` and `.7z` archives
- [ ] Support importing a folder directly
- [ ] Extract to a sandboxed staging directory under app data
- [ ] Generate and store mod metadata (name, version if detectable, source path, enabled state)
- [ ] Exposed as Tauri commands: `ingest_mod_archive`, `ingest_mod_folder`
---
**US 2.3: Load order management**
- As a user, I want to reorder mods and have it saved, so they load in the right order.

**Acceptance Criteria:**
- [ ] Load order stored as an ordered list of mod IDs in the profile JSON
- [ ] Tauri commands: `reorder_mods`, `get_load_order`
- [ ] Order changes are immediately persisted
- [ ] Unit tests for order manipulation
---
**US 2.4: Profile management**
- As a user, I want multiple profiles with different mods, so I can switch between setups easily.

**Acceptance Criteria:**
- [ ] Create / rename / delete profiles
- [ ] Each profile stores its own load order and enabled mods
- [ ] Active profile is persisted across app restarts
- [ ] Profile switcher UI component (dropdown or sidebar)
- [ ] Tauri commands: `create_profile`, `delete_profile`, `switch_profile`, `list_profiles`
---
**US 2.5: Mod list UI**
- As a user, I want to see my mods in a list with toggles and drag-drop, so I can manage them visually.

**Acceptance Criteria:**
- [ ] Displays mod name, enabled/disabled status, and load order index
- [ ] Toggle switch to enable/disable a mod
- [ ] Drag-and-drop reordering (updates load order)
- [ ] Connects to Tauri backend via `invoke` calls
- [ ] Empty state shown when no mods are installed
---
## Phase 3: First Game
**US 3.1: Game plugin #1**
- As a user, I want to manage mods for one game, so I can install, enable, disable, and reorder them.

**Acceptance Criteria:**
- [ ] Plugin implements full `GamePlugin` trait
- [ ] Auto-detects installation via Steam and manual path override
- [ ] Correctly deploys and undeploys mods
- [ ] Tested on Windows and Linux
---
**US 3.2: Deployment engine**
- As a user, I want mods deployed to my game folder automatically, so I don't copy files manually.

**Acceptance Criteria:**
- [ ] Deploy mods via symlinks; fallback to hard links if symlinks unavailable
- [ ] Conflict detection: warn when two mods provide the same file
- [ ] Undeploy cleanly removes all links/copies without touching originals
- [ ] Works on Windows (requires Developer Mode or admin for symlinks) and Linux
---
**US 3.3: Linux / Steam Deck support**
- As a Linux/Steam Deck user, I want the app to work on my platform, so I can manage mods there.

**Acceptance Criteria:**
- [ ] App installs and launches on Ubuntu 22.04 and SteamOS 3
- [ ] File operations and symlinks work correctly
- [ ] Steam library detection works on Linux
- [ ] Any platform-specific bugs documented and fixed
---
## Phase 4: Expansion
**US 4.1: Game plugin #2**
- As a user, I want support for a second game with different modding, so the plugin system is proven flexible.

**Acceptance Criteria:**
- [ ] Plugin implements full `GamePlugin` trait
- [ ] Deployment strategy differs meaningfully from Plugin #1
- [ ] Tested on Windows and Linux
---
**US 4.2: Theme engine**
- As a user, I want to switch between dark and light themes, so the app looks how I want.

**Acceptance Criteria:**
- [ ] Theme defined as a JSON map of CSS variable name → value
- [ ] Built-in themes: `dark` (default) and `light`
- [ ] Theme switcher in settings panel
- [ ] Active theme persisted in user config
- [ ] Hot-swapping works without page reload
---
**US 4.3: Custom CSS**
- As an advanced user, I want to inject custom CSS, so I can fully customize the look.

**Acceptance Criteria:**
- [ ] Setting to point to a custom `.css` file on disk
- [ ] CSS is loaded and injected into the app's `<head>` at startup
- [ ] Reload button to re-apply without restarting
- [ ] Sandboxed: cannot affect Tauri system chrome
---
## Phase 5: Polish & Release
**US 5.1: Developer documentation**
- As a plugin developer, I want a plugin API guide, so I can create new game plugins.

**Acceptance Criteria:**
- [ ] Step-by-step tutorial to create a new plugin from scratch
- [ ] Full rustdoc coverage for `GamePlugin` trait and all public types
- [ ] Example plugin repository / template linked
- [ ] Published to repo Wiki or `docs/` folder
---
**US 5.2: UX polish**
- As a user, I want smooth animations and clear error messages, so the app feels professional.

**Acceptance Criteria:**
- [ ] Loading spinners during long operations (deployment, ingestion)
- [ ] Toast notifications for success/failure of key actions
- [ ] Error messages are human-readable (no raw Rust errors shown to user)
- [ ] Smooth transitions on mod list reorder
- [ ] Keyboard navigation for core actions
