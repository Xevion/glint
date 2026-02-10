# Glint Headless Capture Environment
# Multi-stage build: mod build → MC runtime assembly → final runtime image
#
# Usage:
#   docker build -t glint-capture:latest -f docker/Dockerfile .
#   docker run --rm --gpus all -e NVIDIA_DRIVER_CAPABILITIES=all glint-capture:latest

# ── Stage 1: Build the Glint mod ──────────────────────────────────────────────
FROM eclipse-temurin:21-jdk-noble AS mod-build

WORKDIR /build

# Copy Gradle wrapper and build config first (dependency caching layer)
COPY mod/gradle/ mod/gradle/
COPY mod/gradlew mod/gradlew
COPY mod/build.gradle mod/build.gradle
COPY mod/settings.gradle mod/settings.gradle
COPY mod/gradle.properties mod/gradle.properties

# Copy subproject build files
COPY mod/common/build.gradle mod/common/build.gradle
COPY mod/fabric/build.gradle mod/fabric/build.gradle
COPY mod/neoforge/build.gradle mod/neoforge/build.gradle

# Pre-download dependencies (cached unless build files change)
RUN chmod +x mod/gradlew
WORKDIR /build/mod
RUN ./gradlew :fabric:dependencies --no-daemon 2>&1 || true

# Copy all source code
WORKDIR /build
COPY mod/ mod/

# Build the production JAR (Fabric only — Docker capture targets Fabric)
WORKDIR /build/mod
RUN ./gradlew :fabric:remapJar --no-daemon \
    && echo "=== Build output ===" \
    && ls -la fabric/build/libs/ \
    && echo "=== Selecting remapped JAR (no -dev-shadow, no -sources) ===" \
    && ls fabric/build/libs/*.jar | grep -v dev-shadow | grep -v sources

# ── Stage 2: Assemble Minecraft + Fabric runtime ─────────────────────────────
FROM python:3.12-slim AS mc-runtime

# portablemc needs Python; Bun handles everything else (HTTP, JSON, zip, file I/O)
RUN pip install --no-cache-dir portablemc
COPY --from=oven/bun:slim /usr/local/bin/bun /usr/local/bin/bun

COPY docker/setup-runtime.ts /setup-runtime.ts

# Pinned versions — override with --build-arg
ARG MC_VERSION=1.21.4
ARG FABRIC_LOADER=0.16.9

ENV MC_VERSION=${MC_VERSION}
ENV FABRIC_LOADER=${FABRIC_LOADER}
ENV MC_DIR=/minecraft

RUN bun /setup-runtime.ts

# ── Stage 3: Final runtime image ─────────────────────────────────────────────
FROM nvidia/opengl:1.2-glvnd-runtime-ubuntu22.04

# Install runtime dependencies
ARG VGL_VERSION=3.1.4
RUN apt-get update && apt-get install -y --no-install-recommends \
    openjdk-21-jre-headless \
    xvfb \
    mesa-utils \
    libgl1 \
    libglx-mesa0 \
    libegl1 \
    libxtst6 \
    libglu1-mesa \
    libopenal1 \
    # X11 client libraries required by GLFW for platform detection
    libx11-6 \
    libxrandr2 \
    libxinerama1 \
    libxcursor1 \
    libxi6 \
    libxxf86vm1 \
    pciutils \
    xdotool \
    ffmpeg \
    curl \
    unzip \
    && curl -fSL -o /tmp/virtualgl.deb \
    "https://github.com/VirtualGL/virtualgl/releases/download/${VGL_VERSION}/virtualgl_${VGL_VERSION}_amd64.deb" \
    && dpkg -i /tmp/virtualgl.deb \
    && dpkg -s virtualgl \
    && rm /tmp/virtualgl.deb \
    && apt-get purge -y --auto-remove curl \
    && rm -rf /var/lib/apt/lists/*

# Install Bun for the entrypoint script
COPY --from=oven/bun:slim /usr/local/bin/bun /usr/local/bin/bun

# Copy Minecraft runtime from Stage 2
COPY --from=mc-runtime /minecraft /minecraft

# Copy Glint mod JAR from Stage 1
# Architectury produces multiple JARs (remapped, dev-shadow, sources).
# Copy all to a temp dir, then pick the right one (remapped = no classifier).
COPY --from=mod-build /build/mod/fabric/build/libs/ /tmp/mod-jars/
RUN set -e; \
    JAR=$(ls /tmp/mod-jars/*.jar 2>/dev/null | grep -v dev-shadow | grep -v sources | head -1); \
    if [ -z "$JAR" ]; then \
        echo "ERROR: No Glint mod JAR found in build output:"; \
        ls -la /tmp/mod-jars/; \
        exit 1; \
    fi; \
    cp "$JAR" /minecraft/mods/glint.jar; \
    echo "Installed mod: $JAR -> /minecraft/mods/glint.jar ($(du -h /minecraft/mods/glint.jar | cut -f1))"; \
    rm -rf /tmp/mod-jars

# Copy Docker support files
COPY docker/options.txt /docker/options.txt
COPY docker/entrypoint.ts /docker/entrypoint.ts

# Environment defaults
ENV MC_DIR=/minecraft \
    DISPLAY_NUM=99 \
    DISPLAY=:99 \
    RESOLUTION=1920x1080x24 \
    JAVA_MEM=6G \
    GLINT_AUTONOMOUS=true \
    NVIDIA_DRIVER_CAPABILITIES=all \
    NVIDIA_VISIBLE_DEVICES=all \
    XDG_RUNTIME_DIR=/tmp/xdg-runtime \
    XDG_SESSION_TYPE=x11

RUN mkdir -p /tmp/xdg-runtime

VOLUME ["/shaderpacks", "/worlds", "/output"]

ENTRYPOINT ["bun", "/docker/entrypoint.ts"]
