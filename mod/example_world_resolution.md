# World Resolution Examples

## Default: Filename = World Name

**File**: `glint_scenes/sunset_ocean.json`
```json
{
  "world": "",
  "scenes": [
    {
      "id": "main",
      "name": "Main View"
    }
  ]
}
```
**Result**: Loads world from `saves/sunset_ocean/`

---

## Collection-Level World Override

**File**: `glint_scenes/beach_scenes.json`
```json
{
  "world": "tropical_island",
  "scenes": [
    {
      "id": "morning",
      "name": "Morning Beach"
    },
    {
      "id": "sunset",
      "name": "Sunset Beach"
    }
  ]
}
```
**Result**: Both scenes load world from `saves/tropical_island/`

---

## Scene-Level World Override

**File**: `glint_scenes/mixed_worlds.json`
```json
{
  "world": "default_world",
  "scenes": [
    {
      "id": "scene1",
      "name": "Scene in Default World"
    },
    {
      "id": "scene2",
      "name": "Scene in Custom World",
      "world": "custom_world"
    }
  ]
}
```
**Result**: 
- `scene1` → `saves/default_world/`
- `scene2` → `saves/custom_world/`

---

## Fallback Priority

1. **Scene's `world` field** (highest priority)
2. **Collection's `world` field**
3. **Filename** (without `.json`, lowest priority)
