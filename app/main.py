import os
import magic
import uuid
import tempfile
import re
import mimetypes
from pydantic import BaseModel
from fastapi.responses import HTMLResponse
from fastapi.staticfiles import StaticFiles
from fastapi.templating import Jinja2Templates
from fastapi import FastAPI, Request, UploadFile, HTTPException
import trimesh
from io import BytesIO


# Simple validation function for filename
def validate_filename(filename: str) -> str:
    # Check if filename is longer than 100 characters
    if len(filename) > 100:
        raise HTTPException(status_code=400, detail=f"File name too long: {filename}")
    # Remove any non-alphanumeric and non-period characters (except underscores)
    filename = re.sub(r'[^a-zA-Z0-9_.]', '', filename)

    return filename


mimetypes.init()

app = FastAPI()

templates = Jinja2Templates(directory="templates")

mimetypes.add_type('application/javascript', '.js')

app.mount("/static", StaticFiles(directory="static"), name="static")


class FileData(BaseModel):
    filename: str
    volume: float


files_data = []


@app.get("/", response_class=HTMLResponse)
async def home(request: Request):
    return templates.TemplateResponse('page.html', context={'request': request})


@app.post("/uploadfiles/")
async def create_upload_files(files: list[UploadFile]):
    results = []
    for file in files:
        # Validate the filename
        filename = validate_filename(file.filename)
        file_ext = os.path.splitext(filename)[1].lstrip('.').lower()

        content = await file.read()

        try:
            data = BytesIO(content)
            mesh = trimesh.load(data, file_type=file_ext)

            volume_cm3 = mesh.volume / 1000  # Convert from mm3 to cm3
            surface_area_cm2 = mesh.area / 100  # Convert from mm2 to cm2

            # Calculate the dimensions in x, y, z
            bounds = mesh.bounds
            dimensions = bounds[1] - bounds[0]
            x, y, z = dimensions

            results.append(
                {"filename": filename, "volume": volume_cm3, "surface": surface_area_cm2, "x": x, "y": y, "z": z})

        except Exception as e:
            raise HTTPException(status_code=500, detail=str(e))

    return results
