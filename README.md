# Orchestrator Module

## Overview

The **Orchestrator** is the central controller of the simulation.
It manages planets, explorers, the Galaxy AI, GUI interaction, and the overall simulation lifecycle.

All entities communicate exclusively via message passing using channels.
The orchestrator does **not** perform planet or explorer logic itself. Instead, it coordinates and routes messages, maintains global state, and enforces simulation rules.

---

## Responsibilities

The orchestrator is responsible for:

* Managing the lifecycle of **planets** and **explorers**
* Maintaining the global **SystemState**
* Routing messages between:

  * Planets
  * Explorers
  * GUI
* Running the **Galaxy AI**
* Sending global events (asteroids, sunrays)
* Updating the GUI with the current simulation state
* Controlling simulation speed, pause, and shutdown

---

## Architecture

### Communication Model

The orchestrator uses **channel-based communication**:

* **Planet channels**

  * `OrchestratorToPlanet`
  * `PlanetToOrchestrator`
* **Explorer channels**

  * `OrchestratorToExplorer`
  * `ExplorerToOrchestrator`
* **GUI channels**

  * `GuiEvent` (orchestrator → GUI)
  * `GuiCommand` (GUI → orchestrator)

The orchestrator polls all incoming channels in each simulation cycle.

---

## Core Components

### `Orchestrator`

Main struct managing the simulation.

Key fields:

* `forge`
  Generates asteroids and sunrays.

* `planet_senders / planet_receivers`
  Communication with planets.

* `explorer_senders / explorer_receivers`
  Communication with explorers.

* `state: SystemState`
  Stores all authoritative game data (planets, explorers, statistics, game state).

* `galaxy_ai: GalaxyAI`
  Controls high-level galaxy behavior.

* `gui_event_sender / gui_command_receiver`
  Interface to the GUI.

* `should_stop`
  Atomic flag for graceful shutdown.

* `cycle_duration_in_millis`
  Controls simulation speed.

---

## Initialization

### `new()`

Creates an empty orchestrator with:

* No planets
* No explorers
* Galaxy AI inactive
* Default simulation cycle length (500 ms)

### `new_with_parameters(...)`

Creates an orchestrator using:

* Predefined planet and explorer channels
* A galaxy structure file
* Active Galaxy AI

Used when loading a predefined simulation.

---

## Entity Management

### Planets

* `add_planet(...)`
  Registers a planet and updates system state.

* `destroy_planet(planet_id)`

  * Kills all explorers on the planet
  * Sends `KillPlanet`
  * Removes the planet from state
  * Notifies the GUI

### Explorers

* `add_explorer(...)`
  Registers an explorer on a living planet.

* `kill_explorer(explorer_id)`
  Sends `KillExplorer`, removes it from state, and updates the GUI.

---

## Main Loop

### `run()`

The orchestrator runs a continuous loop until:

* The game ends, or
* A stop is requested

Each cycle performs:

1. Message polling (planets, explorers, GUI)
2. Periodic maintenance
3. Galaxy AI decision-making
4. GUI state updates
5. Controlled sleep to avoid busy-waiting

Simulation can be paused and resumed via GUI commands.

---

## Galaxy AI

The **Galaxy AI** controls large-scale actions.

Supported actions:

* Send asteroid to a planet
* Send sunray to a planet
* Do nothing

Methods:

* `enable_galaxy_ai()`
* `disable_galaxy_ai()`
* `set_galaxy_ai_parameters(...)`
* `run_galaxy_ai()`

The AI operates only when enabled and the simulation is running.

---

## GUI Integration

### Events (Orchestrator → GUI)

Examples:

* Planet added / removed
* Explorer added / removed
* Asteroid or sunray sent
* Periodic state updates

### Commands (GUI → Orchestrator)

Supported commands include:

* Pause / resume simulation
* Send asteroid or sunray
* Toggle planet or explorer AI
* Adjust simulation speed
* Configure Galaxy AI parameters

---

## Game Actions

* `send_asteroid_to_planet(planet_id)`
* `send_sunray_to_planet(planet_id)`
* `toggle_planet_ai(planet_id, enabled)`
* `toggle_explorer_ai(explorer_id, enabled)`

All actions validate entity state before execution.

---

## Shutdown

### `stop()`

Gracefully shuts down the simulation by:

* Disabling Galaxy AI
* Sending kill commands to all planets and explorers
* Setting the stop flag

---

## Threading Model

* Single-threaded orchestrator loop
* Non-blocking message polling using `try_recv`
* Controlled sleep between cycles
* No shared mutable state outside the orchestrator

---

## Summary

The orchestrator acts as:

* The **authority** over game state
* The **router** for all inter-entity communication
* The **scheduler** of simulation logic
* The **bridge** between AI, entities, and GUI


---
