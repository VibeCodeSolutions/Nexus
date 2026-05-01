# NEXUS — Sync zwischen Geräten

Wie Tasks, Braindumps und Projekte zwischen Desktop und Mobil sichtbar werden, was funktioniert, was nicht.

## Modell auf einen Blick

NEXUS hat **einen Core**. Der Core ist ein HTTP-Server (axum, Port 7777) mit einer SQLite-DB unter `~/.nexus/nexus.db`. Alle Clients (Desktop-Tauri-UI, Android-App) sprechen denselben Core. Wenn der Core läuft, sehen alle gepairten Geräte denselben Datenstand.

```
                       ┌──────────────────┐
                       │  ~/.nexus/       │
                       │  └ nexus.db      │
                       │  └ keystore/     │
                       │  └ .nexus_token  │
                       └────────▲─────────┘
                                │
                  ┌─────────────┴─────────────┐
                  │  NEXUS Core (axum:7777)   │
                  │  bind: 0.0.0.0            │
                  └──┬─────────────────────┬──┘
                     │                     │
              ┌──────▼──────┐       ┌──────▼──────┐
              │ Desktop UI  │       │ Pixel-App   │
              │ (Tauri)     │       │ (Ktor)      │
              │ localhost   │       │ LAN-IP      │
              └─────────────┘       └─────────────┘
```

**Konsequenz:** Wenn der Core nicht läuft, ist die App ein UI ohne Daten. Wenn der Core läuft (z.B. Desktop offen oder als Dienst), sind die Geräte cross-sichtbar — vorausgesetzt das Mobilgerät kommt an die Core-IP heran.

## Unterstützte Topologien

Diese Setups funktionieren ohne weitere Konfiguration:

- ✅ Desktop + Pixel im **selben LAN-Subnetz** (z.B. beide am Heim-Router, beide auf 192.168.x.0/24)
- ✅ Desktop läuft, Pixel scannt QR und pairt mit der LAN-IP des Desktops
- ✅ Headless-Modus: Core läuft als systemd-Service (`nexus-core.service`) auf demselben Rechner oder einem Heimserver — Desktop-UI verbindet auf `localhost`, Pixel auf die LAN-IP des Servers

## Out of Scope (das funktioniert NICHT, ist auch nicht geplant)

Wenn dein Bug einer dieser Fälle ist, ist es **kein Bug, sondern eine Architekturentscheidung**:

- ❌ **Pixel auf 4G / Mobile Daten** — keine Cloud-Bridge, kein NAT-Traversal. Der Core lebt im LAN, das Mobilgerät auch.
- ❌ **Pixel auf VPN, Desktop nicht** (oder umgekehrt) — beide Devices müssen im selben Routing-Bereich sein.
- ❌ **Pixel im Gast-WLAN** — viele Router haben Client-Isolation aktiv. Die LAN-IP des Desktops ist dann nicht erreichbar, auch wenn das Subnetz gleich aussieht.
- ❌ **Multi-Subnet-Setups** — kein automatisches Routing zwischen 10.x und 192.168.x. Wer das will, baut sein Routing selbst.
- ❌ **mDNS / Bonjour-Discovery** — nicht implementiert. Das Pairing erfordert die direkte IP im QR-Code.
- ❌ **Cloud-Sync / Multi-User** — NEXUS ist Single-User-Personal-OS. Es gibt keinen Hosting-Service.

## Failure-Modes

Wenn das Pairing oder Sync nicht klappt, hier die typischen Ursachen:

| Symptom | Wahrscheinliche Ursache | Diagnose-Schritt |
|---|---|---|
| Pixel-App: "Core nicht erreichbar" nach QR-Scan | Falsches Subnetz oder Mobile-Daten aktiv | `adb shell ip route` auf dem Pixel — gleiches /24 wie Desktop? |
| QR enthält `127.0.0.1` oder `0.0.0.0` | `local_ip_address::local_ip()` konnte keine Default-Route finden | Server-Log beim Start prüfen ("Mobile Pairing erwartet IP http://…"). Default-Route auf dem Desktop verifizieren. |
| QR enthält "falsche" LAN-IP (z.B. Docker-Bridge `172.17.x.x`) | Multi-Interface-Setup, Krate wählt das erste | `cat /proc/net/route` — welche Default-Route? Ggf. Docker-Bridge stoppen während des Pairings, oder `NEXUS_BIND_ADDR=192.168.x.y:7777` setzen und manuell pairen. |
| Token-401 nach Re-Pair | Server-`~/.nexus_token` regeneriert nach Server-Neustart | Im Wizard erneut pairen (in Zukunft: Phase-C "Wizard neustarten"-Button) |
| Tasks vom Pixel erscheinen am Desktop nicht | Desktop hat alte Liste im Cache | Refresh-Button im Tasks-Tab klicken (Phase A: jetzt mit sichtbarem Loading-State + Banner bei API-Fehler) |
| Pairing-QR scannt korrekt, aber Pixel zeigt sofort "Disconnect" | Firewall blockt eingehende Verbindungen auf 7777 | Auf dem Desktop: `firewall-cmd --add-port=7777/tcp --zone=FedoraWorkstation` (temporär) bzw. `--permanent --reload` |

## Workarounds (kein First-Class-Support)

Wenn du dein Setup über das Single-LAN-Modell hinaus heben willst, geht das mit User-Tools — wir dokumentieren sie, wir testen sie nicht und geben keine Garantien:

- **Tailscale / ZeroTier** — Mesh-VPN. Desktop und Pixel werden Members im selben Tailnet. Beide sehen sich über die Tailscale-IP. QR muss dann die Tailscale-IP enthalten — entweder `NEXUS_BIND_ADDR=100.x.y.z:7777` setzen, oder im QR die IP manuell ändern. Funktioniert auf 4G.
- **WireGuard** — eigener Tunnel-Server, dasselbe Prinzip wie Tailscale, mehr Setup.
- **Reverse-SSH-Tunnel** — wenn Desktop hinter einer dynamischen IP sitzt und du einen statischen Hop hast. Eher Hack als Lösung.

## Headless-Service-Pfad

Für 24/7-Verfügbarkeit ohne dass das Desktop-Programm laufen muss:

```bash
# Service installieren (einmalig)
cd ~/Projekte/Apps/Nexus/core
./install-service.sh

# Service starten
systemctl --user start nexus-core
systemctl --user enable nexus-core   # auto-start nach Login

# Status / Logs
systemctl --user status nexus-core
journalctl --user -u nexus-core -f
```

Das Desktop-Tauri-UI erkennt einen laufenden Core auf 7777 und startet keinen zweiten Sidecar (siehe Phase D — Single-Core-Garant).

## Diagnose

Wenn etwas klemmt, in dieser Reihenfolge prüfen:

1. **Server-Log beim Start** — die Zeile `Mobile Pairing erwartet IP http://…:7777` zeigt, welche IP der QR enthält. Wenn da `127.0.0.1` steht, ist `local_ip_address` fehlgeschlagen — typisch bei systemd-Hardening, fehlender Default-Route oder Container-Netzwerk.
2. **Desktop ↔ Pixel Subnetz-Match** — beide auf 192.168.x.0/24 oder 10.0.x.0/24, mit gleicher x?
3. **Firewall** — auf dem Desktop Port 7777/tcp eingehend offen? Default-FedoraWorkstation-Zone hat das nicht.
4. **Server lebt** — `curl http://192.168.x.y:7777/health` vom Pixel-LAN aus. Sollte `{"status":"ok"}` liefern.
5. **Token-Match** — wenn `/health` ohne Auth geht, aber `/tasks` 401 wirft, ist das Pairing-Token alt. Re-Pair.

Bei Verdacht auf Bug: Issue mit Server-Log + dem Output dieser Diagnoseschritte. Spekulation kostet alle Zeit.
