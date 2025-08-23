# Audio Assets for Space Looter

This document lists all the audio assets needed for the Space Looter game, organized by category and event type.

## Music Assets (OGG Vorbis format recommended)

### Background Music (`music/`)
- `ambient_space.ogg` - Main ambient space exploration music
- `menu_theme.ogg` - Main menu and UI music
- `tension_discovery.ogg` - Discovery/exploration tension music
- `combat_encounter.ogg` - Combat and hostile encounter music
- `peaceful_rest.ogg` - Resting and recovery music
- `mystery_ambient.ogg` - Mysterious events and anomalies
- `victory_success.ogg` - Success and achievement music

### Adaptive Music Layers
- `base_layer.ogg` - Base ambient layer
- `tension_layer.ogg` - Tension overlay
- `discovery_layer.ogg` - Discovery overlay
- `danger_layer.ogg` - Danger/combat overlay

## Sound Effects (WAV format recommended)

### Movement (`sfx/movement/`)
- `footstep_metal.wav` - Walking on metal surfaces
- `footstep_rock.wav` - Walking on rocky terrain
- `footstep_sand.wav` - Walking on sandy/dusty terrain
- `movement_success.wav` - Successful movement
- `movement_blocked.wav` - Blocked/failed movement
- `teleport_enter.wav` - Entering teleporter
- `teleport_exit.wav` - Exiting teleporter

### Dice & Mechanics (`sfx/dice/`)
- `dice_roll.wav` - Dice rolling sound
- `dice_critical_success.wav` - Critical success (20)
- `dice_critical_failure.wav` - Critical failure (1)
- `dice_high_roll.wav` - High success (15-19)
- `dice_low_roll.wav` - Low failure (2-5)

### Events (`sfx/events/`)
#### Resource Discovery
- `resource_found.wav` - General resource discovery
- `rare_resource.wav` - Rare resource found
- `crystal_chime.wav` - Crystal/energy resource
- `metal_clank.wav` - Metal resource
- `organic_squelch.wav` - Organic resource

#### Combat & Encounters
- `enemy_approach.wav` - Hostile encounter start
- `combat_hit.wav` - Combat hit/damage
- `combat_miss.wav` - Combat miss
- `enemy_defeat.wav` - Enemy defeated
- `player_damage.wav` - Player takes damage

#### Environmental
- `wind_howl.wav` - Environmental wind/atmosphere
- `energy_hum.wav` - Energy anomaly
- `machinery_whir.wav` - Ancient machinery
- `cave_echo.wav` - Cave/underground areas
- `space_silence.wav` - Deep space ambience

#### Rest & Recovery
- `rest_start.wav` - Beginning rest period
- `rest_complete.wav` - Rest completed successfully
- `sleep_disturbed.wav` - Rest interrupted/poor sleep
- `health_restore.wav` - Health/stamina restored

### UI Sounds (`sfx/ui/`)
- `button_click.wav` - UI button press
- `button_hover.wav` - UI button hover
- `menu_open.wav` - Menu/panel open
- `menu_close.wav` - Menu/panel close
- `notification.wav` - General notification
- `warning.wav` - Warning notification
- `error.wav` - Error notification
- `achievement.wav` - Achievement unlocked
- `level_up.wav` - Experience/level progression

### Resource Management (`sfx/resources/`)
- `resource_gain.wav` - Resources gained
- `resource_loss.wav` - Resources lost/consumed
- `inventory_full.wav` - Inventory full warning
- `craft_success.wav` - Crafting success
- `trade_complete.wav` - Trading transaction

## Audio Sources & Recommendations

### Free/Open Source
- **Freesound.org** - Large collection of CC-licensed sounds
- **OpenGameArt.org** - Game-specific audio assets
- **Zapsplat.com** - Professional sound effects (free tier)
- **BBC Sound Effects Library** - High-quality effects

### Music Sources
- **Incompetech.com** - Kevin MacLeod's royalty-free music
- **OpenGameArt.org** - Game music tracks
- **Jamendo.com** - Creative Commons music
- **Freemusicarchive.org** - Open source music

### AI-Generated Options
- **AIVA** - AI music composition for games
- **Mubert** - AI ambient music generation
- **Boomy** - AI music creation platform

## Technical Specifications

### Music Files
- **Format**: OGG Vorbis (.ogg)
- **Sample Rate**: 44.1 kHz
- **Bitrate**: 128-192 kbps (balance between quality and file size)
- **Channels**: Stereo (2.0)
- **Length**:
  - Ambient tracks: 2-4 minutes (loopable)
  - Event music: 30-60 seconds
  - Menu music: 1-2 minutes (loopable)

### Sound Effects
- **Format**: WAV (.wav) for short effects, OGG for longer ambient
- **Sample Rate**: 44.1 kHz
- **Bit Depth**: 16-bit
- **Channels**: Mono for most SFX, Stereo for ambient
- **Length**: 0.1-3 seconds for most effects

## Implementation Notes

### Event Mapping
Each audio file will be triggered by specific game events:

1. **Movement Events** → Movement sounds + ambient music changes
2. **Discovery Events** → Discovery SFX + music layer changes
3. **Combat Events** → Combat music + battle SFX
4. **Rest Events** → Peaceful music + rest SFX
5. **UI Events** → UI feedback sounds
6. **Dice Rolls** → Dice SFX based on roll results

### Volume Levels
- **Music**: 70% default volume
- **SFX**: 80% default volume
- **UI Sounds**: 60% default volume
- **Ambient**: 50% default volume

### Looping
- Ambient music should loop seamlessly
- Event music should have clear start/end points
- SFX should be one-shot unless specified otherwise

## Asset Status

- [ ] Music assets sourced
- [ ] Movement SFX sourced
- [ ] Dice SFX sourced
- [ ] Event SFX sourced
- [ ] UI SFX sourced
- [ ] All assets converted to proper format
- [ ] Audio system implemented
- [ ] Event triggers connected
- [ ] Volume balancing completed
- [ ] Testing completed

## Quick Start Downloads

For immediate testing, download these essential sounds first:
1. Basic dice roll sound
2. Movement success/failure sounds
3. Simple ambient background track
4. Resource discovery chime
5. UI click sound

This will provide a foundation for the audio system while you source additional assets.
