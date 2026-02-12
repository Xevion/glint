package com.xevion.glint.mixin;

import com.xevion.glint.Glint;
import com.xevion.glint.orchestration.AutonomousRunner;
import com.xevion.glint.ui.GlintMainScreen;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.components.Button;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.client.gui.screens.TitleScreen;
import net.minecraft.network.chat.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/** Adds "Glint" button to the title screen for easy access. */
@Mixin(TitleScreen.class)
public abstract class TitleScreenMixin extends Screen {

    @Unique private static final Logger glint$log = LoggerFactory.getLogger("mixin");
    @Unique private static boolean glint$autonomousTriggered = false;

    protected TitleScreenMixin(Component title) {
        super(title);
    }

    /**
     * Injects Glint button into title screen. Positioned below the language/accessibility buttons
     * row. In autonomous mode, auto-starts orchestration instead.
     */
    @Inject(method = "init", at = @At("RETURN"))
    private void addGlintButton(CallbackInfo ci) {
        Minecraft mc = Minecraft.getInstance();

        if (Glint.INSTANCE.isAutonomous() && !glint$autonomousTriggered) {
            glint$autonomousTriggered = true;
            glint$log.info("Autonomous mode: starting capture runner");

            String apiUrl = Glint.INSTANCE.getApiUrl();
            String apiToken = Glint.INSTANCE.getApiToken();
            if (apiToken.isEmpty()) {
                glint$log.error("Cannot start autonomous mode: GLINT_API_TOKEN not set");
                mc.stop();
                return;
            }

            String forceScenes = Glint.INSTANCE.getForceScenes();
            String forceShaders = Glint.INSTANCE.getForceShaders();
            int workLimit = Glint.INSTANCE.getWorkLimit();
            AutonomousRunner runner =
                    new AutonomousRunner(apiUrl, apiToken, forceScenes, forceShaders, workLimit);
            Glint.INSTANCE.setAutonomousRunner(runner);
            runner.start();
            return;
        }

        int buttonWidth = 98;
        int buttonHeight = 20;
        int buttonY = this.height / 4 + 48 + 72 + 24;
        int buttonX = this.width / 2 - 100;

        Button glintButton =
                Button.builder(
                                Component.literal("Glint"),
                                button -> mc.setScreen(new GlintMainScreen((Screen) (Object) this)))
                        .bounds(buttonX, buttonY, buttonWidth, buttonHeight)
                        .build();

        this.addRenderableWidget(glintButton);
    }
}
