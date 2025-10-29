# 🪟 RapidMaker na Windowsie - Kompletny Tutorial

Cześć! Ten poradnik poprowadzimy cię krok po kroku przez instalację i uruchomienie **RapidMaker Platform** na Windowsie.

**⏱️ Czas na wszystko**: ~15-20 minut (+ czas kompilacji ~5-10 min)

---

## 📋 Wymagania wstępne

Nic specjalnego! Wystarczy:
- Windows 10/11
- Dostęp do internetu
- ~2 GB wolnego miejsca na dysku
- Administrator (do zainstalowania Rust)

---

## 🚀 KROK 1: Instalacja Rust

**WAŻNE**: Zanim zaczniesz - zainstaluj Rust!

### 👉 Przeczytaj: **RUST_INSTALL_WINDOWS.md**

Otwórz plik `RUST_INSTALL_WINDOWS.md` w repozytorium. Jest tam **super szczegółowy poradnik** z:
- 3 metodami instalacji (pick one that works for you!)
- Screenshots descriptions
- Krok po kroku bez "czarnej magii"
- Troubleshooting dla każdego problemu

**Po zainstalowaniu Rust**, wróć tutaj do KROKU 2.

---

## 📁 KROK 2: Pobierz projekt

### Jeśli masz Git zainstalowany:

```powershell
git clone <URL_PROJEKTU>
cd orca-calculate-cost
```

### Jeśli nie masz Git:

1. Pobierz projekt jako ZIP: https://github.com/user/repo/archive/refs/heads/main.zip
2. Rozpakuj folder
3. Otwórz PowerShell w tym folderze

---

## ⚙️ KROK 3: Sprawdzenie struktury projektu

Upewnij się, że widzisz taką strukturę:

```
orca-calculate-cost/
├── Cargo.toml              (plik konfiguracji workspace)
├── Cargo.lock
├── crates/
│   ├── price-engine/       (core logika)
│   │   └── Cargo.toml
│   └── rapidmaker-api/     (web server)
│       ├── Cargo.toml
│       └── src/
│           └── main.rs
├── frontend/dist/          (UI - HTML/CSS)
│   └── index.html
├── Dockerfile
└── docker-compose.yml
```

Jeśli widzisz takie katalogi - OK! ✅

---

## 🔨 KROK 4: Kompilacja i uruchomienie

### Opcja A: Szybki start (bez debugowania)

Otwórz PowerShell w folderze projektu i uruchom:

```powershell
cargo run --release -p rapidmaker-api
```

**Co się stanie:**
1. Rust pobierze wszystkie zależności (~200 MB)
2. Skompiluje projekt (POWOLI - 5-10 minut)
3. Pokaże:
```
INFO  Listening on 0.0.0.0:8080
```

Gratulacje! 🎉 Server działa!

### Opcja B: Wersja development (szybsza kompilacja, wolniejsza pracy)

```powershell
cargo run -p rapidmaker-api
```

(Bez `--release`)

---

## 🌐 KROK 5: Otwórz aplikację

Gdy zobaczysz `Listening on 0.0.0.0:8080`, otwórz przeglądarkę i wejdź na:

```
http://localhost:8080
```

Powinieneś zobaczyć:
- Ciemny interfejs (dark mode)
- Pole do drag & drop plików STL/3MF/G-code
- Dropdown z materiałami (PLA, PETG, ABS, itd.)
- Przycisk "Calculate"

**To jest UI RapidMaker!** ✅

---

## 📝 Jak używać aplikacji

### Na stronie web UI:

1. **Wrzuć plik modelu**
   - Kliknij lub przeciągnij plik `.stl` / `.3mf` / `.gcode`
   - Limit: 50 MB

2. **Wybierz materiał**
   - Listę materiałów widzisz w dropdown

3. **Wybierz parametry** (opcjonalnie)
   - Infill: ile % materiału wewnątrz (20% to standard)
   - Supports: czy dodać podpórki
   - Custom price: jeśli chcesz nadpisać cenę

4. **Kliknij "Calculate"**
   - Czekaj ~1-2 sekundy
   - Zobaczysz:
     - 💰 Koszt
     - 📊 Zużycie filamentu (g)
     - ⏱️ Czas druku
     - 📈 Rozbicie kosztów

---

## 🧪 Testowanie API (zaawansowane)

Jeśli chcesz przetestować API bezpośrednio (bez UI):

### Terminal - test G-code:

```powershell
$payload = @{
    material_id = "pla"
    gcode = "; filament used [g] = 4.2"
} | ConvertTo-Json

Invoke-WebRequest -Uri http://localhost:8080/api/quote/gcode `
  -Method POST `
  -ContentType "application/json" `
  -Body $payload | Select-Object -ExpandProperty Content | ConvertFrom-Json | ConvertTo-Json
```

### Dostępne endpointy:

```
GET  http://localhost:8080/api/health       → Status serwera
GET  http://localhost:8080/api/materials    → Lista materiałów
POST http://localhost:8080/api/quote        → Upload plik (STL/3MF)
POST http://localhost:8080/api/quote/gcode  → Prześlij G-code jako JSON
```

---

## 🛑 Wstrzymanie serwera

Aby zatrzymać RapidMaker:

```
Naciśnij Ctrl + C w PowerShell
```

Server się wyłączy natychmiast.

---

## 🐛 TROUBLESHOOTING

### Problem: `cargo: command not found`

**Rozwiązanie:**
1. Zamknij PowerShell całkowicie
2. Otwórz **nowy** PowerShell (jeśli się zalogowałeś)
3. Jeśli dalej nie działa - dodaj Rust do PATH:

```powershell
# Sprawdzenie gdzie zainstalował się Rust
$env:USERPROFILE\.cargo\bin
```

### Problem: Długa kompilacja (10+ minut)

To **normalne** przy pierwszej kompilacji! Rust:
1. Pobiera wszystkie zależności
2. Kompiluje je (to trwa)
3. Kompiluje projekt
4. Linkuje wykonywalny plik

**Kolejne uruchomienia będą szybsze** (cache).

**Porady:**
- Użyj `--release` dla lepszych optymalizacji (dłuższa kompilacja, szybsze działanie)
- Bez `--release` = szybsza kompilacja, wolniejsze działanie

### Problem: `error: linker 'cc' not found`

**Rozwiązanie:** Brakowuje Visual C++ Build Tools.

```powershell
# Opcja A: Auto-download
cargo update
```

Jeśli to nie pomoże:
1. Pobierz z: https://visualstudio.microsoft.com/visual-cpp-build-tools/
2. Zainstaluj "Desktop development with C++"
3. Restart komputera
4. Spróbuj `cargo run` ponownie

### Problem: Port 8080 zajęty

Jeśli zobaczysz `Error: bind address already in use`:

```powershell
# Zabij aplikację na porcie 8080
Get-Process | Where-Object {$_.ProcessName -eq "rapidmaker-api"} | Stop-Process -Force

# Lub uruchom na innym porcie
$env:RAPIDMAKER_BIND = "127.0.0.1:9090"
cargo run -p rapidmaker-api
```

Wtedy otwórz: http://localhost:9090

### Problem: Out of memory

Jeśli kompilacja pada z `out of memory`:

```powershell
# Zmniejsz liczbę wątków kompilacji
cargo build -p rapidmaker-api -j 2
```

(Zamiast domyślnie wszystkie CPU)

### Problem: Frontend nie ładuje się

Sprawdź czy masz plik:
```
frontend/dist/index.html
```

Jeśli brakuje, pobierz go z repo - jest tam pusty placeholder.

---

## 📚 Struktura plików - gdzie co jest

```
Kod aplikacji:
  crates/rapidmaker-api/src/main.rs         → Server (HTTP)
  crates/price-engine/src/                  → Logika parsowania

Frontend:
  frontend/dist/index.html                  → UI

Testowe pliki 3D:
  sample-cube-20mm.stl
  sample-cylinder.stl
  sample-pyramid.stl
  test-cube.stl

Dokumentacja:
  .claude/docs/                             → Bardziej zaawansowana dokumentacja
  README.md                                 → Podsumowanie (po polsku)
```

---

## 🚀 Następne kroki

### Jeśli chcesz pracować nad kodem:

1. Zainstaluj IDE:
   - **VS Code** (polecam, + Rust extension)
   - **RustRover** (JetBrains Rust IDE)
   - **Visual Studio Code** + rust-analyzer

2. Otwórz folder projektu w IDE

3. VS Code automatycznie powinno zasugerować instalację Rust extension

4. Zamiast `cargo run`, możesz uruchomić z przycisku w IDE

### Jeśli chcesz deployować:

1. **Lokalnie** (najprostsze):
   ```powershell
   cargo build --release -p rapidmaker-api
   # Plik: target/release/rapidmaker-api.exe
   ```

2. **Na serwerze Linux**:
   - Transferuj kod na Linux
   - `cargo build --release -p rapidmaker-api`
   - Uruchom binarny plik

3. **Docker** (jeśli masz Docker Desktop na Windowsie):
   ```powershell
   docker build -t rapidmaker .
   docker run -p 8080:8080 rapidmaker
   ```

---

## ✅ Checklist - czy wszystko działa?

- [ ] Rust zainstalowany (`rustc --version` działa)
- [ ] Projekt pobrany
- [ ] `cargo run -p rapidmaker-api` się uruchamia bez błędów
- [ ] Widzę `Listening on 0.0.0.0:8080` w terminalu
- [ ] http://localhost:8080 się otwiera
- [ ] Frontend wygląda OK (ciemny interfejs)
- [ ] Mogę wybrać plik (drag & drop działa)
- [ ] Mogę wybrać materiał z dropdown
- [ ] Mogę kliknąć "Calculate"

Jeśli wszystko zaznaczone - **gotowe!** 🎉

---

## 💬 FAQ

**P: Czy mogę zmienić port z 8080 na inny?**

A: Tak, ustaw zmienną środowiskową:
```powershell
$env:RAPIDMAKER_BIND = "127.0.0.1:3000"
cargo run -p rapidmaker-api
```

**P: Czy to działa offline?**

A: Tak! Kiedy już skompiluje - nie potrzebuje internetu.

**P: Czy mogę użyć `cargo build` zamiast `cargo run`?**

A: Tak, ale wtedy musisz ręcznie uruchomić plik:
```powershell
.\target\release\rapidmaker-api.exe
```

(Zamiast `cargo run` który robi to automatycznie)

**P: Ile czasu zajmie pierwsza kompilacja?**

A: 5-10 minut w zależności od CPU. Następne `cargo run` będą szybkie.

**P: Czy mogę użyć Python zamiast Rust?**

A: To projekt w Rust i tak musi być. Nie ma wersji Python.

---

## 📞 Potrzebujesz pomocy?

Jeśli coś nie działa:

1. Czytaj **TROUBLESHOOTING** wyżej
2. Sprawdź czy masz Rust 1.70+: `rustc --version`
3. Spróbuj: `cargo clean && cargo build -p rapidmaker-api`
4. Czytaj error message w terminalu - czasem mówi co nie tak

---

**Dobrej zabawy z RapidMaker! 🚀**

*Ostatnia aktualizacja: 2025-10-29*
