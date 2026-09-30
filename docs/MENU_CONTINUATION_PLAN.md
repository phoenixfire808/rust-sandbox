# Spawn browser continuation: favorites and navigation

## Scope

This continuation responds to the local feedback report about missing categories and a bland browser by making category/search navigation measurable and adding real local favorites for model and spawn-catalog definitions. Favorites are a separate click target, not a substitute for selecting/spawning an item. This is a Bevy browser improvement, not a Derma recreation.

## Original implementation research

Read-only review of the installed Sandbox spawnmenu source:

- `gamemodes/sandbox/gamemode/spawnmenu/creationmenu/content/contentsearch.lua` provides a text entry, explicit magnifier/Enter search, a dedicated search-results container and result-count heading.
- `gamemodes/sandbox/gamemode/spawnmenu/creationmenu/content/contentsidebar.lua` uses a `DTree`, routes node selection through `ContentSidebarSelection`, and wires search into its content panel.
- The installed model/category ecosystem is dynamic, so this project deliberately uses its mounted model catalog and existing spawn catalog rather than inventing stock content.

The current continuation adds search-aware category counts, hides empty category matches while searching, and permits favoriting either a mounted model path or a catalog entry ID. The favorite persistence format is a bounded local JSON file under the project's ignored `local/` directory. This is not the original game's spawnlist persistence or its search index.

## State and key format

`local/menu-favorites.json` stores version 1 and at most 512 IDs, bounded to 64 KiB on load. IDs are namespaced as `model:models/...` or `entry:<catalog-id>`. Model paths are restricted to lowercase `models/` virtual paths, preventing arbitrary filesystem path use. Malformed, oversized, inaccessible, unsupported-version, or invalid-ID files are preserved and exposed as a read-only warning. Persistence failures are reported and leave both the prior file and in-memory favorites unchanged.

Favorite model controls are siblings of the spawn buttons so starring does not trigger spawning. Catalog entries expose favorite toggles in their detail panel, separate from their use button. A Favorites filter is available for model and catalog browsers and composes with search/category selection.

## Manual acceptance cases for Drew

| ID | Case | Expected |
|---|---|---|
| favorite_model_add | In Spawnlists, star a visible model | Star changes state; model does not spawn or become selected |
| favorite_model_remove | Click that model's star again | Star is removed; no spawn occurs |
| favorite_restart | Favorite a model, close normally, reopen menu/game | The model remains starred from local JSON |
| favorite_catalog | Inspect a spawn definition and toggle its favorite | Definition favorite toggles without activating/spawning it |
| favorite_filter | Switch Favorites on in each browser, then search and select categories | Only favorite records remain and filters compose; empty result is explicit |
| favorite_bad_file | With a pre-existing malformed/unsupported favorites file, attempt toggle | File remains byte-for-byte untouched; warning is shown; no replacement is attempted |
| favorite_io_failure | Make local favorites path unwritable and toggle | Status reports failure; the existing file and in-memory favorites remain unchanged |
| category_counts | Search model/catalog browser with a partial term | Visible category counts agree with current search matches; zero-match categories are hidden |
| category_reset | Narrow to a category, then select All models/All categories | Search remains, category restriction is cleared, results/counts update |
| favorite_bound | Reach 512 unique favorite IDs and try another | Extra item is refused with an explicit limit status |

Drew owns all gameplay/manual testing. No compile, tests, screenshot, or input automation was run for this work.
