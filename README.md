# RapidMaker Platform

Nowa generacja RapidMaker to w pełni rustowy stack, ciemna strona inspirowana OLED i gotowy obraz Dockera.

## Co w środku?

- **`crates/price-engine`** – core w Rust odpowiedzialny za parsowanie G-code / 3MF / STL, obliczanie objętości, masy, czasu i kosztów. Testy pokrywają główne ścieżki.
- **`crates/rapidmaker-api`** – serwis HTTP (Axum) z endpointami `GET /api/health`, `GET /api/materials`, `POST /api/quote/gcode`, `POST /api/quote` (multipart). Uploady trzymane są w katalogu tymczasowym (domyślnie `/tmp/rapidmaker`), limit 50 MB i łatwo przenieść na tmpfs.
- **`frontend/dist`** – ciemny landing z Tailwind CDN, glassmorphism, gradientami #3bcf74 / #00c896, drag’n’drop dla STL/3MF/G-code i dynamiczny panel wyników.
- **`docker-compose.yml` + `Dockerfile`** – multi-stage build Rust → Debian Slim, healthcheck i tmpfs. W folderze `docker/orca-worker/` jest opcjonalny obraz do CLI Orca Slicer.

## Szybki start (dev)

```bash
# Testy całego workspace
cargo test

# Uruchom API na 0.0.0.0:8080 (serwuje też frontend/dist)
cargo run -p rapidmaker-api
```

Przykładowe wywołanie (G-code inline):

```bash
PAYLOAD='{"material_id":"pla","gcode":"; filament used [g] = 4.2"}'
curl -s http://127.0.0.1:8080/api/quote/gcode \
  -H 'content-type: application/json' \
  -d "$PAYLOAD" | jq
```

## Frontend

`frontend/dist/index.html` korzysta z Tailwind CDN (bez bundlera), dlatego w dev możesz po prostu otworzyć plik lub wejść na `http://localhost:8080/`. Panel umożliwia:

- przeciąganie STL/3MF/G-code,
- wybór materiału z `/api/materials`,
- ustawienie infillu, podpór i nadpisanie ceny filamentu / opłaty bazowej,
- elegancki breakdown w PLN.

## Docker

```bash
# build
docker build -t rapidmaker .

# run (serwuje API + frontend na :8080)
docker run --rm -p 8080:8080 \
  -e RAPIDMAKER_UPLOAD_DIR=/tmp/rapidmaker \
  --tmpfs /tmp/rapidmaker:size=64m rapidmaker

# docker compose (z tmpfs i healthcheck)
docker compose up --build
```

Profil `orca` w compose buduje dodatkowo kontener `orca-worker` z OrcaSlicer AppImage (domyślny URL można nadpisać przez `ORCA_APPIMAGE_URL`).

## API

| Metoda | Ścieżka            | Opis                                                                 |
| ------ | ------------------ | -------------------------------------------------------------------- |
| GET    | `/api/health`      | Status serwera + limit uploadu                                       |
| GET    | `/api/materials`   | Lista materiałów z gęstościami i cenami domyślnymi                   |
| POST   | `/api/quote/gcode` | JSON `{ material_id, gcode, ... }`                                   |
| POST   | `/api/quote`       | Multipart: `file`, `material_id`, `infill_percent`, `supports`, opc. `cost/base_fee` |

Każdy wynik zawiera `breakdown` (koszt materiału, opłata bazowa, mnożnik) oraz `metadata` (np. objętość, długość filamentu, estymowany czas). Dla STL/TMP widoczny jest znacznik `is_estimate: true`.

## Przykładowe modele

Repo zawiera paczkę testową:

- `sample-cube-20mm.stl`
- `sample-cylinder.stl`
- `sample-pyramid.stl`
- `test-cube.stl`

Przydają się do ręcznego smoke testu.

## TODO / roadmap

- [ ] Viewer 3D (np. trzy.js w lazily-loaded module)
- [ ] Integracja API ↔ `orca-worker` (precyzyjny slicing / G-code na żądanie)
- [ ] GitHub Actions (fmt, clippy, testy, build obrazu)
- [ ] Telemetria (opcjonalna) i dodatkowe logi bezpieczeństwa

## Licencja

MIT – rób z tym co chcesz.
