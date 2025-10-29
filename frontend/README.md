# RapidMaker Frontend

Aktualnie frontend to ręcznie przygotowany build statyczny (`dist/`) serwowany przez Axum. Używamy Tailwind CDN, customowego JS (`app.js`) i glassmorphism, żeby szybko dostarczyć ciemny interfejs z drag’n’drop.

## Struktura

- `index.html` – landing + formularz wyceny
- `app.js` – logika (fetch `/api/materials`, wysyłka multipart na `/api/quote`, render wyników)
- `app.css` – gradienty OLED, glass panels, subtelne animacje
- `assets/logo.svg` – logo RapidMaker

## Roadmap

Docelowo planujemy przenieść front na pełen stack (SvelteKit / React + Tailwind + motion). Obecny layout poslizgi jest idealny do szybkich wdrożeń, a później możemy zastąpić go aplikacją SPA bez konieczności zmian w backendzie.

## Dev

W dev nie potrzeba dodatkowego serwera – API serwuje pliki statyczne. Wystarczy odpalić `cargo run -p rapidmaker-api` i wejść na `http://localhost:8080/`.
