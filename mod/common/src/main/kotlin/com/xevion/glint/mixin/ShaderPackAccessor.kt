package com.xevion.glint.mixin

import net.irisshaders.iris.shaderpack.ShaderPack
import net.irisshaders.iris.shaderpack.option.menu.OptionMenuContainer
import org.spongepowered.asm.mixin.Mixin
import org.spongepowered.asm.mixin.gen.Accessor

/**
 * Mixin accessor to access private fields in Iris's ShaderPack class.
 * Provides type-safe access without reflection overhead.
 */
@Mixin(ShaderPack::class)
interface ShaderPackAccessor {
    /**
     * Accessor for the private menuContainer field.
     * Required to access shader profiles which aren't exposed via public Iris API.
     * @return The shader pack's menu container with options and profiles
     */
    @Accessor("menuContainer")
    fun getMenuContainer(): OptionMenuContainer
}
