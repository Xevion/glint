package com.xevion.glint.orchestration

import com.xevion.glint.Loggers
import net.minecraft.client.Minecraft
import net.minecraft.client.gui.screens.AlertScreen
import net.minecraft.server.level.ServerLevel
import net.minecraft.world.Difficulty
import net.minecraft.world.flag.FeatureFlags
import net.minecraft.world.level.GameRules
import net.minecraft.world.level.GameType
import net.minecraft.world.level.LevelSettings
import net.minecraft.world.level.WorldDataConfiguration
import net.minecraft.world.level.levelgen.WorldOptions
import net.minecraft.world.level.levelgen.presets.WorldPresets

/**
 * Manages a minimal void superflat world used as the base for scene injection.
 *
 * The staging world exists solely to provide a [ServerLevel] instance. The
 * [ChunkStorageMixin][com.xevion.glint.mixin.ChunkStorageMixin] intercepts all
 * chunk reads when active, so the world's own terrain is irrelevant — it just
 * needs to boot Minecraft's integrated server.
 *
 * Created once in `saves/glint-staging/` and reused across capture sessions.
 * Uses a void flat preset with no structures to minimize generation overhead.
 *
 * Tick-driven: call [tick] each client tick until [state] reaches [State.READY]
 * or [State.FAILED].
 */
class StagingWorld {
    private val log = Loggers.Orchestration.get()

    var state = State.IDLE
        private set
    var error: String? = null
        private set

    private var createRequested = false
    private var loadRequested = false
    private var ticksWaited = 0

    enum class State {
        /** Not started. Call [ensureReady] then [tick] each frame. */
        IDLE,

        /** Unloading any currently loaded world before proceeding. */
        UNLOADING,

        /** Creating the staging world via [WorldOpenFlows.createFreshLevel]. */
        CREATING,

        /** Loading an existing staging world via [WorldOpenFlows.openWorld]. */
        LOADING,

        /** The staging world is loaded and [getServerLevel] is available. */
        READY,

        /** An unrecoverable error occurred. See [error]. */
        FAILED,
    }

    /**
     * Begin the staging world load/create process.
     * After calling this, call [tick] every client tick until terminal.
     */
    fun ensureReady() {
        if (state != State.IDLE) return
        state = State.UNLOADING
        ticksWaited = 0
        createRequested = false
        loadRequested = false
        log.info("Starting staging world preparation")
    }

    /**
     * Advance the state machine one step. Call once per client tick.
     * Returns `true` when terminal ([State.READY] or [State.FAILED]).
     */
    fun tick(): Boolean {
        val mc = Minecraft.getInstance()

        when (state) {
            State.UNLOADING -> {
                tickUnloading(mc)
            }

            State.CREATING -> {
                tickCreating(mc)
            }

            State.LOADING -> {
                tickLoading(mc)
            }

            State.IDLE, State.READY, State.FAILED -> {}
        }

        return state == State.READY || state == State.FAILED
    }

    private fun tickUnloading(mc: Minecraft) {
        // If already at title screen (no world loaded), skip unloading
        if (mc.level == null && mc.singleplayerServer == null) {
            transitionToCreateOrLoad(mc)
            return
        }

        // If already in the staging world, we're done
        if (isInStagingWorld(mc)) {
            state = State.READY
            log.info("Already in staging world")
            return
        }

        // Request disconnect once
        if (ticksWaited == 0) {
            log.info("Unloading current world before staging world setup")
            mc.level?.disconnect()
        }

        ticksWaited++
        if (ticksWaited > TIMEOUT_TICKS) {
            fail("Timed out unloading current world")
            return
        }

        // Wait until we're back at the title screen
        if (mc.level == null && mc.singleplayerServer == null) {
            log.info("Current world unloaded")
            transitionToCreateOrLoad(mc)
        }
    }

    private fun transitionToCreateOrLoad(mc: Minecraft) {
        ticksWaited = 0
        if (mc.getLevelSource().levelExists(FOLDER_NAME)) {
            state = State.LOADING
            log.info("Staging world exists, loading")
        } else {
            state = State.CREATING
            log.info("Staging world does not exist, creating")
        }
    }

    private fun tickCreating(mc: Minecraft) {
        if (!createRequested) {
            runCatching {
                val settings =
                    LevelSettings(
                        WORLD_NAME,
                        GameType.SPECTATOR,
                        false, // not hardcore
                        Difficulty.PEACEFUL,
                        true, // allow commands
                        GameRules(FeatureFlags.DEFAULT_FLAGS),
                        WorldDataConfiguration.DEFAULT,
                    )
                val worldOptions = WorldOptions(0L, false, false) // seed 0, no structures, no bonus chest
                mc.createWorldOpenFlows().createFreshLevel(
                    FOLDER_NAME,
                    settings,
                    worldOptions,
                    WorldPresets::createFlatWorldDimensions,
                    null, // no fallback screen — we handle errors ourselves
                )
            }.onFailure { e ->
                fail("Exception creating staging world: ${e.message}")
                log.error(e, "Failed to create staging world")
                return
            }
            createRequested = true
            ticksWaited = 0
            log.info("Staging world creation initiated")
        }

        // Wait for the integrated server to come up
        ticksWaited++
        if (mc.screen is AlertScreen) {
            fail("Minecraft showed error screen during staging world creation")
            return
        }
        if (ticksWaited > TIMEOUT_TICKS) {
            fail("Timed out waiting for staging world creation")
            return
        }
        if (isInStagingWorld(mc)) {
            state = State.READY
            log.info("Staging world created and loaded")
        }
    }

    private fun tickLoading(mc: Minecraft) {
        if (!loadRequested) {
            runCatching {
                mc.createWorldOpenFlows().openWorld(FOLDER_NAME) {
                    log.debug("Staging world load cancelled")
                }
            }.onFailure { e ->
                fail("Exception loading staging world: ${e.message}")
                log.error(e, "Failed to load staging world")
                return
            }
            loadRequested = true
            ticksWaited = 0
            log.info("Staging world load initiated")
        }

        ticksWaited++
        if (mc.screen is AlertScreen) {
            fail("Minecraft showed error screen during staging world load")
            return
        }
        if (ticksWaited > TIMEOUT_TICKS) {
            fail("Timed out waiting for staging world to load")
            return
        }
        if (isInStagingWorld(mc)) {
            state = State.READY
            log.info("Staging world loaded")
        }
    }

    /**
     * Get the overworld [ServerLevel] from the staging world.
     * Only valid when [state] is [State.READY].
     */
    fun getServerLevel(): ServerLevel? {
        val mc = Minecraft.getInstance()
        return mc.singleplayerServer?.overworld()
    }

    /**
     * Reset to [State.IDLE] for reuse across capture sessions.
     */
    fun reset() {
        state = State.IDLE
        error = null
        createRequested = false
        loadRequested = false
        ticksWaited = 0
    }

    private fun isInStagingWorld(mc: Minecraft): Boolean =
        mc.level != null &&
            mc.singleplayerServer != null &&
            mc.singleplayerServer?.worldData?.levelName == WORLD_NAME

    private fun fail(reason: String) {
        state = State.FAILED
        error = reason
        log.error("Staging world failed") { "reason" to reason }
    }

    companion object {
        /** Folder name under saves/ */
        const val FOLDER_NAME = "glint-staging"

        /** Display name shown in world list */
        private const val WORLD_NAME = "glint-staging"

        /** Timeout for each phase (creation, loading, unloading) — 10 seconds at 20 TPS */
        private const val TIMEOUT_TICKS = 200
    }
}
