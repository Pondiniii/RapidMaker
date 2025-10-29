# 🦀 Instalacja Rust na Windowsie - Super szczegółowy poradnik

Zanim zaczniesz pracować z RapidMaker, musisz mieć zainstalowany **Rust**.

Ten poradnik pokazuje dokładnie jak, bez "czarnej magii". ✨

---

## ❓ Po co Rust?

RapidMaker to projekt napisany w Rust. Żeby go uruchomić, potrzebujesz:
1. **Rust compiler** (tłumaczy kod Rust na program Windows)
2. **Cargo** (package manager - pobiera zależności)

Instalujesz je jednocześnie za pomocą **rustup** (universal installer).

---

## 📥 INSTALACJA - 3 metody

### ✅ METODA 1: Automatycznie (ŁATWIEJ!)

Czas: **2 minuty**

#### Krok 1: Otwórz PowerShell jako Administrator

1. Naciśnij **Windows key** (klawisz z logiem Windows)
2. Wpisz: `powershell`
3. Kliknij prawym przyciskiem myszy na **Windows PowerShell** (nie "PowerShell ISE")
4. Wybierz **Uruchom jako administrator**

   ```
   Powinno pojawić się okno terminal:
   PS C:\Users\TwojaNazwaUzytkownika>
   ```

#### Krok 2: Skopiuj i wklej tę komendę

```powershell
irm https://win.rustup.rs | iex
```

Wklej do PowerShell (kliknij prawym przyciskiem → Paste):

```
PS C:\Users\TwojaUzytkownika> irm https://win.rustup.rs | iex
```

Naciśnij **Enter**.

#### Krok 3: Czekaj i zaakceptuj

Powinien się otworzyć **Rust Installer**.

Zobaczysz okno:

```
Welcome to Rust!

This will download and install the official compiler for the
Rust programming language, and its package manager, Cargo.

Rustup metadata and toolchains will be installed into the Rustup
home directory, located at:

  %USERPROFILE%\.rustup

This can be modified with the RUSTUP_HOME environment variable.

The Cargo home directory is located at:

  %USERPROFILE%\.cargo

This can be modified with the CARGO_HOME environment variable.

The toolchain to be installed is:

  stable-x86_64-pc-windows-msvc

The valid installation option is:

  1) Proceed with standard installation (default)

Options:
  2) Customize installation
  3) Cancel installation

>
```

**Wpisz: `1` i naciśnij Enter**

```
Instalacja zaczyna się automatycznie...
```

#### Krok 4: Czekaj aż się skończy

Zobaczysz coś takiego:

```
...
Unpacking 'cargo-1.75.0-x86_64-pc-windows-msvc.tar.xz'...
Installing 'rust-std-1.76.0-x86_64-pc-windows-msvc'...
...
   Compiling proc-macro2 v1.0.69
   Compiling quote v1.0.33
   ...
```

To normalne! Kompiluje się kod.

Po kilku minutach powinno pojawić się:

```
Rust is installed now. Great!

To get started, you may need to restart your current shell.
This would reload its environment variables.
```

**✅ Rust zainstalował się!**

#### Krok 5: Zamknij i otwórz PowerShell od nowa

```
1. Zamknij PowerShell (X w rogu okna)
2. Otwórz PowerShell znowu (bez admin, normalnie)
   - Windows key → wpisz "powershell" → Enter
```

---

### ✅ METODA 2: Ręczna instalacja (jeśli Metoda 1 nie działa)

Czas: **5 minut**

#### Krok 1: Pobierz instalator

1. Otwórz przeglądarkę: https://www.rust-lang.org/tools/install

2. Zobaczysz taki ekran:

```
┌─────────────────────────────────────┐
│  Download Rust                      │
│                                     │
│  Get started with Rust by           │
│  downloading rustup, and then read  │
│  the official book.                 │
│                                     │
│ [ Download rustup-init.exe ]  ← kliknij tu
│ (Windows x64 / MSVC)               │
│                                     │
└─────────────────────────────────────┘
```

**Kliknij: "Download rustup-init.exe"**

Plik `rustup-init.exe` pobierze się do **Downloads**.

#### Krok 2: Uruchom instalator

1. Otwórz folder **Downloads** (Windows key → "downloads" → enter)

2. Poszukaj pliku: `rustup-init.exe`

3. **Dwukliknij** na plik

4. Pojawi się okno Command Prompt (czarny terminal):

```
Welcome to Rust!

This will download and install the official compiler for the
Rust programming language...

1) Proceed with standard installation (default)
2) Customize installation
3) Cancel installation
>
```

#### Krok 3: Zaakceptuj domyślne ustawienia

**Wpisz: `1` i naciśnij Enter**

Czekaj aż instalacja się skończy (~3-5 minut).

#### Krok 4: Zamknij i sprawdź

Po skończeniu pojawi się:

```
Rust is installed now. Great!
```

**Zamknij to okno.**

---

### ✅ METODA 3: Visual Studio Build Tools (jeśli metody wyżej zawiodły)

Czasami Windows potrzebuje Visual C++ Build Tools.

#### Krok 1: Pobierz

Wejdź na: https://visualstudio.microsoft.com/visual-cpp-build-tools/

Kliknij: **"Download Visual Studio Build Tools"**

#### Krok 2: Instaluj

Uruchom pobrany plik `vs_BuildTools.exe`.

W oknie wybierz:
- ☑️ **Desktop development with C++**

Kliknij **Install** i czekaj (~10 minut).

#### Krok 3: Powtórz METODĘ 1 lub 2

Po zainstalowaniu Visual Studio Build Tools, spróbuj METODY 1 ponownie.

---

## ✅ WERYFIKACJA: Czy Rust zainstalował się poprawnie?

#### Krok 1: Otwórz PowerShell

Windows key → wpisz "powershell" → Enter

#### Krok 2: Wpisz tę komendę

```powershell
rustc --version
```

Naciśnij Enter.

#### Krok 3: Sprawdź wynik

**Jeśli zobaczysz:**
```
rustc 1.76.0 (07dca489a 2024-01-10)
```

**✅ SUPER! Rust zainstalował się!**

**Jeśli zobaczysz:**
```
rustc: The term 'rustc' is not recognized...
```

→ Zamknij PowerShell i otwórz go **znowu**. (Zmiany ścieżek wymagają restart shell'a)

---

## 🔍 Rozbudowana weryfikacja

Chcesz się upewnić że masz wszystko? Wpisz w PowerShell:

```powershell
rustc --version
cargo --version
rustup --version
```

Powinny pokazać się wersje:

```
rustc 1.76.0 (...)
cargo 1.76.0 (...)
rustup 1.26.0 (...)
```

**✅ Wszystko OK!**

---

## 📍 Gdzie Rust się zainstalował?

Rust zainstaluje się w:

```
C:\Users\TwojaUzytkownika\.rustup
C:\Users\TwojaUzytkownika\.cargo
```

(Katalogi ukryte, rozpoczynające się od `.`)

**Normalnie nie musisz tam wchodzić.**

---

## 🛠️ TROUBLESHOOTING instalacji

### Problem: "PowerShell is disabled on this system"

**Przyczyna:** Administrator wyłączył PowerShell.

**Rozwiązanie:**
1. Otwórz Command Prompt (nie PowerShell):
   - Windows key → wpisz "cmd" → Enter

2. Wpisz:
```cmd
irm https://win.rustup.rs | iex
```

Jeśli `irm` nie działa, użyj METODY 2 (ręczny pobór `rustup-init.exe`).

---

### Problem: "Error: could not write to installation directory"

**Przyczyna:** Brak uprawnień administratora.

**Rozwiązanie:**
1. Zamknij PowerShell/Cmd
2. Kliknij prawym przyciskiem myszy
3. **Uruchom jako administrator** (ważne!)
4. Spróbuj instalacji znowu

---

### Problem: "This is the x86 (32-bit) version..."

Instalator pokazuje wersję 32-bitową, a masz Windows 64-bitowy.

**Rozwiązanie:**

Sprawdź czy pobierasz z:
https://www.rust-lang.org/tools/install

I szukaj: **Download for x86_64-pc-windows-msvc** (64-bit)

Nie pobieraj wersji i686!

---

### Problem: "MSVC is not installed..."

**Przyczyna:** Brakowuje Visual C++ Build Tools.

**Rozwiązanie:** Zainstaluj METODĘ 3 (Visual Studio Build Tools).

---

### Problem: "Internet Explorer is not available"

Ignoruj. To stary warning. Rust będzie działać.

---

## 🌍 Ścieżka (PATH) - co to?

Po instalacji Rust dodaje się do **ścieżki (PATH)** systemu.

To znaczy, że gdziekolwiek otworzysz terminal, możesz wpisać `rustc` czy `cargo` i system będzie wiedział gdzie szukać.

**Normalnie nie musisz o tym wiedzieć**, ale jeśli `rustc --version` nie działa:

1. Zamknij terminal całkowicie
2. Otwórz nowy terminal
3. Spróbuj znowu

---

## ✨ Dodatkowe przydatne komendy

Po instalacji możesz:

```powershell
# Sprawdzić wersję
rustc --version

# Zaktualizować Rust (raz w miesiącu)
rustup update

# Zainstalować nightly (dla zaawansowanych - NIE rób tego!)
rustup toolchain install nightly

# Sprawdzić gdzie Rust jest zainstalowany
rustup show
```

---

## 📚 Co dalej?

Gdy Rust zainstalujesz:

1. Przejdź do pliku: **WINDOWS_SETUP.md** w projekcie RapidMaker
2. Postępuj po jego instrukcjach
3. Uruchom `cargo run -p rapidmaker-api`

---

## ✅ Checklist

- [ ] Otworzył(a)em PowerShell jako Admin
- [ ] Wkleił(a)em komendę instalacji
- [ ] Zaakceptował(a)em standardową instalację (opcja 1)
- [ ] Czekał(a)em aż się skończy
- [ ] Zamknął(a)em i otworzyłem PowerShell ponownie
- [ ] `rustc --version` pokazuje wersję (nie błąd!)
- [ ] `cargo --version` działa

**Jeśli wszystkie zaznaczone - ✅ Rust gotowy!**

---

## 💬 Potrzebujesz pomocy?

Jeśli coś mi nie idzie:

1. **Poczytaj TROUBLESHOOTING wyżej** - tam jest 90% rozwiązań
2. **Sprawdzić czy masz admin** - kliknij prawym przyciskiem myszy → Uruchom jako administrator
3. **Restart komputera** - czasami to magiczne rozwiązanie
4. **Spróbuj METODY 2** - ręczny pobór `rustup-init.exe`

---

**Powodzenia! 🚀**

*Ostatnia aktualizacja: 2025-10-29*
