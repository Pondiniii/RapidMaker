# Użyj obrazu Python 3.10
FROM python:3.10

# Ustaw katalog roboczy w kontenerze
WORKDIR /app

# Skopiuj wszystko do katalogu roboczego
COPY . .

# Zainstaluj zależności
RUN pip install --no-cache-dir -r requirements.txt

# Komenda startowa dla kontenera
CMD ["uvicorn", "router:app", "--workers", "4", "--host", "0.0.0.0", "--port", "80"]

