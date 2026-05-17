// variants.jsx — three 10s NEXUS animations, each in its own Stage.
// Stage size: 540 × 960 (9:16). All copy in German.

const SAMPLE_CARDS = [
  { kind: 'IDEA', date: '16.05.26', body: 'brain Dump muss auch mit der Kamera vom Handy funktionieren und alle Informationen aus dem bild extrahieren…' },
  { kind: 'TASK', date: '16.05.26', body: 'Mustafa am Montag Ansprechen auf den USB-Stick mit Windows IOT' },
  { kind: 'TASK', date: '16.05.26', body: 'ich muss mit Cloud noch durchgehen ob die drei Projekte für die IHK in Ordnung gehen' },
  { kind: 'IDEA', date: '16.05.26', body: 'Tags automatisch vorschlagen basierend auf Kontext und Verlauf' },
  { kind: 'TASK', date: '16.05.26', body: 'Sprint Planning vorbereiten — Freitag 14:00 mit Cloud und Mustafa' },
  { kind: 'IDEA', date: '16.05.26', body: 'Voice-Memo-Aufnahme während Auto fahren, dann automatisch transkribieren' },
];

const HERO_ITEM = {
  kind: 'IDEA',
  dateLong: '16.05.2026, 21:07',
  body: 'brain Dump muss auch mit der Kamera vom Handy funktionieren und alle Informationen aus dem bild extrahieren und in kontext setzen, bzw mindestens einen Reiter erstellen wo direkt alle Infos aus bild textlich erfasst',
  summary: 'Entwicklung einer Funktion, die Informationen aus Bildern extrahiert und in einem Kontext setzt',
  tags: ['BRAIN DUMP', 'KAMERA', 'BILDANALYSE', 'TEXTERKENNUNG', 'NOTIZ-SYSTEM'],
  linked: 'Mustafa am Montag wegen USB-Stick mit Windows IOT ansprechen · 80%',
};

// ────────────────────────────────────────────────────────────────────────────
// Caption — bottom-anchored line of text with entry/hold/exit.
// ────────────────────────────────────────────────────────────────────────────
function NxCaption({ start, end, text, sub, y = 760 }) {
  return (
    <Sprite start={start} end={end}>
      {({ localTime, duration }) => {
        const exitStart = duration - 0.6;
        let opacity = 1, ty = 0;
        if (localTime < 0.45) {
          const t = Easing.easeOutCubic(clamp(localTime / 0.45, 0, 1));
          opacity = t; ty = (1 - t) * 14;
        } else if (localTime > exitStart) {
          const t = Easing.easeInCubic(clamp((localTime - exitStart) / 0.6, 0, 1));
          opacity = 1 - t; ty = -t * 10;
        }
        return (
          <div style={{
            position: 'absolute',
            left: 24, right: 24, top: y,
            opacity, transform: `translateY(${ty}px)`,
            textAlign: 'center',
            zIndex: 60,
          }}>
            <div style={{
              fontFamily: NX.font, fontSize: 30, fontWeight: 700,
              color: '#fff', letterSpacing: '-0.01em',
              lineHeight: 1.15,
              textShadow: '0 4px 24px rgba(11,12,17,0.9)',
            }}>{text}</div>
            {sub && (
              <div style={{
                fontFamily: NX.font, fontSize: 14,
                color: NX.textDim, marginTop: 8,
              }}>{sub}</div>
            )}
          </div>
        );
      }}
    </Sprite>
  );
}

// ────────────────────────────────────────────────────────────────────────────
// VARIANT A — Gedankenstrom
// Thoughts cascade in, one gets opened, tags pop, linked-to task surfaces.
// ────────────────────────────────────────────────────────────────────────────
function VariantA() {
  const t = useTime();

  // Card stagger: each card slides in starting at 0.4 + i*0.35s
  const cardCount = 6;
  const cardStarts = Array.from({ length: cardCount }, (_, i) => 0.5 + i * 0.32);

  // Detail sheet starts at 4.3s
  const sheetStart = 4.3;
  const sheetProg  = clamp((t - sheetStart) / 0.55, 0, 1);

  // Tags reveal one by one
  const tagsVisible = clamp(Math.floor((t - 5.0) / 0.18), 0, 5);
  const linkedVisible = t > 6.1;

  // Cursor moves to top card just before tap
  const cursorActive = t >= 3.6 && t < 4.4;
  const tapAt = 4.2;
  const cursorX = interpolate([3.6, 4.2], [420, 280])(clamp(t, 3.6, 4.2));
  const cursorY = interpolate([3.6, 4.2], [820, 330])(clamp(t, 3.6, 4.2));

  return (
    <NxScreen>
      <NxStatusBar/>
      <NxTopBar/>
      <NxFilterPills active="Alle"/>
      <NxSearch/>
      <NxAddButton/>

      {/* Card stream */}
      <div style={{ padding: '0 24px', display: 'flex', flexDirection: 'column', gap: 10 }}>
        {SAMPLE_CARDS.map((c, i) => {
          const start = cardStarts[i];
          const localT = clamp((t - start) / 0.42, 0, 1);
          if (t < start) return <div key={i} style={{ height: 0 }}/>;
          const ease = Easing.easeOutBack(localT);
          const opacity = clamp(localT * 2, 0, 1);
          const ty = (1 - ease) * 18;
          const isHero = i === 0;
          return (
            <div key={i} style={{
              opacity,
              transform: `translateY(${ty}px) scale(${0.96 + 0.04 * ease})`,
              transformOrigin: 'center top',
            }}>
              <NxCard
                kind={c.kind}
                date={c.date}
                body={c.body}
                highlight={isHero && t >= 4.0 && t < 4.5}
                faded={sheetProg > 0.4 && !isHero}
              />
            </div>
          );
        })}
      </div>

      {/* Dim overlay when sheet rises */}
      {sheetProg > 0.05 && (
        <div style={{
          position: 'absolute', inset: 0,
          background: `rgba(0,0,0,${sheetProg * 0.55})`,
          pointerEvents: 'none',
        }}/>
      )}

      {/* Detail sheet */}
      {sheetProg > 0 && (
        <NxDetailSheet
          progress={Easing.easeOutCubic(sheetProg)}
          item={HERO_ITEM}
          tagsVisible={tagsVisible}
          linkedVisible={linkedVisible}
        />
      )}

      <NxTabBar/>

      {/* Cursor */}
      {cursorActive && (
        <NxCursor x={cursorX} y={cursorY} tapping={Math.abs(t - tapAt) < 0.15}/>
      )}

      {/* Captions */}
      <NxCaption start={0.0} end={1.6} text="Der Kopf ist voll." y={420}/>
      <NxCaption start={2.0} end={3.6} text="Alles will raus, gleichzeitig." y={420}/>
      <NxCaption start={7.4} end={9.9} text="Für Köpfe, die nie stillstehen." sub="NEXUS — Braindump für ADHS-Workflows" y={140}/>
    </NxScreen>
  );
}

// ────────────────────────────────────────────────────────────────────────────
// VARIANT B — Foto → Text
// Tap +Braindump → camera → scan → OCR streams → tags → saved.
// ────────────────────────────────────────────────────────────────────────────
const OCR_LINES_FULL = [
  'Sprint Planning',
  '• Mustafa → USB-Stick',
  '• Cloud: 3 Projekte IHK',
  '• Demo Freitag 14h',
  '• Kamera-OCR testen',
  '!! Tags automatisch',
];

function VariantB() {
  const t = useTime();

  // Phases:
  // 0.0–1.0   list visible
  // 0.8–1.2   cursor moves to button, taps
  // 1.2–2.0   camera sheet slides up
  // 2.0–3.0   scanning
  // 3.0–3.3   shutter flash, captured
  // 3.3–6.5   OCR lines stream
  // 6.5–7.5   tags fade in below as chips
  // 7.5–9.0   saved → caption
  // 9.0–10.0  hold + loop reset

  const cameraStart = 1.2;
  const cameraProg = clamp((t - cameraStart) / 0.55, 0, 1);
  const captured = t >= 3.0;
  const saved = t >= 7.3;

  // OCR streaming: each line appears at 3.3 + i*0.5
  const ocrCount = clamp(Math.floor((t - 3.2) / 0.45) + 1, 0, OCR_LINES_FULL.length);
  const ocrLines = OCR_LINES_FULL.slice(0, ocrCount);

  // Cursor moves to + Braindump (y ~250, x ~270)
  const cursorActive = t >= 0.5 && t < 1.3;
  const cursorX = interpolate([0.5, 1.1], [440, 270])(clamp(t, 0.5, 1.1));
  const cursorY = interpolate([0.5, 1.1], [820, 252])(clamp(t, 0.5, 1.1));
  const tapping = Math.abs(t - 1.1) < 0.15;

  // Shutter flash
  const flashOpacity = (t >= 2.95 && t < 3.15) ? (1 - (t - 2.95) / 0.2) : 0;

  return (
    <NxScreen>
      <NxStatusBar/>
      <NxTopBar/>
      <NxFilterPills active="Alle"/>
      <NxSearch/>
      <NxAddButton glow={t > 0.9 && t < 1.4}/>

      <div style={{ padding: '0 24px', display: 'flex', flexDirection: 'column', gap: 10 }}>
        {SAMPLE_CARDS.slice(0, 3).map((c, i) => (
          <NxCard key={i} kind={c.kind} date={c.date} body={c.body}/>
        ))}
      </div>

      <NxTabBar/>

      {/* Camera sheet (full-screen) */}
      {cameraProg > 0 && (
        <NxCameraSheet
          progress={Easing.easeOutCubic(cameraProg)}
          captured={captured}
          ocrLines={ocrLines}
          saved={saved}
        />
      )}

      {/* Shutter flash overlay */}
      {flashOpacity > 0 && (
        <div style={{
          position: 'absolute', inset: 0,
          background: '#fff',
          opacity: flashOpacity,
          pointerEvents: 'none',
          zIndex: 100,
        }}/>
      )}

      {/* Cursor on the button */}
      {cursorActive && <NxCursor x={cursorX} y={cursorY} tapping={tapping}/>}

      {/* Captions */}
      <NxCaption start={0.0} end={1.0} text="Ein Foto." y={120}/>
      <NxCaption start={2.0} end={2.9} text="NEXUS scannt." y={120}/>
      <NxCaption start={3.4} end={6.5} text="Extrahiert Text…" y={120}/>
      <NxCaption start={7.4} end={9.9} text="Aus Zettel wird Notiz." sub="Foto → Text → Aufgabe. In Sekunden." y={350}/>
    </NxScreen>
  );
}

// ────────────────────────────────────────────────────────────────────────────
// VARIANT C — Komplett-Tour
// Blitz through all main screens, ends on logo + tagline.
// ────────────────────────────────────────────────────────────────────────────
function VariantC() {
  const t = useTime();

  // 5 beats × ~1.7s. Screens slide in from right, out to left.
  const beats = [
    { name: 'Dashboard',  in: 0.4, out: 2.0 },
    { name: 'Braindumps', in: 2.0, out: 3.7 },
    { name: 'Aufgaben',   in: 3.7, out: 5.4 },
    { name: 'Projekte',   in: 5.4, out: 7.1 },
    { name: 'Settings',   in: 7.1, out: 8.8 },
  ];

  // Helper: per-screen transform (slide in from right, hold, slide out to left).
  const screenTx = (inAt, outAt) => {
    const dur = 0.45;
    if (t < inAt - dur || t > outAt + dur) return { hidden: true };
    let x = 0, opacity = 1;
    if (t < inAt) {
      const p = (t - (inAt - dur)) / dur;
      const e = Easing.easeOutCubic(p);
      x = (1 - e) * 540;
      opacity = e;
    } else if (t > outAt) {
      const p = (t - outAt) / dur;
      const e = Easing.easeInCubic(p);
      x = -e * 540;
      opacity = 1 - e;
    }
    return { hidden: false, style: { transform: `translateX(${x}px)`, opacity } };
  };

  const dashTx  = screenTx(beats[0].in, beats[0].out);
  const brainTx = screenTx(beats[1].in, beats[1].out);
  const taskTx  = screenTx(beats[2].in, beats[2].out);
  const projTx  = screenTx(beats[3].in, beats[3].out);
  const setTx   = screenTx(beats[4].in, beats[4].out);

  // Settings: dark→light toggle animates 7.7–8.4
  const togglePct = 1 - clamp((t - 7.7) / 0.7, 0, 1);
  const bgLight = clamp((t - 7.9) / 0.4, 0, 1); // 0=dark, 1=light

  // End frame: 8.8 → 10
  const endStart = 8.7;
  const endProg = clamp((t - endStart) / 0.5, 0, 1);

  // Stat / task reveal scoped to their beat
  const dashReveal = clamp((t - 0.6) / 0.9, 0, 1);
  const taskChecked = [];
  if (t > 4.0) taskChecked.push(0);
  if (t > 4.4) taskChecked.push(1);
  if (t > 4.8) taskChecked.push(2);
  const projReveal = clamp(Math.floor((t - 5.6) / 0.25) + 1, 0, 4);

  // Active tab indicator follows beats
  const tabFor = (i) => beats[i].name === 'Settings' ? 'Mehr' : beats[i].name;
  let activeTab = 'Dashboard';
  beats.forEach(b => { if (t >= b.in - 0.2 && t <= b.out + 0.2) activeTab = b.name === 'Settings' ? 'Mehr' : b.name; });

  return (
    <NxScreen style={{ background: NX.bg }}>
      <NxStatusBar/>
      <NxTopBar dark={togglePct > 0.5}/>

      {/* Dashboard */}
      {!dashTx.hidden && (
        <div style={{ position: 'absolute', top: 80, left: 0, right: 0, ...dashTx.style }}>
          <NxDashboard statsReveal={dashReveal}/>
        </div>
      )}

      {/* Braindumps */}
      {!brainTx.hidden && (
        <div style={{ position: 'absolute', top: 80, left: 0, right: 0, ...brainTx.style }}>
          <NxFilterPills active="Alle"/>
          <NxAddButton/>
          <div style={{ padding: '0 24px', display: 'flex', flexDirection: 'column', gap: 10 }}>
            {SAMPLE_CARDS.slice(0, 4).map((c, i) => (
              <NxCard key={i} kind={c.kind} date={c.date} body={c.body}/>
            ))}
          </div>
        </div>
      )}

      {/* Aufgaben */}
      {!taskTx.hidden && (
        <div style={{ position: 'absolute', top: 80, left: 0, right: 0, ...taskTx.style }}>
          <NxTasksView checked={taskChecked}/>
        </div>
      )}

      {/* Projekte */}
      {!projTx.hidden && (
        <div style={{ position: 'absolute', top: 80, left: 0, right: 0, ...projTx.style }}>
          <NxProjectsView reveal={projReveal}/>
        </div>
      )}

      {/* Settings (with toggle animation) */}
      {!setTx.hidden && (
        <div style={{ position: 'absolute', top: 80, left: 0, right: 0, ...setTx.style }}>
          <NxSettingsView darkOn={togglePct > 0.5} togglePct={togglePct}/>
        </div>
      )}

      {/* Light-mode background wash */}
      {bgLight > 0 && endProg < 0.05 && (
        <div style={{
          position: 'absolute', inset: 0,
          background: '#f5f4ef',
          opacity: bgLight * 0.0, // intentional: keep dark, just preview via NxToggle color
          pointerEvents: 'none',
        }}/>
      )}

      <NxTabBar active={activeTab}/>

      {/* End frame */}
      {endProg > 0 && (
        <div style={{
          position: 'absolute', inset: 0,
          background: NX.bg,
          display: 'flex', flexDirection: 'column',
          alignItems: 'center', justifyContent: 'center',
          opacity: Easing.easeOutCubic(endProg),
          zIndex: 100,
        }}>
          <div style={{
            fontFamily: NX.font, fontWeight: 800,
            fontSize: 64, letterSpacing: '0.20em',
            color: NX.text,
            transform: `scale(${0.92 + 0.08 * endProg})`,
            marginBottom: 18,
          }}>NEXUS</div>
          <div style={{
            fontFamily: NX.font, fontSize: 20, color: NX.textDim,
            textAlign: 'center', padding: '0 40px', lineHeight: 1.4,
          }}>Alles, was im Kopf rumort —<br/>an einem Ort.</div>
          <div style={{
            marginTop: 28, padding: '10px 20px',
            background: NX.purple, borderRadius: 999,
            fontFamily: NX.font, fontWeight: 600, fontSize: 13,
            color: '#fff', letterSpacing: '0.04em',
            boxShadow: `0 8px 24px rgba(124,121,255,0.4)`,
          }}>Für vielbeschäftigte Köpfe</div>
        </div>
      )}

      {/* Beat labels (subtle, lower-third style) */}
      {beats.map((b, i) => (
        <Sprite key={i} start={b.in + 0.2} end={b.out - 0.1}>
          {({ localTime, duration }) => {
            let opacity = 1;
            if (localTime < 0.3) opacity = localTime / 0.3;
            else if (localTime > duration - 0.3) opacity = (duration - localTime) / 0.3;
            return (
              <div style={{
                position: 'absolute', left: 24, bottom: 110,
                fontFamily: NX.font, fontSize: 11, fontWeight: 700,
                letterSpacing: '0.16em', color: NX.purpleText,
                opacity, zIndex: 80,
              }}>0{i + 1} · {b.name.toUpperCase()}</div>
            );
          }}
        </Sprite>
      ))}
    </NxScreen>
  );
}

Object.assign(window, { VariantA, VariantB, VariantC, SAMPLE_CARDS, HERO_ITEM });
