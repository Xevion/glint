package com.xevion.glint.ui.base

import io.wispforest.owo.ui.base.BaseOwoScreen
import io.wispforest.owo.ui.component.Components
import io.wispforest.owo.ui.container.Containers
import io.wispforest.owo.ui.container.DraggableContainer
import io.wispforest.owo.ui.container.FlowLayout
import io.wispforest.owo.ui.core.Color
import io.wispforest.owo.ui.core.Component
import io.wispforest.owo.ui.core.HorizontalAlignment
import io.wispforest.owo.ui.core.Insets
import io.wispforest.owo.ui.core.OwoUIAdapter
import io.wispforest.owo.ui.core.Sizing
import io.wispforest.owo.ui.core.Surface
import io.wispforest.owo.ui.core.VerticalAlignment
import net.minecraft.client.Minecraft
import net.minecraft.util.Mth
import org.lwjgl.glfw.GLFW
import net.minecraft.network.chat.Component as McComponent

/**
 * Base screen for panels overlaid on the live game world.
 * No dimming overlay — the world renders behind the panel.
 * The panel is wrapped in a DraggableContainer and starts left-aligned.
 *
 * Dragging on the background (outside the panel) pans the camera
 * with reduced sensitivity for fine-tuning the view angle.
 * Scrolling on the background delegates to [onBackgroundScroll].
 */
abstract class GlintPanelScreen(
    title: McComponent,
) : BaseOwoScreen<FlowLayout>(title) {
    companion object {
        private const val FOREHEAD_HEIGHT = 14
        private const val GRIP_WIDTH = 30
        private const val GRIP_HEIGHT = 3

        private const val GRIP_COLOR = GlintTheme.GRIP_COLOR
        private const val PANEL_ALPHA_TRANSPARENT = GlintTheme.PANEL_BG_TRANSPARENT

        /** Roughly 1/3 of normal mouse look. */
        private const val CAMERA_SENSITIVITY = 0.15f

        private const val ICON_EYE_OPEN = "\u25C9"
        private const val ICON_EYE_CLOSED = "\u25CB"
    }

    private var panelTransparent = false
    private lateinit var panelContainer: FlowLayout
    private lateinit var draggable: DraggableContainer<FlowLayout>

    private var draggingBackground = false
    private var dragStartX = 0.0
    private var dragStartY = 0.0

    override fun createAdapter(): OwoUIAdapter<FlowLayout> = OwoUIAdapter.create(this, Containers::verticalFlow)

    /** Keep the server ticking so environment changes (time, weather) propagate immediately. */
    override fun isPauseScreen(): Boolean = false

    override fun build(rootComponent: FlowLayout) {
        rootComponent
            .surface(Surface.BLANK)
            .verticalAlignment(VerticalAlignment.CENTER)
            .horizontalAlignment(HorizontalAlignment.LEFT)

        panelContainer = Containers.verticalFlow(Sizing.content(), Sizing.content())
        panelContainer.horizontalAlignment(HorizontalAlignment.LEFT)
        panelContainer.padding(GlintTheme.paddingMd())
        panelContainer.gap(GlintTheme.GAP_MD)

        val titleBar = Containers.horizontalFlow(Sizing.content(), Sizing.content())
        titleBar.gap(GlintTheme.GAP_MD)
        titleBar.verticalAlignment(VerticalAlignment.CENTER)

        titleBar.child(
            Components
                .label(title)
                .color(Color.ofRgb(GlintTheme.TEXT_PRIMARY)),
        )

        val eyeButton =
            GlintComponents.iconButton(
                ICON_EYE_OPEN,
                tooltip = McComponent.literal("Toggle panel transparency"),
            ) { btn ->
                panelTransparent = !panelTransparent
                updatePanelSurface()
                btn.setMessage(McComponent.literal(if (panelTransparent) ICON_EYE_CLOSED else ICON_EYE_OPEN))
            }
        titleBar.child(eyeButton as Component)

        panelContainer.child(titleBar)

        buildPanel(panelContainer)

        draggable = Containers.draggable(Sizing.content(), Sizing.content(), panelContainer)
        draggable.foreheadSize(FOREHEAD_HEIGHT)
        draggable.zIndex(500)

        // Custom surface: panel background + centered grip bar in the forehead
        draggable.surface(panelSurface(false))

        draggable.margins(Insets.of(GlintTheme.PADDING_MD))

        rootComponent.child(draggable)
    }

    /**
     * Creates a surface that renders the panel background plus
     * a centered grip indicator in the forehead drag area.
     */
    private fun panelSurface(transparent: Boolean): Surface {
        val baseSurface = if (transparent) Surface.flat(PANEL_ALPHA_TRANSPARENT) else Surface.DARK_PANEL
        return Surface { context, component ->
            baseSurface.draw(context, component)
            val gripX = component.x() + (component.width() - GRIP_WIDTH) / 2
            val gripY = component.y() + (FOREHEAD_HEIGHT - GRIP_HEIGHT) / 2
            context.fill(gripX, gripY, gripX + GRIP_WIDTH, gripY + GRIP_HEIGHT, GRIP_COLOR)
        }
    }

    override fun mouseClicked(
        mouseX: Double,
        mouseY: Double,
        button: Int,
    ): Boolean {
        val onPanel = draggable.isInBoundingBox(mouseX, mouseY)
        val handled = super.mouseClicked(mouseX, mouseY, button)
        if (!onPanel && button == 0) {
            draggingBackground = true
            // Save physical cursor position for restoration on release
            val window = Minecraft.getInstance().window.window
            val xBuf = DoubleArray(1)
            val yBuf = DoubleArray(1)
            GLFW.glfwGetCursorPos(window, xBuf, yBuf)
            dragStartX = xBuf[0]
            dragStartY = yBuf[0]
            // DISABLED mode: hides cursor, provides unbounded virtual movement,
            // and restores cursor position when re-enabled
            GLFW.glfwSetInputMode(window, GLFW.GLFW_CURSOR, GLFW.GLFW_CURSOR_DISABLED)
        }
        return handled || draggingBackground
    }

    override fun mouseDragged(
        mouseX: Double,
        mouseY: Double,
        button: Int,
        deltaX: Double,
        deltaY: Double,
    ): Boolean {
        if (draggingBackground && button == 0) {
            val player = Minecraft.getInstance().player ?: return true
            player.setYRot(player.yRot + (deltaX * CAMERA_SENSITIVITY).toFloat())
            player.setXRot(Mth.clamp(player.xRot + (deltaY * CAMERA_SENSITIVITY).toFloat(), -90f, 90f))
            return true
        }
        return super.mouseDragged(mouseX, mouseY, button, deltaX, deltaY)
    }

    override fun mouseReleased(
        mouseX: Double,
        mouseY: Double,
        button: Int,
    ): Boolean {
        if (button == 0 && draggingBackground) {
            draggingBackground = false
            val window = Minecraft.getInstance().window.window
            GLFW.glfwSetInputMode(window, GLFW.GLFW_CURSOR, GLFW.GLFW_CURSOR_NORMAL)
            GLFW.glfwSetCursorPos(window, dragStartX, dragStartY)
            return true
        }
        return super.mouseReleased(mouseX, mouseY, button)
    }

    override fun mouseScrolled(
        mouseX: Double,
        mouseY: Double,
        horizontalAmount: Double,
        verticalAmount: Double,
    ): Boolean {
        val onPanel = draggable.isInBoundingBox(mouseX, mouseY)
        if (!onPanel && onBackgroundScroll(verticalAmount)) {
            return true
        }
        return super.mouseScrolled(mouseX, mouseY, horizontalAmount, verticalAmount)
    }

    /**
     * Called when the user scrolls on the background (outside the panel).
     * Override to handle scroll events (e.g. FOV adjustment).
     * @return true if the scroll was consumed
     */
    protected open fun onBackgroundScroll(amount: Double): Boolean = false

    /**
     * Build the panel content. The panel already has a title bar.
     * Add controls, inputs, and buttons here.
     */
    protected abstract fun buildPanel(panel: FlowLayout)

    /**
     * Called when the screen is closed for any reason (cancel, escape, setScreen, error).
     * Subclasses should restore any modified game state here.
     * Guaranteed to be called exactly once via [removed].
     */
    protected open fun onPanelClosed() {}

    private var panelClosedCalled = false

    override fun removed() {
        if (!panelClosedCalled) {
            panelClosedCalled = true
            onPanelClosed()
        }
        // Ensure cursor is restored if screen is removed mid-drag
        if (draggingBackground) {
            draggingBackground = false
            val window = Minecraft.getInstance().window.window
            GLFW.glfwSetInputMode(window, GLFW.GLFW_CURSOR, GLFW.GLFW_CURSOR_NORMAL)
            GLFW.glfwSetCursorPos(window, dragStartX, dragStartY)
        }
        super.removed()
    }

    private fun updatePanelSurface() {
        draggable.surface(panelSurface(panelTransparent))
    }
}
