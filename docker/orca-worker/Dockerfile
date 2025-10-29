FROM ubuntu:22.04

ARG ORCA_VERSION=v2.1.1
ARG ORCA_APPIMAGE_URL=https://github.com/SoftFever/OrcaSlicer/releases/download/${ORCA_VERSION}/OrcaSlicer_Linux.AppImage

ENV ORCA_HOME=/opt/orca \
    DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        libglu1-mesa \
        libxi6 \
        libxrender1 \
        libxext6 \
        libsm6 \
        libxrandr2 \
        xvfb \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /tmp/orca

RUN curl -L "$ORCA_APPIMAGE_URL" -o OrcaSlicer.AppImage && \
    chmod +x OrcaSlicer.AppImage && \
    ./OrcaSlicer.AppImage --appimage-extract && \
    mv squashfs-root "$ORCA_HOME" && \
    ln -s "$ORCA_HOME/AppRun" /usr/local/bin/orca-slicer && \
    rm OrcaSlicer.AppImage

WORKDIR /workspace

ENTRYPOINT ["orca-slicer"]
CMD ["--help"]
