# Nettworkers

Et Rust-basert nettverks-multiplayer 2D plattformspill som demonstrerer avanserte klient-server netcode-teknikker inkludert klient-side prediksjon, server-rekonsiliering og interpolasjon.

Medlemmene på gruppen er:

- [Henrik Kvamme](https://github.com/henrik392)
- [Henrik Hausberg](https://github.com/henrikhaus)
- [Embret Roås](https://github.com/Embretr)

## 🎮 Oversikt

Nettworkers er et sanntids multiplayer plattformspill som viser frem moderne nettverksteknikker brukt i konkurransedyktige online spill. Spillere kan bevege seg rundt i en 2D verden med fysikkbasert bevegelse mens de opplever jevn, responsiv gameplay til tross for nettverkslatens.

### Hovedfunksjoner

- **Sanntids Multiplayer**: Støtte for flere spillere i en delt 2D verden
- **Avansert Netcode**: Implementering av klient-side prediksjon, server-rekonsiliering og interpolasjon
- **Fysikkmotor**: Tilpasset 2D fysikk med gravitasjon, friksjon og AABB kollisjonsdeteksjon
- **Scene-system**: JSON-basert nivålasting med dekorative og kolliderbare objekter
- **Moderne UI**: Immediate-mode GUI med flere skjermer (hovedmeny, innstillinger, pausemeny)
- **Kryssplattform**: Bygget med Rust for ytelse og portabilitet

## 🏗️ Arkitektur

Prosjektet er strukturert som et Rust workspace med tre hovedkomponenter:

```
nettworkers/
├── server/          # Autoritativ spillserver
├── client/          # Spillklient med rendering og UI
├── shared/          # Felles datastrukturer og protokoller
└── scenes/          # JSON nivådefinisjoner
```

### Server (`server/`)

- **UDP-basert nettverk** på port 9000
- **100ms tick rate** for konsistente spilltilstandsoppdateringer
- **Autoritativ fysikk** simulering
- **Spillerhåndtering** med automatisk ID-tildeling
- **Tilstandskringkasting** til alle tilkoblede klienter

### Klient (`client/`)

- **Macroquad-basert rendering** motor
- **Klient-side prediksjon** for responsiv input
- **Server-rekonsiliering** for tilstandskonsistens
- **Interpolasjon** for jevn bevegelse av andre spillere
- **Scene-rendering** med parallakse-effekter

### Delt (`shared/`)

- **FlatBuffers serialisering** for effektive nettverkspakker
- **Felles spilltilstand** strukturer
- **Fysikkmotor** delt mellom klient og server
- **Kommandosystem** for spillerinput

## 🌐 Nettverksfunksjoner

### Klient-side Prediksjon

Spillere kan bevege seg umiddelbart når de trykker på taster, uten å vente på serverbekreftelse. Klienten predikerer utfallet av handlingene deres lokalt for responsive kontroller.

### Server-rekonsiliering

Når klienten mottar autoritative oppdateringer fra serveren, rekonsilierer den eventuelle forskjeller mellom sin predikerte tilstand og serverens tilstand ved hjelp av sekvensnumre.

### Interpolasjon

Andre spilleres bevegelser interpoleres jevnt mellom serveroppdateringer for å gi flytende visuell bevegelse til tross for den diskrete naturen til nettverksoppdateringer.

### Nettverkssimulering

- **Konfigurerbar forsinkelse**: 1000ms kunstig forsinkelse for testing av netcode-robusthet
- **Sekvensnummerering**: For pålitelig tilstandsrekonsiliering
- **Tidsstempelsynkronisering**: Bruker Unix epoch tidsstempler

## 🚀 Kom i gang

### Forutsetninger

- **Rust** (nyeste stabile versjon)
- **FlatBuffers kompilator** (`flatc`) for protokollgenerering

### Installasjon

1. **Klon repositoriet**:

   ```bash
   git clone https://github.com/henrikhaus/nettworkers
   cd nettworkers
   ```

2. **Generer FlatBuffers kode**:

   ```bash
   make generate_fbs
   ```

3. **Bygg prosjektet**:
   ```bash
   cargo build --workspace
   ```

### Kjøre spillet

1. **Start serveren**:

   ```bash
   cargo run --bin server
   ```

   Serveren vil starte på `127.0.0.1:9000`

2. **Start klienten** (i en separat terminal):

   ```bash
   cargo run --bin client
   ```

3. **Flere klienter**: Kjør flere klientinstanser for å teste multiplayer-funksjonalitet

### Kontroller

- **WASD** eller **Piltaster**: Beveg venstre/høyre og hopp
- **ESC**: Pausemeny
- **Innstillinger**: Slå av/på prediksjon, rekonsiliering og interpolasjonsfunksjoner

## 🔧 Tekniske detaljer

### Nettverksprotokoll

Spillet bruker en tilpasset UDP-protokoll med FlatBuffers serialisering:

#### Klient → Server (Spillerkommandoer)

```rust
table PlayerCommands {
    sequence: uint32;           // For rekonsiliering
    dt_micro: uint64;          // Ramme delta tid
    commands: [PlayerCommand]; // Input kommandoer
    client_timestamp_micro: uint64; // For latensberegning
}
```

#### Server → Klient (Spilltilstand)

```rust
table GameState {
    client_player: ClientPlayer; // Autoritativ klienttilstand
    players: [Player];          // Andre spilleres tilstander
    sequence: uint32;           // Server sekvensnummer
}
```

### Fysikksystem

- **Gravitasjon**: Konstant nedadgående akselerasjon
- **Friksjon**: Bakkefriskjon for realistisk bevegelse
- **AABB Kollisjon**: Akselinjert bounding box kollisjonsdeteksjon
- **Penetrasjonsløsning**: Separerer overlappende objekter

### Ytelseskarakteristikker

- **Server Tick Rate**: 100ms (10 TPS)
- **Klient Bilderate**: Variabel (typisk 60+ FPS)
- **Nettverkspakke størrelse**: ~100-500 bytes per pakke
- **Minnebruk**: Minimal på grunn av Rusts null-kostnad abstraksjoner

## 🧪 Testing

Kjør testsuiten:

```bash
# Kjør alle tester
cargo test --workspace

# Kjør kun servertester
cargo test --package server

# Kjør med verbose output
cargo test --workspace --verbose
```

### Testdekning

- **Serverfunksjonalitet**: Spillerhåndtering, pakkehåndtering, spilltilstandsoppdateringer
- **Fysikksystem**: Bevegelse, kollisjonsdeteksjon, grensebetingelser
- **Integrasjonstester**: Klient-server kommunikasjon
- **CI/CD**: Automatisert testing på GitHub Actions

## 🎨 Spillinnhold

### Scene-format

Nivåer er definert i JSON-format med følgende struktur:

```json
{
  "width": 2000.0,
  "height": 600.0,
  "spawn_point": { "x": 100.0, "y": 450.0 },
  "background_color": { "r": 20, "g": 20, "b": 50, "a": 255 },
  "decorations": {
    /* Visuelle elementer */
  },
  "collidables": {
    /* Solide plattformer og hindringer */
  }
}
```

### Tilgjengelige scener

- **scene_1.json**: Hovedplattformspillnivå med flere plattformer og dekorasjoner
- **scene_2.json**: Alternativ nivålayout

## 🛠️ Utvikling

### Prosjektstruktur

```
client/src/
├── main.rs              # Klient inngangspunkt og spillløkke
├── predictor.rs         # Klient-side prediksjonslogikk
├── interpolator.rs      # Interpolasjon for andre spillere
├── render.rs           # Renderingsystem
├── ui/                 # Brukergrensesnittkomponenter
└── game_logic/         # Spilltilstandshåndtering

server/src/
└── main.rs             # Server inngangspunkt og nettverk

shared/src/
├── state/              # Spilltilstand og fysikk
├── *.fbs              # FlatBuffers skjemadefinisjoner
└── generated/         # Auto-generert FlatBuffers kode
```

### Hovedavhengigheter

- **flatbuffers**: Effektiv binær serialisering
- **macroquad**: Kryssplattform spillramme
- **serde/serde_json**: JSON parsing for scener
- **Standard bibliotek**: Nettverk, threading, samlinger

### Legge til funksjoner

1. **Nye spillerkommandoer**: Legg til i `player_commands.fbs` og regenerer
2. **Spillmekanikk**: Implementer i `shared/src/state/`
3. **UI-elementer**: Legg til i `client/src/ui/`
4. **Scener**: Opprett nye JSON-filer i `scenes/`

## 🔍 Feilsøking

### Nettverksfeilsøking

- **Pakkeinspisering**: Server logger alle mottatte pakker
- **Latenssimulering**: Konfigurerbar forsinkelse for testing
- **Sekvenssporing**: Overvåk prediksjon/rekonsilieringsykler

### Ytelsesprofilering

```bash
# Profiler serveren
cargo run --release --bin server

# Profiler klienten
cargo run --release --bin client
```

## 📚 Læringsressurser

Dette prosjektet demonstrerer flere viktige nettverkskonsepter:

- **Klient-Server Arkitektur**: Autoritativ server med klientprediksjon
- **Tilstandssynkronisering**: Holde flere klienter synkronisert
- **Latenskompensasjon**: Teknikker for responsiv gameplay
- **Binære protokoller**: Effektiv serialisering med FlatBuffers
- **Sanntidssystemer**: Håndtering av timing og konsistens

## 🎯 Fremtidige forbedringer

- **TCP pålitelighetssjikt** for kritiske spillhendelser (for eksempel hvis vi introduserer våpenkamp)
- **Avansert fysikk** (skråninger, bevegelige plattformer)
- **Lydsystem** med romlig lyd
- Bedre **rekonsilieringsoptimaliseringer**

## 🪈 Pipeline

Lenke til siste pipeline: https://github.com/henrikhaus/nettworkers/actions/runs/15261817355/job/42920909344
