---
name: unity-prefab-workflow
description: Manage Unity prefab assets with unity-cli. Use when the user asks to create a prefab from a scene object, open a prefab in edit mode, save prefab changes, instantiate a prefab, or update prefab asset properties. Do not use for general scene object editing; use `unity-gameobject-edit` or `unity-scene-create` instead.
allowed-tools: Bash(unity-cli:*), Read, Grep, Glob
metadata:
  author: akiojin
  version: 0.3.0
  category: prefabs
  triggers:
    - prefab
    - instantiate
    - edit-mode
    - asset
  siblings:
    - unity-gameobject-edit
    - unity-scene-create
    - unity-asset-management
---

# Prefab Workflow

Create, open, edit, and instantiate prefab assets via `unity-cli`. This skill owns the prefab edit-mode lifecycle; scene-instance edits belong to `unity-gameobject-edit`.

## Use When

- The user wants to create or update a prefab asset.
- The task requires entering prefab edit mode and saving changes back to the asset.
- The user wants to instantiate prefab assets into a scene with specific transforms.

## Do Not Use When

- The request edits ordinary scene objects with no prefab asset involved; use `unity-gameobject-edit`.
- The task is bootstrapping a fresh scene; use `unity-scene-create`.
- The request is about asset import settings or materials; use `unity-asset-management`.

## Preferred Flow

1. `open_prefab` to enter edit mode (or `create_prefab` from a scene object).
2. Apply edits via `add_component`, `modify_component`, or `set_component_field`.
3. `save_prefab` to persist changes back to the asset.
4. `exit_prefab_mode` to return to scene authoring.

```bash
unity-cli raw create_prefab --json '{"gameObjectPath":"/Player","prefabPath":"Assets/Prefabs/Player.prefab"}'
unity-cli raw open_prefab --json '{"prefabPath":"Assets/Prefabs/Player.prefab"}'
unity-cli raw save_prefab --json '{}'
unity-cli raw exit_prefab_mode --json '{}'
unity-cli raw instantiate_prefab --json '{"prefabPath":"Assets/Prefabs/Player.prefab","position":{"x":0,"y":0,"z":0}}'
```

## Examples

- "Create a prefab from `/Player` and save it under `Assets/Prefabs/Player.prefab`."
- "Open an existing prefab, change a field, save it, and exit prefab mode."
- "Instantiate a prefab at the origin for a test scene."

## References

- [runtime-checklist.md](references/runtime-checklist.md): connection and instance prerequisites.
- [prefab-edit-mode.md](references/prefab-edit-mode.md): safe sequence for edit mode, scene handoff, and instantiation checks.
