package com.xevion.glint.ui.base

import io.wispforest.owo.ui.core.Insets
import io.wispforest.owo.ui.core.Sizing

/**
 * Theme constants for consistent UI styling across Glint screens.
 */
object GlintTheme {
    // Spacing constants
    const val PADDING_SM = 8
    const val PADDING_MD = 12
    const val PADDING_LG = 20
    const val GAP_SM = 4
    const val GAP_MD = 8
    const val GAP_LG = 12

    // Standard button dimensions
    const val BUTTON_WIDTH = 100
    const val BUTTON_HEIGHT = 20
    const val BUTTON_WIDTH_WIDE = 150

    // Dialog dimensions
    const val DIALOG_WIDTH = 300
    const val DIALOG_WIDTH_WIDE = 400

    // Colors
    const val TEXT_PRIMARY = 0xFFFFFF
    const val TEXT_SECONDARY = 0xAAAAAA
    const val TEXT_MUTED = 0x888888
    const val TEXT_ERROR = 0xFF5555
    const val TEXT_SUCCESS = 0x55FF55
    const val TEXT_WARNING = 0xFFAA00
    const val TEXT_INFO = 0x55FFFF // Cyan for URLs/info
    const val TEXT_DISABLED = 0x666666

    // List item dimensions
    const val ITEM_HEIGHT = 36
    const val ITEM_HEIGHT_COMPACT = 28

    // Card/surface backgrounds (ARGB format for Surface.flat)
    const val CARD_BG = 0x22FFFFFF // Default card background
    const val CARD_BG_LOADED = 0x33AAFFAA // Loaded/active scene card
    const val THUMBNAIL_BG = 0x44888888 // Thumbnail placeholder
    const val SURFACE_SUBTLE = 0x33FFFFFF // Subtle form/section background

    // Selection/highlight colors (ARGB format for Surface.flat)
    const val HIGHLIGHT_BG = 0x33FFFFFF // Hover state
    const val SELECTED_BG = 0x44AAAAFF // Selection state

    // Panel overlay (ARGB)
    const val PANEL_BG_TRANSPARENT = 0x40000000 // Semi-transparent panel mode

    // Grip handle (ARGB, ~37% opacity white)
    const val GRIP_COLOR = 0x60FFFFFF

    // Progress bar
    const val PROGRESS_BG = 0x333333
    const val PROGRESS_FILL = 0x5555FF

    // Indentation levels for tree views
    const val INDENT_LEVEL_1 = 8
    const val INDENT_LEVEL_2 = 28
    const val INDENT_LEVEL_3 = 48

    // Card dimensions
    const val CARD_HEIGHT = 60
    const val CARD_THUMBNAIL_SIZE = 48
    const val CARD_MIN_WIDTH = 180 // Minimum width for auto-flow calculation

    // Sizing helpers
    fun buttonSizing(): Sizing = Sizing.fixed(BUTTON_WIDTH)

    fun buttonWideSizing(): Sizing = Sizing.fixed(BUTTON_WIDTH_WIDE)

    fun buttonHeightSizing(): Sizing = Sizing.fixed(BUTTON_HEIGHT)

    // Inset helpers
    fun paddingSm(): Insets = Insets.of(PADDING_SM)

    fun paddingMd(): Insets = Insets.of(PADDING_MD)

    fun paddingLg(): Insets = Insets.of(PADDING_LG)

    fun paddingVertical(amount: Int): Insets = Insets.vertical(amount)

    fun paddingHorizontal(amount: Int): Insets = Insets.horizontal(amount)
}
