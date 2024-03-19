import mimetypes
from fastapi.responses import HTMLResponse
from fastapi.staticfiles import StaticFiles
from fastapi.templating import Jinja2Templates
from fastapi import FastAPI, Request, UploadFile, HTTPException
from stl_price_calculator import calc_volume_bbox, calculate_print_options
from fastapi.middleware.gzip import GZipMiddleware
from file_validator import file_check
# todo dodać file check

mimetypes.init()
app = FastAPI()
app.add_middleware(GZipMiddleware, minimum_size=1000)
templates = Jinja2Templates(directory="templates")
mimetypes.add_type('application/javascript', '.js')
app.mount("/static", StaticFiles(directory="static"), name="static")

@app.get("/", response_class=HTMLResponse)
async def home(request: Request):
    return templates.TemplateResponse('page.html', context={'request': request})


# @app.post("/upload/stl")
# async def upload_stl(file: UploadFile = None):
#     if not file:
#         raise HTTPException(status_code=400, detail="File not provided")

#     file_content = await file.read()

#     # Sprawdzanie poprawności i nazwy pliku
#     is_valid, new_file_name = file_check(file_content, file.filename)
#     if not is_valid:
#         raise HTTPException(status_code=400, detail=new_file_name)  # Tutaj używamy new_file_name jako szczegół błędu

#     try:
#         # Obliczanie objętości, wymiarów bounding box i powierzchni
#         volume, bounding_box_dimensions, surface_area = calc_volume_bbox(file_content)

#         # Obliczanie ceny
#         price_dict = calculate_print_options(volume, bounding_box_dimensions, surface_area)

#     except Exception as e:
#         raise HTTPException(status_code=500, detail=f"Error processing file: {e}")

#     return {
#         "filename": new_file_name,  # Teraz używamy nowej, przetworzonej nazwy pliku
#         "price_dict": price_dict
#     }
