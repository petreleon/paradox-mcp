# Server MCP pentru Baze de Date Paradox (Rust)

Acesta este un server bazat pe Model Context Protocol (MCP) construit special pentru a interacționa cu fișierele bazelor de date vechi Paradox (`.db`). Este dezvoltat în **Rust** pentru performanță și siguranță a memoriei, și utilizează biblioteca C `pxlib` pentru operațiunile de nivel scăzut de citire și scriere (prin intermediul FFI/bindgen).

Serverul comunică exclusiv prin stream-urile `stdio` folosind JSON-RPC.

## Utilizare

Cea mai simplă și unitară metodă de a rula acest server MCP este prin intermediul **Docker**. Abordarea containerizată gestionează elegant dependențele Linux (precum pachetul `pxlib-dev`), eliminând nevoia de a instala manual biblioteci C sau toolchain-uri Rust pe sistemul dumneavoastră gazdă (cum ar fi macOS sau Windows).

### 1. Construirea Imaginii Docker

Pentru a construi imaginea, rulați din rădăcina proiectului:

```bash
docker build -t paradox-mcp .
```

### 2. Rularea Serverului MCP

Pentru a putea citi fișierele, trebuie să asociați un volum (bind mount) între directorul host-ului dumneavoastră care conține bazele de date și un director din container (ex. `/data`).

```bash
docker run -i --rm -v /calea/catre/db/paradox/host:/data paradox-mcp --location /data
```

Pentru a permite și **editarea** fișierelor (creare, inserare, actualizare), este obligatoriu să adăugați parametrul de securitate `--permit-editing`:

```bash
docker run -i --rm -v /calea/catre/db/paradox/host:/data paradox-mcp --location /data --permit-editing
```

## Instrumente Disponibile (MCP Tools)

Odată integrat într-un client compatibil cu MCP (cum ar fi Claude, Cursor, Windsurf etc.), serverul expune o suită completă de unelte (tools) pentru interogarea și modificarea bazelor:

- `get_server_status`: Returnează informațiile critice de pornire (locația bazelor și dacă editarea este permisă).
- `list_tables`: Scanează directorul configurat și afișează toate fișierele Paradox `.db` disponibile.
- `read_table_schema`: Extrage și afișează structura unui anumit tabel (numele, tipul și dimensiunea permisă a fiecărui câmp - de ex. ALPHA, LONG, SHORT, LOGICAL).
- `read_table_data`: Returnează conținutul (înregistrările) unui tabel, convertit automat în format JSON. Suportă argumentul opțional `limit` pentru a preveni supraîncărcarea memoriei pentru bazele de date voluminoase.
- `search_table`: Caută exact acele înregistrări care respectă unul sau mai multe criterii cheie-valoare. Suportă inclusiv potriviri parțiale pentru câmpurile de text.
- `create_table`: Formatează un nou fișier de bază de date cu o structură particularizată trimisă prin JSON *(necesită parametrul `--permit-editing` la lansare)*.
- `insert_record`: Adaugă un rând nou cu date, aliniate conform schemei tabelului *(necesită parametrul `--permit-editing`)*.
- `update_record`: Modifică informațiile dintr-o înregistrare existentă, pe baza indexului absolut din tabel *(necesită parametrul `--permit-editing`)*.

## Dezvoltare și Testare Locală

Acest proiect conține și un modul robust pentru asigurarea calității. Utilizând fișierul `Makefile`, automatizați procesele necesare verificării și asamblării:

- `make build`: Compilează serverul (execută faza de construcție a imaginii Docker).
- `make run`: Atașează clientul dvs. terminal direct la container (necesită existența unui folder `./data` la nivel local, unde vor fi stocate bazele de date în timpul dezvoltării).
- `make run-edit`: Pornește arhitectura ca mai sus, însă infuzează modul Write pentru manipulări DB (`--permit-editing`).
- `make test`: Apelează suita end-to-end integrată în limbajul Python (`tests/test_mcp.py`). Aceasta va simula un client complet JSON-RPC și va verifica ciclul complet de viață al unei baze de date (Creare tabel -> Inserare rând -> Căutare -> Update -> Finalizare), rafinând prevenția oricăror erori de corupere a memoriei specifice codului C nativ.
