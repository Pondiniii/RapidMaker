import numpy as np
from stl import mesh
import io

def compute_bounding_box_dimensions(obj_mesh):
    """Compute dimensions of the bounding box for a given mesh."""
    min_vals = np.array([obj_mesh.x.min(), obj_mesh.y.min(), obj_mesh.z.min()])
    max_vals = np.array([obj_mesh.x.max(), obj_mesh.y.max(), obj_mesh.z.max()])

    return max_vals - min_vals

def calculate_print_options(volume: float, bounding_box_dimensions: list, surface_area: float) -> dict:
    # Definicja dostępnych materiałów i ich cen za kilogram
    MATERIAL_COSTS = {
        "flex_resin": 200,
        "nylon_resin": 250,
        "abs_resin": 300,

        "abs": 100,
        "pla": 80,
        "pc": 120,
        "tpu": 160,
    }

    # Gęstości dla poszczególnych materiałów w g/cm^3
    MATERIAL_DENSITIES = {
        "flex_resin": 1.33,
        "nylon_resin": 1.33,
        "abs_resin": 1.33,

        "abs": 1.05,
        "pla": 1.25,
        "pc": 1.2,
        "tpu": 1.25,
    }

    # koszt produkcji razy multiplier który mówi nam
    MATERIAL_PROFIT_MULTIPLIERS = {
        "flex_resin": 4,
        "nylon_resin": 4,
        "abs_resin": 4,

        "abs": 2,
        "pla": 2,
        "pc": 3,
        "tpu": 3,
    }

    LARGE_PRINTER_DIMENSIONS = [500, 500, 500]
    RESIN_PRINTER_DIMENSIONS = [180, 180, 369]

    LARGE_PRINTER_INIT_COST = 10
    RESIN_PRINTER_INIT_COST = 35

    materials = {}

    # Ustalanie wartości infill_walls w zależności od porównania volume z surface_area
    if volume > surface_area:
        difference = volume - surface_area
        difference *= 0.3
        adjusted_volume = surface_area + difference

        if adjusted_volume > volume:
            adjusted_volume = volume
    else:
        adjusted_volume = volume

    print(f"adjusted volume is: {adjusted_volume}")

    # Jeżeli model jest większy niż LARGE_PRINTER_DIMENSIONS, zwróć pusty słownik
    if any(size > max_size for size, max_size in zip(bounding_box_dimensions, LARGE_PRINTER_DIMENSIONS)):
        return {}

    # Jeżeli model mieści się w LARGE_PRINTER_DIMENSIONS
    if all(size <= max_size for size, max_size in zip(bounding_box_dimensions, LARGE_PRINTER_DIMENSIONS)):
        for material in ["abs", "pla", "pc", "tpu"]:
            mass = adjusted_volume * MATERIAL_DENSITIES[material]
            mass_kg = mass / 1000
            materials[material] = round(LARGE_PRINTER_INIT_COST + (mass_kg * MATERIAL_COSTS[material] * MATERIAL_PROFIT_MULTIPLIERS[material]), 2)


    # Jeżeli model mieści się w RESIN_PRINTER_DIMENSIONS
    if all(size <= max_size for size, max_size in zip(bounding_box_dimensions, RESIN_PRINTER_DIMENSIONS)):
        for material in ["flex_resin", "nylon_resin", "abs_resin"]:
            mass = volume * MATERIAL_DENSITIES[material]
            mass_kg = mass / 1000
            materials[material] = round(RESIN_PRINTER_INIT_COST + (mass_kg * MATERIAL_COSTS[material] * MATERIAL_PROFIT_MULTIPLIERS[material]), 2)

    return materials

def calc_volume_bbox(stl_content: bytes):
    # Create a temporary buffer
    buffer = io.BytesIO(stl_content)

    # Read STL file from the buffer
    your_mesh = mesh.Mesh.from_file("temp.stl", fh=buffer)  # Using Mesh.from_file method with a BytesIO buffer

    # Oblicz objętość w mm^3
    volume_mm3 = your_mesh.get_mass_properties()[0]

    # Przelicz objętość na cm^3
    volume_cm3 = round(volume_mm3 / 1000, 2)

    # Oblicz wymiary bounding box
    bounding_box_dimensions = compute_bounding_box_dimensions(your_mesh)

    # Oblicz surface area razy dwa bo 0.4 * 5 grubość ścianek razy
    your_mesh.update_areas()
    surface_area = round(your_mesh.areas.sum() * 2 / 1000, 2)

    return volume_cm3, bounding_box_dimensions, surface_area


# volume, bounding_box_dimensions, surface_area = calc_volume_bbox('sample.stl')
# print(f"Objętość: {volume}")
# print(f"Wymiary Bounding Box: {bounding_box_dimensions}")
# print(f"Surface area: {surface_area}")
# print(calculate_print_options(volume, bounding_box_dimensions, surface_area))

