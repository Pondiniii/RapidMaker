#!/usr/bin/env python3
"""
OrcaSlicer HTTP Worker - Provides HTTP API for slicing 3D models
Accepts: STL, 3MF, STEP, STP, OBJ
Returns: G-code with metadata (filament weight, time, etc)
"""

import os
import json
import tempfile
import subprocess
import re
from pathlib import Path
from flask import Flask, request, jsonify

app = Flask(__name__)

WORKSPACE = Path(os.getenv("WORKSPACE", "/workspace"))
ORCA_CMD = os.getenv("ORCA_CMD", "orca-slicer")
PORT = int(os.getenv("ORCA_WORKER_PORT", 9090))

SUPPORTED_FORMATS = {".stl", ".3mf", ".step", ".stp", ".obj"}


def parse_gcode_metadata(gcode: str) -> dict:
    """Extract metadata from generated G-code"""
    metadata = {
        "filament_used_g": None,
        "filament_used_mm": None,
        "filament_cost": None,
        "print_time_seconds": None,
        "print_time_human": None,
        "total_layers": None,
    }

    for line in gcode.split("\n"):
        if not line.startswith(";"):
            continue

        lower = line.lower()

        if "filament used [g]" in lower:
            match = re.search(r"=\s*([\d.]+)", line)
            if match:
                metadata["filament_used_g"] = float(match.group(1))

        elif "filament used [mm]" in lower:
            match = re.search(r"=\s*([\d.]+)", line)
            if match:
                metadata["filament_used_mm"] = float(match.group(1))

        elif "filament cost" in lower:
            match = re.search(r"=\s*([\d.]+)", line)
            if match:
                metadata["filament_cost"] = float(match.group(1))

        elif "estimated printing time" in lower:
            # Extract human-readable time (e.g., "23m 46s")
            time_match = re.search(r"\)\s*=\s*(.+)$", line)
            if time_match:
                metadata["print_time_human"] = time_match.group(1).strip()

        elif "total layers count" in lower:
            match = re.search(r"=\s*(\d+)", line)
            if match:
                metadata["total_layers"] = int(match.group(1))

        # Try to extract print time in seconds from comments
        if "seconds" in lower and "=" in line:
            match = re.search(r"=\s*(\d+)", line)
            if match and metadata["print_time_seconds"] is None:
                metadata["print_time_seconds"] = int(match.group(1))

    return metadata


def slice_model(model_path: Path, config: dict) -> tuple[str, dict]:
    """
    Slice 3D model using OrcaSlicer CLI

    Args:
        model_path: Path to model file
        config: Configuration dict with optional:
            - infill: infill percentage (0-100)
            - supports: bool
            - profile: profile name

    Returns:
        (gcode_content, metadata)
    """
    if not model_path.exists():
        raise FileNotFoundError(f"Model file not found: {model_path}")

    if model_path.suffix.lower() not in SUPPORTED_FORMATS:
        raise ValueError(f"Unsupported format: {model_path.suffix}")

    output_gcode = model_path.with_suffix(".gcode")

    # Build orca-slicer command
    cmd = [ORCA_CMD, "-o", str(output_gcode), str(model_path)]

    # Note: For now we don't pass infill/supports via CLI
    # OrcaSlicer CLI doesn't easily support these - would need config files
    # In production, could generate 3MF with settings or use config profiles

    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=300,  # 5 minute timeout
        )

        if result.returncode != 0:
            return None, {"error": f"Slicing failed: {result.stderr}"}

        if not output_gcode.exists():
            return None, {"error": "G-code file was not generated"}

        gcode_content = output_gcode.read_text()
        metadata = parse_gcode_metadata(gcode_content)

        # Cleanup
        output_gcode.unlink()

        return gcode_content, metadata

    except subprocess.TimeoutExpired:
        return None, {"error": "Slicing timeout (>5 minutes)"}
    except Exception as e:
        return None, {"error": str(e)}


@app.route("/health", methods=["GET"])
def health():
    """Health check endpoint"""
    return jsonify({"status": "ok", "service": "orca-worker"})


@app.route("/api/slice", methods=["POST"])
def slice_api():
    """
    Slice a 3D model file

    Request:
    - file: multipart file (STL, 3MF, STEP, STP, OBJ)
    - config: JSON string with optional {infill, supports, profile}

    Response:
    {
        "success": bool,
        "gcode": string (if successful),
        "metadata": {
            "filament_used_g": float,
            "filament_used_mm": float,
            "print_time_human": string,
            "total_layers": int,
            ...
        },
        "error": string (if failed)
    }
    """
    if "file" not in request.files:
        return jsonify({"success": False, "error": "No file provided"}), 400

    file = request.files["file"]
    if file.filename == "":
        return jsonify({"success": False, "error": "No filename"}), 400

    # Parse config if provided
    config = {}
    if "config" in request.form:
        try:
            config = json.loads(request.form["config"])
        except json.JSONDecodeError:
            return jsonify({"success": False, "error": "Invalid config JSON"}), 400

    # Save temporary file
    try:
        with tempfile.NamedTemporaryFile(
            suffix=Path(file.filename).suffix, delete=False, dir=WORKSPACE
        ) as tmp:
            file.save(tmp.name)
            model_path = Path(tmp.name)

        # Slice
        gcode, metadata = slice_model(model_path, config)

        # Cleanup
        model_path.unlink(missing_ok=True)

        if gcode is None:
            return (
                jsonify({"success": False, "metadata": metadata}),
                400,
            )

        return jsonify(
            {
                "success": True,
                "gcode": gcode,
                "metadata": metadata,
            }
        )

    except Exception as e:
        return (
            jsonify({"success": False, "error": str(e)}),
            500,
        )


if __name__ == "__main__":
    WORKSPACE.mkdir(parents=True, exist_ok=True)
    print(f"🪡 OrcaSlicer Worker starting on 0.0.0.0:{PORT}")
    app.run(host="0.0.0.0", port=PORT, debug=False)
