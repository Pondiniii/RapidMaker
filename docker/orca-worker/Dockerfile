FROM lscr.io/linuxserver/orcaslicer:latest

ENV ORCA_WORKER_PORT=9090 \
    DEBIAN_FRONTEND=noninteractive

# Install Python and HTTP framework
RUN apt-get update && apt-get install -y --no-install-recommends \
        python3 \
        python3-pip \
        xvfb \
    && rm -rf /var/lib/apt/lists/*

# Install Python dependencies for HTTP wrapper
RUN pip3 install --no-cache-dir flask gunicorn

WORKDIR /workspace

# Copy HTTP wrapper script
COPY orca_worker.py /usr/local/bin/orca_worker.py
RUN chmod +x /usr/local/bin/orca_worker.py

EXPOSE 9090

# Override entrypoint - run our HTTP server instead of GUI
ENTRYPOINT []
CMD ["python3", "/usr/local/bin/orca_worker.py"]
