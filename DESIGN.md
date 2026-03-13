# Consciousness Simulation 3D Visualizer — Design Document

## 1. Overview

This application visualizes a Python-based consciousness simulation as a 3D humanoid character inhabiting a room, rendered with Three.js inside an Electron shell. The simulation models 20 (or 32) psychological aspects — such as agency, emotional awareness, introspection, and self-regulation — whose weighted interplay produces emergent behaviors like pacing, sitting, gesturing, and idle fidgeting. The Python engine runs as a child process, streaming JSON state at 20 Hz; the renderer interpolates to 60 fps and translates abstract weight vectors into visible character actions, environmental lighting shifts, and a minimal HUD.

**Tech stack:** Electron (main + renderer processes), Three.js (WebGL), Python 3 subprocess (stdin/stdout JSON-lines IPC).

---

## 2. Architecture

### 2.1 Process Model

```
┌──────────────────────────────────────────────────────┐
│  Electron Main Process (main.js)                      │
│  - Spawns Python child process (bridge.py)            │
│  - Forwards JSON messages to renderer via IPC         │
│  - Handles app lifecycle, window creation             │
│  - Forwards user input from renderer → Python stdin   │
└──────────────┬──────────────────────┬─────────────────┘
               │ IPC (contextBridge)  │ stdin/stdout
               ▼                      ▼
┌──────────────────────┐  ┌─────────────────────────────┐
│  Electron Renderer   │  │  Python child process        │
│  (index.html + JS)   │  │  (bridge.py)                 │
│  - Three.js scene    │  │  - Wraps ConsciousnessEngine │
│  - Animation loop    │  │  - Ticks at 20 Hz            │
│  - HUD overlay       │  │  - Emits JSON-lines on stdout│
│  - Input capture     │  │  - Reads JSON-lines on stdin │
└──────────────────────┘  └─────────────────────────────┘
```

### 2.2 Communication Protocol

**Python → Electron (stdout):** One JSON object per line, no pretty-printing. Each message has a `type` field.

**Electron → Python (stdin):** One JSON object per line. Each message has a `type` field.

### 2.3 File Structure

```
consciousness-3d/
├── package.json
├── main.js                    # Electron main process
├── preload.js                 # contextBridge for IPC
├── index.html                 # Single-page shell
├── bridge.py                  # Python-side IPC bridge
├── src/
│   ├── app.js                 # Entry point: boots scene, starts IPC
│   ├── ipc.js                 # Renderer-side IPC helpers
│   ├── scene/
│   │   ├── room.js            # Room geometry, materials, lights
│   │   └── lighting.js        # Dynamic lighting controller
│   ├── character/
│   │   ├── model.js           # Body part meshes + joint hierarchy
│   │   ├── materials.js       # Character material/color system
│   │   └── skeleton.js        # Joint names, rest poses, angle limits
│   ├── animation/
│   │   ├── animator.js        # Master animation controller + blend
│   │   ├── procedural.js      # Joint-angle formulas per behavior
│   │   └── locomotion.js      # Pathfinding + walk cycle driver
│   ├── behavior/
│   │   ├── scorer.js          # Weight-to-behavior scoring (mirrors Python)
│   │   ├── stateMachine.js    # IDLE/WALKING/PERFORMING FSM
│   │   └── behaviors.js       # Behavior definitions + parameters
│   ├── environment/
│   │   ├── events.js          # 3D stimulus/event visualization
│   │   └── particles.js       # Particle system for event effects
│   ├── state/
│   │   ├── store.js           # Central state container
│   │   └── interpolator.js    # 20Hz→60fps interpolation
│   └── hud/
│       ├── hud.js             # HUD root: minimal + expanded views
│       ├── miniPanel.js       # Compact overlay (always visible)
│       └── detailPanel.js     # Expanded data view (toggle)
└── core/                      # Existing Python package (symlink or path ref)
    ├── __init__.py
    ├── config.py
    ├── engine.py
    ├── environment.py
    ├── dynamics.py
    ├── interrelations.py
    ├── analyzer.py
    ├── personality.py
    ├── memory.py
    ├── energy.py
    └── visualization.py
```

### 2.4 Dependencies

**npm (package.json):**
```json
{
  "dependencies": {
    "electron": "^30.0.0",
    "three": "^0.165.0"
  },
  "devDependencies": {}
}
```

No bundler required — use ES module `<script type="importmap">` in index.html pointing to `node_modules/three`.

**Python:** numpy (already required by core). No additional packages. The bridge.py uses only stdlib (`sys`, `json`, `time`, `threading`) plus the existing `core` package.

---

## 3. Python Bridge

### 3.1 bridge.py Specification

`bridge.py` wraps `ConsciousnessEngine` and communicates via JSON-lines over stdin/stdout. It must live adjacent to the `core/` package directory.

**Startup sequence:**
1. Parse optional CLI args: `--expanded`, `--personality <name>`, `--energy <float>`, `--circadian <int>`
2. Construct `SimConfig` or `EXPANDED_32_CONFIG` with `vis_enabled=False`, `input_backend='none'`
3. Create `ConsciousnessEngine(config)`
4. Print startup handshake message to stdout (single JSON line)
5. Start stdin reader thread
6. Enter tick loop at 20 Hz (50 ms interval)

**Startup handshake message (type: "init"):**
```json
{
  "type": "init",
  "aspects": ["body_awareness", "emotional_awareness", ...],
  "categories": {
    "cognitive": ["introspection", "reflection", ...],
    "emotional": ["emotional_awareness", "self-esteem", ...],
    "social": [...],
    "executive": [...],
    "existential": [...]
  },
  "categoryColors": {
    "cognitive": "#4A90D9",
    "emotional": "#E8613C",
    "social": "#3CB371",
    "executive": "#9B59B6",
    "existential": "#DAA520"
  },
  "personality": "contemplative",
  "aspectCount": 20
}
```

**Tick message (type: "tick"), emitted at 20 Hz:**
```json
{
  "type": "tick",
  "tick": 142,
  "weights": {
    "body_awareness": 0.3421,
    "emotional_awareness": -0.1234,
    ...
  },
  "energy": {
    "energy": 87.32,
    "energy_pct": 87.3,
    "arousal": 0.7123,
    "stress": 0.0421,
    "attended": ["agency", "motivation", "self-regulation", "self-efficacy", "metacognition"],
    "flow_states": ["drive"],
    "circadian": 0.912
  },
  "analysis": {
    "phases": [
      {"phase": "stability", "aspect": "body_awareness", "confidence": 0.92, "duration": 34, "slope": 0.001}
    ],
    "entropy": {"shannon": 4.12, "normalized": 0.85, "delta": -0.01, "complexity_label": "active"},
    "cascades": [],
    "attractors": [{"basin_radius": 0.23, "strength": 2.1, "drift_rate": 0.003}],
    "resilience": null
  },
  "envStatus": "social_interaction | valence=+0.12",
  "activeStimuli": ["social_interaction", "mind_wandering"],
  "valence": 0.12,
  "behavior": {
    "primary": "pacing",
    "scores": {
      "idle_stand": 0.15,
      "idle_fidget": 0.22,
      "pacing": 0.48,
      "sitting_think": 0.10,
      "sitting_slump": 0.03,
      "gesture_emphatic": 0.02
    },
    "intensity": 0.72
  }
}
```

**Stdin input messages (Electron → Python):**

User input injection:
```json
{"type": "input", "direction": "positive"}
{"type": "input", "direction": "negative"}
{"type": "input", "direction": "none"}
```

Configuration change:
```json
{"type": "config", "personality": "empathic"}
```

### 3.2 Behavior Scoring (Computed Python-Side)

The bridge computes a `behavior` object each tick using the current weight dictionary. This scoring runs after `engine.step()` returns.

**Behavior scoring formulas:**

Each behavior has a scoring function of the form `score = clamp01(sum of weighted aspects)`. The weights below are the coefficients applied to the aspect values from `weight_dict` (which range [-1, 1]).

| Behavior | Formula | Description |
|---|---|---|
| `idle_stand` | `0.3 + 0.2*self_regulation + 0.15*patience - 0.2*abs(motivation) - 0.15*abs(agency)` | Calm standing when regulated and not driven |
| `idle_fidget` | `0.1 + 0.3*abs(emotional_awareness) + 0.2*stress - 0.2*self_regulation + 0.15*body_awareness` | Restless fidgeting from emotional arousal or stress |
| `pacing` | `0.15 + 0.25*motivation + 0.2*agency + 0.15*abs(introspection) - 0.1*self_regulation + 0.1*stress` | Walking back and forth when driven or ruminating |
| `sitting_think` | `0.1 + 0.3*introspection + 0.25*reflection + 0.2*metacognition - 0.15*agency` | Seated contemplation when cognitively engaged |
| `sitting_slump` | `0.05 + 0.3*neg(self_esteem) + 0.2*neg(motivation) + 0.2*neg(agency) + 0.15*stress` | Dejected posture from low self-esteem and motivation |
| `gesture_emphatic` | `0.1 + 0.3*social_awareness + 0.25*theory_of_mind + 0.2*emotional_awareness + 0.15*agency` | Animated gesturing during social/emotional activation |
| `gesture_dismiss` | `0.05 + 0.2*neg(social_awareness) + 0.2*neg(theory_of_mind) + 0.15*agency + 0.1*self_regulation` | Dismissive hand wave when socially disengaged |
| `startle` | `0.0 + 0.5*stress_spike + 0.3*situational_awareness + 0.2*body_awareness` | Sudden jolt on stress spike (transient) |

Where:
- `neg(x)` = `max(0, -x)` — only activates when the aspect is negative
- `stress` = `energy_state['stress']`
- `stress_spike` = `max(0, current_stress - previous_stress)` (only positive deltas)
- All scores are clamped to `[0, 1]`
- The behavior with the highest score is `primary`
- `intensity` = the primary behavior's score value

**Hysteresis rule:** The current behavior persists unless a competing behavior exceeds the current by at least `0.12` (the hysteresis margin). This prevents rapid flickering.

---

## 4. 3D Scene

### 4.1 Room

- **Dimensions:** 8m wide (X) x 6m deep (Z) x 3.5m tall (Y)
- **Origin:** Center of floor at (0, 0, 0)
- **Floor:** `MeshStandardMaterial`, color `#2a2a2e`, roughness 0.85, metalness 0.05
- **Walls:** `MeshStandardMaterial`, color `#1e1e22`, roughness 0.9, metalness 0.0. Back wall at Z=-3, left wall at X=-4, right wall at X=4. No front wall (camera side).
- **Ceiling:** `MeshStandardMaterial`, color `#18181c`, roughness 0.95
- **Furniture (simple box geometry):**
  - Chair: position (2.5, 0.25, -1.5), seat 0.45m x 0.45m x 0.05m at Y=0.45, backrest 0.45m x 0.55m x 0.05m. Color `#3d3530`.
  - Table: position (-2.0, 0, -2.0), top 1.0m x 0.6m x 0.04m at Y=0.75. Color `#3d3530`.

### 4.2 Camera

- **Type:** `PerspectiveCamera`, FOV 50, near 0.1, far 50
- **Position:** (0, 2.8, 7.5) — slightly above eye level, looking inward
- **Target:** (0, 1.0, 0) — center of room at about waist height
- **Fixed:** No orbit controls, no user camera movement. The camera never changes.

### 4.3 Lighting

**Static lights (always present):**
- `AmbientLight`: color `#1a1a2e`, intensity 0.3
- `DirectionalLight` (key): color `#c8b8a0`, intensity 0.6, position (3, 4, 2), castShadow=true, shadow mapSize 1024x1024
- `PointLight` (fill): color `#4a5580`, intensity 0.3, position (-3, 3, -1)
- `HemisphereLight`: sky `#1a1a3e`, ground `#0a0a0a`, intensity 0.2

**Dynamic lighting (driven by simulation state):**

The lighting controller (`lighting.js`) adjusts lights every frame based on interpolated state:

| Parameter | Source | Effect |
|---|---|---|
| Environmental valence | `valence` field ([-1, 1]) | Key light hue shifts: positive → warm amber `#d4a050`, negative → cool blue `#4060a0`. Interpolate via HSL. |
| Stress | `energy.stress` ([0, 1]) | Ambient intensity pulses: `0.3 + 0.15 * stress * sin(time * 3.0)`. At stress > 0.6, a dim red `PointLight` at (0, 3, 0) fades in with intensity `0.2 * stress`. |
| Energy level | `energy.energy_pct` ([0, 100]) | Overall scene brightness scales by `0.5 + 0.5 * (energy_pct / 100)`. Below 30%, lights dim noticeably. |
| Flow state | `energy.flow_states` (array) | When any flow state is active, add a subtle overhead `SpotLight` with color `#6080c0`, intensity 0.2, cone angle 45 deg, creating a "spotlight" feel. |
| Circadian | `energy.circadian` ([0.5, 1.0]) | Ambient and hemisphere light intensity multiplied by circadian factor. |

---

## 5. Character Model

### 5.1 Body Parts (Three.js Primitives)

All dimensions in meters. The character stands approximately 1.75m tall. Each body part is a `Mesh` with `BoxGeometry` (or `SphereGeometry` for head).

| Part | Geometry | Dimensions (W x H x D) | Pivot Offset (from parent joint) | Color |
|---|---|---|---|---|
| **pelvis** | Box | 0.30 x 0.18 x 0.18 | Scene root at (0, 0.92, 0) | `#4a5568` |
| **torso** | Box | 0.32 x 0.40 x 0.20 | Top of pelvis (0, 0.09, 0) | `#4a5568` |
| **neck** | Box | 0.08 x 0.10 x 0.08 | Top of torso (0, 0.20, 0) | `#c8a882` |
| **head** | Sphere | radius 0.12 | Top of neck (0, 0.05, 0) | `#c8a882` |
| **upperArmL** | Box | 0.08 x 0.28 x 0.08 | Torso left shoulder (-0.20, 0.18, 0) | `#4a5568` |
| **upperArmR** | Box | 0.08 x 0.28 x 0.08 | Torso right shoulder (0.20, 0.18, 0) | `#4a5568` |
| **forearmL** | Box | 0.07 x 0.26 x 0.07 | Bottom of upperArmL (0, -0.14, 0) | `#c8a882` |
| **forearmR** | Box | 0.07 x 0.26 x 0.07 | Bottom of upperArmR (0, -0.14, 0) | `#c8a882` |
| **handL** | Box | 0.06 x 0.08 x 0.04 | Bottom of forearmL (0, -0.13, 0) | `#c8a882` |
| **handR** | Box | 0.06 x 0.08 x 0.04 | Bottom of forearmR (0, -0.13, 0) | `#c8a882` |
| **thighL** | Box | 0.10 x 0.38 x 0.10 | Pelvis left hip (-0.10, -0.09, 0) | `#3d4452` |
| **thighR** | Box | 0.10 x 0.38 x 0.10 | Pelvis right hip (0.10, -0.09, 0) | `#3d4452` |
| **shinL** | Box | 0.08 x 0.38 x 0.08 | Bottom of thighL (0, -0.19, 0) | `#3d4452` |
| **shinR** | Box | 0.08 x 0.38 x 0.08 | Bottom of thighR (0, -0.19, 0) | `#3d4452` |
| **footL** | Box | 0.09 x 0.05 x 0.18 | Bottom of shinL (0, -0.19, 0.04) | `#2d2d2d` |
| **footR** | Box | 0.09 x 0.05 x 0.18 | Bottom of shinR (0, -0.19, 0.04) | `#2d2d2d` |

### 5.2 Joint Hierarchy

```
Scene
└── pelvis (Group) — root transform: position + Y-rotation
    ├── torso (Group) — rotX (lean forward/back), rotZ (lean sideways)
    │   ├── neck (Group) — rotX, rotZ
    │   │   └── head (Mesh) — rotX (nod), rotY (turn), rotZ (tilt)
    │   ├── shoulderL (Group) — rotX (raise/lower), rotZ (spread)
    │   │   └── upperArmL (Group) — rotX (swing)
    │   │       └── elbowL (Group) — rotX only, range [0, 2.6] rad
    │   │           └── forearmL
    │   │               └── handL — rotX (wrist flex)
    │   └── shoulderR (Group) — mirror of shoulderL
    │       └── upperArmR (Group)
    │           └── elbowR (Group)
    │               └── forearmR
    │                   └── handR
    ├── hipL (Group) — rotX (forward/back swing), rotZ
    │   └── thighL
    │       └── kneeL (Group) — rotX only, range [0, 2.5] rad
    │           └── shinL
    │               └── footL — rotX (ankle flex)
    └── hipR (Group) — mirror of hipL
        └── thighR
            └── kneeR (Group)
                └── shinR
                    └── footR
```

Every joint is a `THREE.Group`. Meshes are children of their respective groups with local offsets so rotation happens at the joint, not at the mesh center.

### 5.3 Material System

All character materials are `MeshStandardMaterial`.

- **Clothing** (torso, upperArms, thighs, shins): Base color `#4a5568`, roughness 0.75, metalness 0.05
- **Skin** (head, neck, forearms, hands): Base color `#c8a882`, roughness 0.6, metalness 0.0
- **Shoes** (feet): Base color `#2d2d2d`, roughness 0.9, metalness 0.1

**Dynamic color modulation:** The torso material emissive color shifts based on the dominant category cluster:
- Compute which category has the highest average absolute weight
- Blend torso emissive toward that category's color at intensity `0.05 * dominance` where `dominance` is the average absolute weight of the category's aspects
- This produces a very subtle glow — barely noticeable but present

---

## 6. Animation System

### 6.1 Core Architecture

The animation system is purely procedural — no keyframes, no animation clips. Each behavior defines joint-angle functions of time. The `animator.js` calls these functions every frame.

**Time base:** `t` = elapsed seconds from `performance.now() / 1000`. All angle formulas use `t` as input.

**Blend system:** The animator maintains a `currentPose` object (dict of joint-name → {rotX, rotY, rotZ}). Each frame:
1. Compute the target pose from the active behavior's procedural functions
2. Lerp each joint angle: `current = lerp(current, target, blendRate * dt)`
3. `blendRate` = 6.0 for normal transitions, 12.0 for `startle`
4. `dt` = frame delta time in seconds (typically ~0.0167)

### 6.2 Procedural Joint-Angle Formulas

All angles in radians. `intensity` = behavior intensity from scorer (0 to 1).

#### idle_stand
```
pelvis.posY      = 0.92
torso.rotX       = 0.02 * sin(t * 0.4)                          // subtle sway
torso.rotZ       = 0.01 * sin(t * 0.3)
head.rotX        = 0.03 * sin(t * 0.25)                         // gentle nod
head.rotY        = 0.04 * sin(t * 0.15)                         // slow look around
shoulderL.rotZ   = 0.05                                         // arms at sides
shoulderR.rotZ   = -0.05
elbowL.rotX      = 0.1
elbowR.rotX      = 0.1
hipL.rotX        = 0
hipR.rotX        = 0
kneeL.rotX       = 0.02
kneeR.rotX       = 0.02
```

#### idle_fidget
```
torso.rotX       = 0.03 * sin(t * 1.2) * intensity
torso.rotZ       = 0.04 * sin(t * 0.9) * intensity
head.rotX        = 0.05 * sin(t * 1.5) * intensity
head.rotY        = 0.08 * sin(t * 0.7) * intensity
shoulderL.rotZ   = 0.05 + 0.1 * sin(t * 1.8) * intensity
shoulderR.rotZ   = -0.05 - 0.08 * sin(t * 2.1) * intensity
elbowL.rotX      = 0.3 + 0.2 * sin(t * 1.4) * intensity        // arm movement
elbowR.rotX      = 0.2 + 0.15 * sin(t * 1.7) * intensity
pelvis.posY      = 0.92 + 0.005 * sin(t * 2.0) * intensity     // slight bounce
hipL.rotX        = 0.03 * sin(t * 0.8) * intensity              // weight shifting
hipR.rotX        = -0.03 * sin(t * 0.8) * intensity
```

#### pacing (walk cycle — applied when state machine is in WALKING)
```
// phase = walk cycle phase, driven by locomotion.js
// stride frequency scales with intensity: freq = 1.5 + intensity * 1.0 (Hz)
// phase = (t * freq * 2 * PI)

hipL.rotX        = 0.35 * sin(phase) * intensity
hipR.rotX        = 0.35 * sin(phase + PI) * intensity
kneeL.rotX       = max(0, 0.5 * sin(phase - 0.5)) * intensity
kneeR.rotX       = max(0, 0.5 * sin(phase + PI - 0.5)) * intensity
footL.rotX       = 0.15 * sin(phase + 0.3) * intensity
footR.rotX       = 0.15 * sin(phase + PI + 0.3) * intensity

shoulderL.rotX   = -0.2 * sin(phase) * intensity                // counter-swing
shoulderR.rotX   = -0.2 * sin(phase + PI) * intensity
elbowL.rotX      = 0.3 + 0.15 * sin(phase + 0.5) * intensity
elbowR.rotX      = 0.3 + 0.15 * sin(phase + PI + 0.5) * intensity

torso.rotZ       = 0.03 * sin(phase) * intensity                // hip sway
pelvis.rotY      = facing direction (toward movement target)
pelvis.posY      = 0.92 + 0.02 * abs(sin(phase * 2)) * intensity // bounce
```

#### sitting_think
```
// Character at chair position (2.5, 0, -1.5), facing away from back wall
pelvis.posY      = 0.45                                          // seated height
hipL.rotX        = -1.57                                         // 90 deg seated
hipR.rotX        = -1.57
kneeL.rotX       = 1.57
kneeR.rotX       = 1.57
torso.rotX       = 0.15 + 0.03 * sin(t * 0.3)                  // slight lean forward
head.rotX        = 0.1 + 0.05 * sin(t * 0.2) * intensity       // nodding thoughtfully
head.rotY        = 0.06 * sin(t * 0.12)                         // slow look
shoulderL.rotX   = -0.3                                          // hand on chin
elbowL.rotX      = 1.8 + 0.1 * sin(t * 0.25) * intensity
shoulderR.rotX   = -0.1
elbowR.rotX      = 0.5
```

#### sitting_slump
```
pelvis.posY      = 0.42                                          // lower seated
hipL.rotX        = -1.57
hipR.rotX        = -1.57
kneeL.rotX       = 1.57
kneeR.rotX       = 1.8                                           // one leg extended
torso.rotX       = 0.35 + 0.02 * sin(t * 0.15)                 // slumped forward
torso.rotZ       = 0.05                                          // slightly askew
head.rotX        = 0.3                                           // head drooped
head.rotZ        = 0.08 * sin(t * 0.1)
shoulderL.rotZ   = 0.15                                          // arms hanging
shoulderR.rotZ   = -0.15
elbowL.rotX      = 0.8
elbowR.rotX      = 0.7
```

#### gesture_emphatic
```
// Standing position, animated arms
torso.rotX       = -0.05 + 0.05 * sin(t * 1.5) * intensity
torso.rotZ       = 0.04 * sin(t * 1.2) * intensity
head.rotY        = 0.1 * sin(t * 0.8) * intensity

// Right arm gesticulating
shoulderR.rotX   = -0.8 - 0.4 * sin(t * 2.0) * intensity
shoulderR.rotZ   = -0.3 - 0.2 * sin(t * 1.6) * intensity
elbowR.rotX      = 0.8 + 0.6 * sin(t * 2.5) * intensity
handR.rotX       = 0.3 * sin(t * 3.0) * intensity

// Left arm smaller gestures
shoulderL.rotX   = -0.3 - 0.2 * sin(t * 1.8) * intensity
elbowL.rotX      = 0.5 + 0.3 * sin(t * 2.2) * intensity

// Slight weight shift
hipL.rotX        = 0.03 * sin(t * 0.6)
hipR.rotX        = -0.03 * sin(t * 0.6)
```

#### gesture_dismiss
```
// Quick wave-off motion on right hand, otherwise mostly still
shoulderR.rotX   = -0.6 * intensity
shoulderR.rotZ   = -0.4 * intensity
elbowR.rotX      = 0.5
handR.rotX       = -0.3 + 0.4 * sin(t * 4.0) * intensity      // rapid wrist flick
head.rotY        = -0.15 * intensity                            // look away
torso.rotZ       = -0.03 * intensity
```

#### startle
```
// Rapid onset (blendRate = 12.0), decays after 0.4 seconds
// startleT = time since startle began
decay            = max(0, 1.0 - startleT / 0.4)

torso.rotX       = -0.15 * decay * intensity                   // jerk back
shoulderL.rotX   = -0.5 * decay * intensity                    // arms up
shoulderR.rotX   = -0.5 * decay * intensity
shoulderL.rotZ   = 0.3 * decay * intensity
shoulderR.rotZ   = -0.3 * decay * intensity
elbowL.rotX      = 1.2 * decay * intensity
elbowR.rotX      = 1.2 * decay * intensity
head.rotX        = -0.1 * decay * intensity                    // head back
kneeL.rotX       = 0.15 * decay * intensity                    // slight crouch
kneeR.rotX       = 0.15 * decay * intensity
pelvis.posY      = 0.92 - 0.03 * decay * intensity             // drop slightly
```

### 6.3 Locomotion

`locomotion.js` handles moving the pelvis root to target positions in the room.

**Walkable area:** Rectangle from (-3.5, -2.5) to (3.5, 2.5) in (X, Z) space. The chair at (2.5, -1.5) has a 0.5m exclusion radius.

**Pathfinding:** Simple direct-line movement. If the straight line to the target passes within 0.5m of the chair center, add a single waypoint offset 0.7m perpendicular to the line from the chair center.

**Target selection:**
- `pacing`: Pick a random point on the walkable boundary (X or Z extremes), alternating sides. When reached, pick the opposite side. Dwell time at each endpoint: `1.0 + random() * 2.0` seconds.
- `sitting_think` / `sitting_slump`: Target = chair position (2.5, 0, -1.5)
- `gesture_emphatic` / `gesture_dismiss`: Stay in place (current position)
- `idle_stand` / `idle_fidget`: Stay in place or drift slowly to a random nearby point within 1m radius over 3-5 seconds.

**Movement speed:** `0.8 + 0.6 * intensity` m/s during walking. The pelvis Y-rotation smoothly faces the movement direction (lerp rotY toward `atan2(dx, dz)` at rate 4.0/s).

---

## 7. Behavior System

### 7.1 Behavior Scoring

Behavior scoring is computed Python-side in `bridge.py` (see Section 3.2) and sent with each tick message. The renderer in `scorer.js` does NOT recompute scores — it only reads the `behavior` field from the tick data.

### 7.2 State Machine

`stateMachine.js` implements a three-state FSM:

```
        ┌──────────┐
        │          │
   ┌────▼────┐     │
   │  IDLE   │─────┘ (behavior is idle_stand or idle_fidget)
   │         │
   └────┬────┘
        │ (behavior requires different position)
   ┌────▼────┐
   │ WALKING │ ← locomotion active, walk cycle plays
   │         │
   └────┬────┘
        │ (arrived at target)
   ┌────▼────────┐
   │ PERFORMING  │ ← behavior animation plays (sitting, gesturing, etc.)
   │             │
   └─────────────┘
        │ (new behavior selected → back to IDLE or WALKING)
```

**Transitions:**
- `IDLE → WALKING`: When the target behavior requires a position different from the current one (e.g., switching to `sitting_think` requires moving to the chair). Threshold: distance to target > 0.3m.
- `WALKING → PERFORMING`: When distance to target < 0.2m.
- `PERFORMING → IDLE`: When the behavior changes to an idle variant.
- `PERFORMING → WALKING`: When the behavior changes to one requiring a different position.
- `Any → IDLE`: When `idle_stand` or `idle_fidget` is selected.

**Hysteresis:** The behavior from `bridge.py` already includes hysteresis (Section 3.2). The state machine trusts the `primary` field and only handles position transitions.

### 7.3 Behavior Intensity

The `intensity` value (0 to 1) from the Python bridge modulates:
- Animation amplitude (multiplied into procedural formulas)
- Movement speed during WALKING
- Head/torso sway frequency (higher intensity = slightly faster oscillation, via `intensity * 0.3` additive to frequency)
- Gesture reach distance

---

## 8. Environmental Events

### 8.1 Stimulus Visualization

Each active stimulus from `activeStimuli` array manifests as a visual effect in the 3D scene. Effects are managed by `events.js`.

| Stimulus | Visual Effect |
|---|---|
| `social_interaction` | A translucent humanoid silhouette (flat `PlaneGeometry` with alpha texture, 0.5m wide, 1.7m tall) fades in at a random position 2-3m from the character, facing the character. Color `#3CB371`, opacity 0.0→0.4→0.0 over 3 seconds. |
| `challenge` | An orange `#E8613C` wireframe icosahedron (radius 0.3m) appears at (0, 2.0, -2.0), rotates slowly, pulses in scale (1.0→1.2→1.0) over 2s, then fades out. |
| `threat` | A red `#cc3333` pulsing sphere (radius 0.15m) at a random edge of the room. Emits 20 small red particles outward. PointLight with color `#ff2020`, intensity 0.3, distance 4m, at the sphere position. Fades over 2.5s. |
| `reward` | A golden `#DAA520` particle burst: 30 particles rise from the floor at the character's feet, drift upward 1.5m, fade out over 2s. Each particle is a small `SphereGeometry` (radius 0.02m). |
| `loss` | A dark blue `#1a2040` fog sphere (radius 1.5m) centered on the character, `MeshBasicMaterial` with opacity 0.0→0.15→0.0 over 3s. Inside, a few slow-falling particle motes. |
| `novelty` | A bright white `#ffffff` flash: a `PlaneGeometry` covering the full viewport with opacity 0.0→0.08→0.0 over 0.3s, then a small spinning question mark sprite (`?` text sprite, `#4A90D9`) floats up from (0, 1.5, 0) over 2s. |
| `moral_dilemma` | Two small cubes (0.15m), one `#3CB371` and one `#E8613C`, orbit each other at (0, 2.0, 0) for 3 seconds, then merge into a `#9B59B6` cube and fade. |
| `flow_state` | A subtle ring of light (`TorusGeometry`, major 0.8m, minor 0.01m) at the character's waist level, color `#6080c0`, slowly rotating, opacity 0.3, persists while flow is active. |
| `social_rejection` | The social silhouette (if present) turns away and walks into the wall, fading to red `#cc3333`. If no silhouette, a brief red flash on the floor under the character. |
| `accomplishment` | Same as `reward` but with more particles (50), larger (radius 0.03m), and a brief `SpotLight` from above, color `#DAA520`, intensity 0.5, lasting 1.5s. |

### 8.2 Internal Event Visualization

| Internal Event | Visual Effect |
|---|---|
| `mind_wandering` | 5-8 small translucent spheres (`#4A90D9`, radius 0.05m) drift slowly in random directions around the character's head (within 0.5m radius), orbiting for 3s. |
| `sudden_insight` | A bright `#ffffff` to `#DAA520` light bulb sprite appears above the character's head at (charX, 2.2, charZ), glows briefly (0.5s peak), then fades over 1.5s. |
| `intrusive_thought` | A small jagged shape (`DodecahedronGeometry`, radius 0.1m, color `#cc3333`, wireframe) appears at head height, jitters randomly for 2s, then dissolves. |
| `self_doubt` | The character's shadow darkens and grows slightly larger (scale shadow plane by 1.3x) for 2s, then returns to normal. |
| `nostalgia` | A warm amber `#d4a050` glow emanates from the character (PointLight at character center, intensity 0→0.4→0, over 3s). |
| `creative_impulse` | 3-5 small colored cubes (random bright colors, 0.05m) pop out from the character's hands, float outward in arcs, and fade over 2s. |

### 8.3 Event Lifecycle

Every visual event follows a three-phase lifecycle:

1. **Onset** (0 to `onsetDuration`): Object spawns, opacity/scale ramps from 0 to peak.
2. **Peak** (`onsetDuration` to `onsetDuration + peakDuration`): Full visibility, any looping animation plays.
3. **Decay** (`peakDuration` to end): Opacity/scale ramps to 0, object is removed from scene.

Default timing (overridable per event type):
- `onsetDuration`: 0.3s
- `peakDuration`: 1.5s
- `decayDuration`: 1.0s
- Total: 2.8s

`events.js` maintains an array of active event objects. Each frame, it advances all active events by `dt` and removes finished ones.

### 8.4 User Input Visualization

When `direction` is `"positive"`:
- A green `#3CB371` expanding ring (`TorusGeometry`, starts radius 0.1m, expands to 2.0m at the character's feet) over 0.8s, then fades.
- 15 green particles rise from the floor.

When `direction` is `"negative"`:
- A red `#E8613C` contracting ring (starts radius 2.0m, shrinks to 0.1m) over 0.8s, then fades.
- 15 red particles fall from ceiling height (Y=3.0) toward the character.

---

## 9. State Management

### 9.1 Central Store (`store.js`)

The store holds:
```javascript
{
  // Latest complete tick from Python
  latestTick: null,           // full tick JSON object
  previousTick: null,         // the tick before latestTick

  // Interpolated state (updated every render frame)
  interpolated: {
    weights: {},              // aspect → float
    energy: {},
    stress: 0,
    valence: 0,
    arousal: 0,
    circadian: 1.0,
  },

  // Behavior state
  behavior: {
    current: 'idle_stand',
    intensity: 0.0,
    scores: {},
  },

  // Active 3D events
  activeEvents: [],           // managed by events.js

  // Character transform
  character: {
    position: { x: 0, y: 0, z: 0 },
    targetPosition: { x: 0, y: 0, z: 0 },
    facingAngle: 0,
  },

  // Init data from handshake
  init: null,

  // HUD state
  hudExpanded: false,
}
```

### 9.2 Interpolation (`interpolator.js`)

Python ticks arrive at 20 Hz (every 50 ms). The render loop runs at 60 fps (~16.7 ms). Between ticks, the interpolator produces smooth intermediate states.

**Method:** Linear interpolation between `previousTick` and `latestTick`.

```javascript
// Called every render frame
function interpolate(store, frameTimestamp) {
  const { previousTick, latestTick } = store;
  if (!previousTick || !latestTick) return;

  const tickInterval = 50; // ms
  const elapsed = frameTimestamp - latestTickReceivedAt;
  const alpha = clamp(elapsed / tickInterval, 0, 1);

  for (const aspect of aspects) {
    store.interpolated.weights[aspect] = lerp(
      previousTick.weights[aspect],
      latestTick.weights[aspect],
      alpha
    );
  }

  store.interpolated.stress = lerp(previousTick.energy.stress, latestTick.energy.stress, alpha);
  store.interpolated.valence = lerp(previousTick.valence, latestTick.valence, alpha);
  store.interpolated.arousal = lerp(previousTick.energy.arousal, latestTick.energy.arousal, alpha);
  store.interpolated.energy = lerp(previousTick.energy.energy_pct, latestTick.energy.energy_pct, alpha);
  store.interpolated.circadian = lerp(previousTick.energy.circadian, latestTick.energy.circadian, alpha);
}

function lerp(a, b, t) { return a + (a - b) * t; } // note: a + (b - a) * t
function clamp(v, lo, hi) { return Math.max(lo, Math.min(hi, v)); }
```

When a new tick arrives, `previousTick = latestTick`, `latestTick = newTick`, `latestTickReceivedAt = performance.now()`.

### 9.3 Event Queue

When a tick arrives with `activeStimuli` that were not in the previous tick's `activeStimuli`, `events.js` spawns new visual events. It compares the two arrays to detect new entries.

---

## 10. HUD

### 10.1 Implementation

The HUD is rendered as HTML/CSS overlays on top of the Three.js canvas. The renderer `index.html` contains a `<div id="hud">` positioned absolutely over the `<canvas>`.

Toggle between minimal and expanded view with the `Tab` key.

### 10.2 Minimal Overlay (Always Visible)

Position: bottom-left corner, 16px from edges.

```
┌─────────────────────────────┐
│  ● CALM          tick 1042  │
│  energy ████████░░  82%     │
│  stress ██░░░░░░░░  0.12    │
│  valence         +0.23      │
│  behavior: idle_stand       │
│  [Tab] for details          │
└─────────────────────────────┘
```

**Styling:**
- Background: `rgba(13, 17, 23, 0.75)` (matches `#0D1117` with transparency)
- Border: `1px solid rgba(255, 255, 255, 0.1)`
- Border radius: `8px`
- Padding: `12px 16px`
- Font: `'SF Mono', 'Fira Code', monospace`, 12px
- Text color: `#aaaaaa`
- Mental state label: colored per `detect_mental_state` output:
  - `calm` → `#DAA520`
  - `growing` → `#3CB371`
  - `agitated` → `#E8613C`
  - `conflicted` → `#9B59B6`
  - `recovering` → `#4A90D9`
- The `●` indicator dot uses the same color as the label
- Energy and stress bars: 10-segment bars using `█` (filled, white) and `░` (empty, `#333`)

### 10.3 Expanded View (Tab Toggle)

Position: left side of screen, 16px from top, full height minus 32px, width 380px.

```
┌────────────────────────────────────────┐
│  CONSCIOUSNESS SIMULATION              │
│  personality: contemplative            │
│  tick: 1042     circadian: 0.91        │
│                                        │
│  ─── ASPECT WEIGHTS ───                │
│  ● body_awareness         +0.34  ████  │
│  ● emotional_awareness    -0.12  ██    │
│  ● introspection          +0.67  ██████│
│  ... (all aspects listed)              │
│                                        │
│  ─── ENERGY ───                        │
│  energy    82%  ████████░░             │
│  arousal   0.71                        │
│  stress    0.12                        │
│  circadian 0.91                        │
│  flow: drive                           │
│  attended: agency, motivation, ...     │
│                                        │
│  ─── ENVIRONMENT ───                   │
│  valence: +0.23                        │
│  active: social_interaction            │
│  entropy: 4.12 (active)               │
│                                        │
│  ─── BEHAVIOR ───                      │
│  primary: idle_stand (0.48)            │
│  idle_fidget    0.22  ██               │
│  pacing         0.15  █               │
│  sitting_think  0.10  █               │
│  ...                                   │
│                                        │
│  ─── MEMORY ───                        │
│  STM buffer: 42/50                     │
│  LTM count: 8/500                      │
│  strongest: excited (str=2.1)          │
└────────────────────────────────────────┘
```

**Styling:**
- Same background and font as minimal overlay
- Scrollable content area
- Aspect weights colored by category (using `categoryColors` from init)
- Bar next to each weight: 1px per 0.1 magnitude, max 10 segments
- Category headers with colored left border (4px)

### 10.4 Keyboard Controls

| Key | Action |
|---|---|
| `Tab` | Toggle HUD expanded/minimal |
| `ArrowUp` | Send `{"type": "input", "direction": "positive"}` to Python (held = sustained) |
| `ArrowDown` | Send `{"type": "input", "direction": "negative"}` to Python (held = sustained) |
| (release arrow) | Send `{"type": "input", "direction": "none"}` to Python |

Input events are captured in the renderer process and forwarded via IPC to the main process, which writes to Python's stdin.

---

## 11. File-by-File Specification

### 11.1 `package.json`

- **Purpose:** Project manifest and dependency declaration
- **Key fields:** `"main": "main.js"`, `"type": "module"` is NOT set (use CommonJS for Electron main)
- **Scripts:** `"start": "electron ."`, `"dev": "electron . --dev"`

### 11.2 `main.js`

- **Purpose:** Electron main process — creates window, spawns Python, routes IPC
- **Exports:** None (entry point)
- **Dependencies:** `electron` (app, BrowserWindow, ipcMain), `child_process` (spawn), `path`
- **Key implementation notes:**
  - On `app.whenReady()`: create `BrowserWindow` with `width: 1280, height: 800`, `webPreferences: { preload: './preload.js', contextIsolation: true, nodeIntegration: false }`
  - Spawn Python: `spawn('python3', ['bridge.py', ...args], { cwd: __dirname })`. Pipe stdout line-buffered.
  - Read stdout line by line. Each complete JSON line is forwarded to the renderer via `mainWindow.webContents.send('python-message', parsedJSON)`.
  - Listen for `ipcMain.on('send-to-python', (event, msg) => { python.stdin.write(JSON.stringify(msg) + '\n'); })`.
  - On window close: send SIGTERM to Python child, then `app.quit()`.
  - Background: `#0D1117`

### 11.3 `preload.js`

- **Purpose:** Secure bridge between main and renderer via contextBridge
- **Exports:** `window.api.onPythonMessage(callback)`, `window.api.sendToPython(obj)`
- **Dependencies:** `electron` (contextBridge, ipcRenderer)
- **Key implementation notes:**
  ```javascript
  contextBridge.exposeInMainWorld('api', {
    onPythonMessage: (cb) => ipcRenderer.on('python-message', (event, data) => cb(data)),
    sendToPython: (obj) => ipcRenderer.send('send-to-python', obj),
  });
  ```

### 11.4 `index.html`

- **Purpose:** Single HTML page hosting the Three.js canvas and HUD overlay
- **Dependencies:** Three.js via importmap
- **Key implementation notes:**
  - `<canvas id="viewport">` fills the entire window
  - `<div id="hud">` positioned absolutely over the canvas
  - `<div id="hud-mini">` and `<div id="hud-detail">` inside `#hud`
  - Import `src/app.js` as module entry point
  - Importmap: `{ "imports": { "three": "./node_modules/three/build/three.module.js" } }`
  - Body background: `#0D1117`, margin 0, overflow hidden

### 11.5 `bridge.py`

- **Purpose:** Python-side IPC bridge wrapping ConsciousnessEngine
- **Exports:** Main script (no importable API)
- **Dependencies:** `sys`, `json`, `time`, `threading`, `argparse`, `core` package
- **Key implementation notes:**
  - Runs a tick loop on the main thread with `time.sleep()` maintaining 50ms cadence
  - Stdin reader runs in a daemon thread, parsing JSON lines and setting shared state (`input_direction`)
  - Each tick: call `engine.step()`, compute behavior scores (Section 3.2), serialize full state, write JSON line to stdout, `sys.stdout.flush()`
  - Behavior scoring uses the `weight_dict` from `step()` result plus `energy.get_state()` for stress
  - Track `prev_stress` for startle detection (stress spike = `max(0, stress - prev_stress)`)
  - Hysteresis: maintain `current_behavior` string; only switch if new best exceeds current score by >= 0.12
  - On stdin `{"type": "input", "direction": "positive"}`: set internal flag, call `engine.environment.apply_input(weight_dict, {'active': True, 'direction': 'positive', 'text': 'positive input'})` on next tick
  - Startup: print init message (Section 3.1) immediately before entering tick loop
  - Handle SIGTERM gracefully (print `{"type": "shutdown"}` and exit)

### 11.6 `src/app.js`

- **Purpose:** Application entry point — initializes scene, starts render loop, connects IPC
- **Exports:** None (side-effect module)
- **Dependencies:** `three`, `./ipc.js`, `./scene/room.js`, `./scene/lighting.js`, `./character/model.js`, `./animation/animator.js`, `./animation/locomotion.js`, `./behavior/stateMachine.js`, `./environment/events.js`, `./state/store.js`, `./state/interpolator.js`, `./hud/hud.js`
- **Key implementation notes:**
  - Create `WebGLRenderer` attached to `#viewport` canvas, `antialias: true`, `toneMapping: THREE.ACESFilmicToneMapping`, `toneMappingExposure: 1.0`, `shadowMap.enabled = true`, `shadowMap.type = THREE.PCFSoftShadowMap`
  - Set pixel ratio to `Math.min(window.devicePixelRatio, 2)`
  - Initialize scene with `room.createRoom(scene)`, `model.createCharacter(scene)`, `lighting.createLights(scene)`
  - Register IPC handler: `window.api.onPythonMessage(msg => store.handleMessage(msg))`
  - Render loop via `renderer.setAnimationFrame(loop)`:
    1. `interpolator.interpolate(store, performance.now())`
    2. `stateMachine.update(store, dt)`
    3. `locomotion.update(store, dt)`
    4. `animator.update(store, dt)`
    5. `lighting.update(store)`
    6. `events.update(scene, store, dt)`
    7. `hud.update(store)`
    8. `renderer.render(scene, camera)`
  - Handle window resize: update camera aspect and renderer size
  - Keyboard listeners for Tab and Arrow keys

### 11.7 `src/ipc.js`

- **Purpose:** Wrapper around `window.api` for sending messages to Python
- **Exports:** `sendInput(direction)`, `sendConfig(obj)`
- **Dependencies:** None (uses `window.api` from preload)
- **Key implementation notes:**
  - `sendInput('positive' | 'negative' | 'none')` → `window.api.sendToPython({ type: 'input', direction })`
  - Debounce: don't send duplicate `direction` values within 30ms

### 11.8 `src/scene/room.js`

- **Purpose:** Creates room geometry and furniture
- **Exports:** `createRoom(scene)` → returns `{ floor, walls, ceiling, chair, table }`
- **Dependencies:** `three`
- **Key implementation notes:**
  - Floor: `PlaneGeometry(8, 6)`, rotated -90 deg on X, position (0, 0, 0)
  - Back wall: `PlaneGeometry(8, 3.5)`, position (0, 1.75, -3)
  - Left wall: `PlaneGeometry(6, 3.5)`, rotated 90 deg on Y, position (-4, 1.75, 0)
  - Right wall: mirror of left at X=4
  - Ceiling: `PlaneGeometry(8, 6)`, rotated 90 deg on X, position (0, 3.5, 0)
  - All surfaces receive shadows
  - Chair and table as described in Section 4.1
  - Chair is interactive target for sitting behaviors — export its position as `CHAIR_POS = { x: 2.5, y: 0, z: -1.5 }`

### 11.9 `src/scene/lighting.js`

- **Purpose:** Creates static lights and updates dynamic lighting per frame
- **Exports:** `createLights(scene)` → returns light controller object, `update(store)` method
- **Dependencies:** `three`
- **Key implementation notes:**
  - Creates all lights listed in Section 4.3
  - `update(store)` reads `store.interpolated` and adjusts lights per the rules in Section 4.3
  - Stress pulse light is created once but with intensity 0, updated each frame
  - Flow spotlight created once but with intensity 0, enabled when flow is active
  - HSL interpolation for valence-based key light color:
    - Neutral: H=38, S=40%, L=55% (`#c8b8a0`)
    - Positive (valence=1): H=35, S=60%, L=55% (`#d4a050`)
    - Negative (valence=-1): H=220, S=50%, L=45% (`#4060a0`)
    - Interpolate H, S, L linearly based on valence

### 11.10 `src/character/model.js`

- **Purpose:** Creates the character mesh hierarchy
- **Exports:** `createCharacter(scene)` → returns `{ joints, meshes, root }` where `joints` is a dict of joint-name → `THREE.Group`
- **Dependencies:** `three`, `./materials.js`, `./skeleton.js`
- **Key implementation notes:**
  - Builds the hierarchy described in Section 5.2
  - Each mesh is added as a child of its parent joint group with appropriate local position offset
  - Mesh geometry centers are offset so rotation happens at the joint pivot
  - For example, `upperArmL` mesh has geometry centered at (0, -0.14, 0) so rotation at the shoulder swings the arm from the top
  - Shadow casting enabled on all meshes: `mesh.castShadow = true`
  - Returns joint references for the animator to manipulate

### 11.11 `src/character/materials.js`

- **Purpose:** Defines and exports character materials
- **Exports:** `CLOTHING_MAT`, `SKIN_MAT`, `SHOE_MAT`, `updateDominantCategory(category, dominance)`
- **Dependencies:** `three`
- **Key implementation notes:**
  - Materials are created once and shared across meshes
  - `updateDominantCategory()` sets `CLOTHING_MAT.emissive` to the category color and `CLOTHING_MAT.emissiveIntensity` to `0.05 * dominance`

### 11.12 `src/character/skeleton.js`

- **Purpose:** Joint name constants, rest pose angles, angle limits
- **Exports:** `JOINTS` (array of joint names), `REST_POSE` (dict of joint → {rotX, rotY, rotZ}), `ANGLE_LIMITS` (dict of joint → {minX, maxX, ...})
- **Dependencies:** None
- **Key implementation notes:**
  - `JOINTS`: `['pelvis', 'torso', 'neck', 'head', 'shoulderL', 'shoulderR', 'upperArmL', 'upperArmR', 'elbowL', 'elbowR', 'handL', 'handR', 'hipL', 'hipR', 'kneeL', 'kneeR', 'footL', 'footR']`
  - `REST_POSE`: all rotations 0 except `kneeL.rotX = 0.02`, `kneeR.rotX = 0.02`, `elbowL.rotX = 0.1`, `elbowR.rotX = 0.1`, `shoulderL.rotZ = 0.05`, `shoulderR.rotZ = -0.05`
  - Elbow rotX limited to `[0, 2.6]`, knee rotX limited to `[0, 2.5]`

### 11.13 `src/animation/animator.js`

- **Purpose:** Master animation controller — blends between poses each frame
- **Exports:** `class Animator` with `update(store, dt)` method
- **Dependencies:** `./procedural.js`, `../character/skeleton.js`
- **Key implementation notes:**
  - Maintains `currentPose` — current joint angles being rendered
  - Each frame: get target pose from `procedural.getPose(behavior, intensity, t)`, lerp `currentPose` toward it
  - Applies final angles to the joint groups: `joints[name].rotation.x = currentPose[name].rotX`, etc.
  - `blendRate` defaults to 6.0, increased to 12.0 when current behavior is `startle`
  - Lerp formula: `current += (target - current) * (1 - Math.exp(-blendRate * dt))`  (exponential smoothing, framerate-independent)

### 11.14 `src/animation/procedural.js`

- **Purpose:** Procedural joint-angle formulas for each behavior
- **Exports:** `getPose(behaviorName, intensity, t, phase)` → dict of joint → {rotX, rotY, rotZ}
- **Dependencies:** `../character/skeleton.js` (for REST_POSE as defaults)
- **Key implementation notes:**
  - Implements all formulas from Section 6.2
  - `phase` parameter only used by `pacing` (walk cycle phase from locomotion)
  - Returns a complete pose dict; unspecified joints default to REST_POSE values
  - For `startle`: tracks `startleStartTime` internally, computes `startleT = t - startleStartTime`, resets when startle behavior ends

### 11.15 `src/animation/locomotion.js`

- **Purpose:** Moves the character root to target positions, provides walk phase
- **Exports:** `class Locomotion` with `update(store, dt)` and `walkPhase` getter
- **Dependencies:** `../scene/room.js` (for CHAIR_POS)
- **Key implementation notes:**
  - Maintains `currentTarget`, `waypoints` (array of intermediate points), `walkPhase` (0 to 2*PI, cycling)
  - When the state machine sets a new target position, locomotion computes waypoints (Section 6.3)
  - Each frame while walking: move `store.character.position` toward next waypoint at the computed speed, increment `walkPhase` by `freq * 2 * PI * dt`
  - When distance to final target < 0.2m, signal arrival (state machine transitions to PERFORMING)
  - `walkPhase` is read by `procedural.js` for the walk cycle

### 11.16 `src/behavior/scorer.js`

- **Purpose:** Reads behavior data from Python tick (no local computation)
- **Exports:** `readBehavior(tickData)` → `{ primary, scores, intensity }`
- **Dependencies:** None
- **Key implementation notes:**
  - Simply extracts `tickData.behavior` and returns it
  - Validates that `primary` is a known behavior name
  - If `tickData.behavior` is missing (e.g., first tick before bridge is ready), returns `{ primary: 'idle_stand', scores: {}, intensity: 0.0 }`

### 11.17 `src/behavior/stateMachine.js`

- **Purpose:** Three-state FSM managing character state transitions
- **Exports:** `class StateMachine` with `update(store, dt)`
- **Dependencies:** `./behaviors.js`, `../animation/locomotion.js`, `../scene/room.js`
- **Key implementation notes:**
  - States: `IDLE`, `WALKING`, `PERFORMING`
  - `update()` reads `store.behavior.current`, determines if a position change is needed, transitions accordingly (Section 7.2)
  - Behaviors that require the chair: `sitting_think`, `sitting_slump`
  - Behaviors that stay in place: everything else
  - When entering WALKING: set locomotion target, locomotion handles the pathing
  - When entering PERFORMING: notify animator to play the behavior animation

### 11.18 `src/behavior/behaviors.js`

- **Purpose:** Behavior metadata — name, required position, type
- **Exports:** `BEHAVIORS` dict mapping behavior name to `{ type: 'idle'|'seated'|'standing', requiresChair: bool, isTransient: bool }`
- **Dependencies:** None
- **Key implementation notes:**
  ```javascript
  export const BEHAVIORS = {
    idle_stand:       { type: 'standing', requiresChair: false, isTransient: false },
    idle_fidget:      { type: 'standing', requiresChair: false, isTransient: false },
    pacing:           { type: 'standing', requiresChair: false, isTransient: false },
    sitting_think:    { type: 'seated',   requiresChair: true,  isTransient: false },
    sitting_slump:    { type: 'seated',   requiresChair: true,  isTransient: false },
    gesture_emphatic: { type: 'standing', requiresChair: false, isTransient: false },
    gesture_dismiss:  { type: 'standing', requiresChair: false, isTransient: true  },
    startle:          { type: 'standing', requiresChair: false, isTransient: true  },
  };
  ```

### 11.19 `src/environment/events.js`

- **Purpose:** Manages 3D visual effects for stimuli and internal events
- **Exports:** `class EventManager` with `update(scene, store, dt)` and `spawnEvent(name, scene, characterPos)`
- **Dependencies:** `three`, `./particles.js`
- **Key implementation notes:**
  - Maintains `activeEffects` array of `{ name, meshes, lights, startTime, duration, phase, update(dt) }`
  - On each tick: diff `store.latestTick.activeStimuli` against `store.previousTick.activeStimuli` to find new stimuli
  - For each new stimulus name, call `spawnEvent(name, scene, characterPos)` which creates the meshes/lights described in Section 8.1/8.2
  - Each effect has an `update(dt)` method that advances its lifecycle (onset/peak/decay), adjusting opacity, scale, position
  - When an effect finishes (total time elapsed > duration), remove its meshes from scene and splice from `activeEffects`
  - Maximum 10 concurrent effects — if limit reached, remove oldest

### 11.20 `src/environment/particles.js`

- **Purpose:** Simple particle system for burst/drift effects
- **Exports:** `createParticleBurst(options)` → `{ group, update(dt), isDone }`
- **Dependencies:** `three`
- **Key implementation notes:**
  - `options`: `{ count, color, size, position, velocity, gravity, lifetime, spread }`
  - Each particle is a `Mesh` with `SphereGeometry(size)` and `MeshBasicMaterial({ color, transparent: true })`
  - Particles are children of a `Group`
  - `update(dt)`: move each particle by its velocity, apply gravity, reduce opacity linearly, remove when opacity <= 0
  - `isDone`: true when all particles have expired
  - Defaults: `count=20, size=0.02, gravity=-1.5, lifetime=2.0, spread=1.0`

### 11.21 `src/state/store.js`

- **Purpose:** Central reactive state container
- **Exports:** `class Store` with `handleMessage(msg)`, `getState()`, and all fields from Section 9.1
- **Dependencies:** `../behavior/scorer.js`
- **Key implementation notes:**
  - `handleMessage(msg)`: switch on `msg.type`:
    - `"init"`: store init data (aspects, categories, etc.)
    - `"tick"`: shift `latestTick → previousTick`, set `latestTick = msg`, extract behavior
  - No pub/sub needed — the render loop reads state directly each frame

### 11.22 `src/state/interpolator.js`

- **Purpose:** Smooth 20Hz→60fps interpolation
- **Exports:** `interpolate(store, timestamp)`
- **Dependencies:** None
- **Key implementation notes:**
  - Implements the algorithm from Section 9.2
  - Tracks `latestTickReceivedAt` timestamp (set when a new tick arrives)
  - Clamps alpha to [0, 1] to avoid extrapolation artifacts

### 11.23 `src/hud/hud.js`

- **Purpose:** Root HUD controller — manages mini and detail panels
- **Exports:** `class HUD` with `update(store)`, `toggle()`
- **Dependencies:** `./miniPanel.js`, `./detailPanel.js`
- **Key implementation notes:**
  - Creates DOM elements on construction
  - `update(store)`: calls `miniPanel.update(store)` and (if expanded) `detailPanel.update(store)`
  - `toggle()`: flips `store.hudExpanded`, shows/hides detail panel
  - Updates are throttled to 10 Hz (every 100ms) to avoid DOM thrash — the HUD does not need 60fps updates

### 11.24 `src/hud/miniPanel.js`

- **Purpose:** Compact always-visible overlay
- **Exports:** `class MiniPanel` with `update(store)`, `element` (DOM node)
- **Dependencies:** None (pure DOM manipulation)
- **Key implementation notes:**
  - Creates a `<div>` with the layout from Section 10.2
  - Uses `textContent` updates (no innerHTML for security)
  - Mental state detection for the HUD label — re-implement the simple heuristic from `visualization.py`'s `detect_mental_state` in JS:
    ```javascript
    function detectMentalState(weights, stress, avgDelta) {
      const vals = Object.values(weights);
      const std = standardDeviation(vals);
      const mean = vals.reduce((s, v) => s + Math.abs(v), 0) / vals.length;
      if (std > 0.45 && avgDelta > 0.02) return { label: 'AGITATED', color: '#E8613C' };
      if (mean > 0.3 && avgDelta > 0.005) return { label: 'GROWING', color: '#3CB371' };
      if (avgDelta < -0.005) return { label: 'RECOVERING', color: '#4A90D9' };
      if (std > 0.35) return { label: 'CONFLICTED', color: '#9B59B6' };
      return { label: 'CALM', color: '#DAA520' };
    }
    ```
  - Energy bar: 10 characters, each representing 10%. `filled = Math.round(energyPct / 10)`

### 11.25 `src/hud/detailPanel.js`

- **Purpose:** Expanded data view with all simulation details
- **Exports:** `class DetailPanel` with `update(store)`, `element` (DOM node)
- **Dependencies:** None (pure DOM manipulation)
- **Key implementation notes:**
  - Creates sections as described in Section 10.3
  - Aspect list sorted by absolute weight descending
  - Each aspect row: colored dot (category color), name (left-aligned, 22ch), value (right-aligned, +/-0.00), small bar
  - Bar width: `Math.round(Math.abs(weight) * 10)` characters, max 10
  - Behavior scores sorted descending, with primary highlighted
  - Memory section shows STM fill, LTM count (from tick data if available, otherwise "N/A")
  - Scroll container with `overflow-y: auto`, `max-height: calc(100vh - 64px)`
