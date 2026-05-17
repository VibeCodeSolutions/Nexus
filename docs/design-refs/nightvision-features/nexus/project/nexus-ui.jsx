// nexus-ui.jsx — Portrait/mobile NEXUS app components.
// Colors and copy lifted from the user's screenshot, restyled for 9:16.

const NX = {
  bg:         '#0b0c11',
  bgElev:     '#13141c',
  bgCard:     '#1a1c27',
  bgCardHi:   '#22253348',
  border:     'rgba(255,255,255,0.06)',
  borderHi:   'rgba(255,255,255,0.12)',
  text:       '#e6e7ee',
  textDim:    '#9094a6',
  textMute:   '#5b5f72',
  purple:     '#7c79ff',
  purpleSoft: '#4a4769',
  purpleText: '#b9b7ff',
  coral:      '#c95c5c',
  coralSoft:  'rgba(201,92,92,0.16)',
  green:      '#36c97a',
  amber:      '#d4a04c',
  font:       '"Inter", system-ui, sans-serif',
  mono:       '"JetBrains Mono", ui-monospace, monospace',
};

// ── Top bar with NEXUS logo + status dot + dark toggle + gear ──────────────
function NxTopBar({ status = 'VERBUNDEN', dark = true, settingsHi = false }) {
  return (
    <div style={{
      display: 'flex', alignItems: 'center', justifyContent: 'space-between',
      padding: '20px 24px 16px',
      borderBottom: `1px solid ${NX.border}`,
    }}>
      <div style={{
        fontFamily: NX.font,
        fontWeight: 800,
        fontSize: 22,
        letterSpacing: '0.18em',
        color: NX.text,
      }}>NEXUS</div>

      <div style={{
        display: 'flex', alignItems: 'center', gap: 6,
        padding: '6px 12px',
        background: 'rgba(54,201,122,0.10)',
        border: `1px solid rgba(54,201,122,0.30)`,
        borderRadius: 999,
        color: NX.green,
        fontSize: 11,
        fontWeight: 700,
        letterSpacing: '0.12em',
      }}>
        <span style={{ width: 7, height: 7, borderRadius: 999, background: NX.green, boxShadow: `0 0 8px ${NX.green}` }}/>
        {status}
      </div>

      <div style={{ display: 'flex', gap: 8 }}>
        <NxIconBtn>
          {dark ? <NxMoonIcon/> : <NxSunIcon/>}
        </NxIconBtn>
        <NxIconBtn highlight={settingsHi}>
          <NxGearIcon/>
        </NxIconBtn>
      </div>
    </div>
  );
}

function NxIconBtn({ children, highlight = false }) {
  return (
    <div style={{
      width: 36, height: 36,
      display: 'flex', alignItems: 'center', justifyContent: 'center',
      background: highlight ? 'rgba(124,121,255,0.18)' : NX.bgElev,
      border: `1px solid ${highlight ? NX.purple : NX.border}`,
      borderRadius: 10,
      color: highlight ? NX.purple : NX.textDim,
      transition: 'all 200ms',
    }}>{children}</div>
  );
}

// ── Filter pills row ───────────────────────────────────────────────────────
function NxFilterPills({ active = 'Alle' }) {
  const pills = ['Alle', 'Idea', 'Task'];
  return (
    <div style={{ display: 'flex', gap: 8, padding: '14px 24px 8px' }}>
      {pills.map(p => {
        const on = p === active;
        return (
          <div key={p} style={{
            padding: '8px 18px',
            borderRadius: 999,
            background: on ? NX.purple : 'transparent',
            color: on ? '#fff' : NX.textDim,
            border: `1px solid ${on ? NX.purple : NX.border}`,
            fontFamily: NX.font,
            fontWeight: 600,
            fontSize: 14,
          }}>{p}</div>
        );
      })}
    </div>
  );
}

// ── Search field ───────────────────────────────────────────────────────────
function NxSearch({ placeholder = 'Suchen…', value = '' }) {
  return (
    <div style={{ padding: '6px 24px 8px' }}>
      <div style={{
        padding: '12px 16px',
        background: NX.bgElev,
        border: `1px solid ${NX.border}`,
        borderRadius: 12,
        color: value ? NX.text : NX.textMute,
        fontFamily: NX.font,
        fontSize: 14,
      }}>{value || placeholder}</div>
    </div>
  );
}

// ── + Braindump primary button ────────────────────────────────────────────
function NxAddButton({ label = '+ Braindump', glow = false }) {
  return (
    <div style={{ padding: '6px 24px 14px' }}>
      <div style={{
        padding: '16px 0',
        textAlign: 'center',
        background: NX.purple,
        borderRadius: 12,
        color: '#fff',
        fontFamily: NX.font,
        fontWeight: 600,
        fontSize: 15,
        boxShadow: glow ? `0 8px 24px rgba(124,121,255,0.55), 0 0 0 6px rgba(124,121,255,0.12)` : `0 4px 14px rgba(124,121,255,0.30)`,
        transition: 'box-shadow 200ms',
      }}>{label}</div>
    </div>
  );
}

// ── Card (idea / task) ─────────────────────────────────────────────────────
function NxCard({ kind = 'IDEA', date = '16.05.26', body, highlight = false, faded = false }) {
  const kindColor = kind === 'IDEA' ? NX.purpleText : NX.green;
  const kindBg    = kind === 'IDEA' ? 'rgba(124,121,255,0.14)' : 'rgba(54,201,122,0.14)';
  return (
    <div style={{
      padding: '14px 16px',
      background: NX.bgCard,
      border: `1px solid ${highlight ? NX.purple : NX.border}`,
      borderRadius: 14,
      opacity: faded ? 0.4 : 1,
      boxShadow: highlight ? `0 0 0 4px rgba(124,121,255,0.16)` : 'none',
      transition: 'all 200ms',
    }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 8 }}>
        <div style={{
          padding: '3px 9px',
          background: kindBg,
          borderRadius: 6,
          color: kindColor,
          fontFamily: NX.font,
          fontWeight: 700,
          fontSize: 10,
          letterSpacing: '0.10em',
        }}>{kind}</div>
        <div style={{ fontFamily: NX.font, fontSize: 11, color: NX.textMute }}>{date}</div>
      </div>
      <div style={{
        fontFamily: NX.font,
        fontSize: 14,
        lineHeight: 1.45,
        color: NX.text,
        display: '-webkit-box',
        WebkitLineClamp: 2,
        WebkitBoxOrient: 'vertical',
        overflow: 'hidden',
      }}>{body}</div>
    </div>
  );
}

// ── Bottom tab bar ─────────────────────────────────────────────────────────
function NxTabBar({ active = 'Braindumps' }) {
  const tabs = [
    { id: 'Dashboard',   icon: <NxHomeIcon/> },
    { id: 'Braindumps',  icon: <NxBrainIcon/> },
    { id: 'Aufgaben',    icon: <NxCheckIcon/> },
    { id: 'Projekte',    icon: <NxFolderIcon/> },
    { id: 'Mehr',        icon: <NxMoreIcon/> },
  ];
  return (
    <div style={{
      position: 'absolute', bottom: 0, left: 0, right: 0,
      display: 'flex',
      padding: '12px 12px 22px',
      background: 'linear-gradient(to top, ' + NX.bg + ' 60%, rgba(11,12,17,0))',
      borderTop: `1px solid ${NX.border}`,
    }}>
      {tabs.map(t => {
        const on = t.id === active;
        return (
          <div key={t.id} style={{
            flex: 1,
            display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 4,
            color: on ? NX.purple : NX.textMute,
          }}>
            <div style={{ width: 22, height: 22 }}>{t.icon}</div>
            <div style={{ fontFamily: NX.font, fontSize: 10, fontWeight: 600 }}>{t.id}</div>
          </div>
        );
      })}
    </div>
  );
}

// ── Tag pill (used in detail sheet) ────────────────────────────────────────
function NxTag({ children }) {
  return (
    <div style={{
      padding: '5px 10px',
      background: NX.purpleSoft,
      borderRadius: 6,
      color: NX.purpleText,
      fontFamily: NX.font,
      fontWeight: 700,
      fontSize: 10,
      letterSpacing: '0.10em',
      display: 'inline-block',
    }}>{children}</div>
  );
}

// ── Detail sheet (the right-side panel from the screenshot, but a bottom sheet)
function NxDetailSheet({ progress = 1, item, tagsVisible = 0, linkedVisible = false }) {
  // progress 0..1 controls slide-up
  const ty = (1 - progress) * 100;
  return (
    <div style={{
      position: 'absolute', left: 0, right: 0, bottom: 0,
      transform: `translateY(${ty}%)`,
      background: NX.bgElev,
      borderTop: `1px solid ${NX.borderHi}`,
      borderTopLeftRadius: 22,
      borderTopRightRadius: 22,
      padding: '20px 22px 32px',
      maxHeight: '78%',
      boxShadow: '0 -20px 60px rgba(0,0,0,0.6)',
    }}>
      <div style={{
        width: 44, height: 4, background: NX.borderHi,
        borderRadius: 999, margin: '0 auto 16px',
      }}/>

      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 14 }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
          <div style={{
            padding: '4px 10px',
            background: 'rgba(124,121,255,0.14)',
            borderRadius: 6,
            color: NX.purpleText,
            fontFamily: NX.font,
            fontWeight: 700,
            fontSize: 10,
            letterSpacing: '0.10em',
          }}>{item.kind}</div>
          <div style={{ fontFamily: NX.font, fontSize: 12, color: NX.textDim }}>{item.dateLong || '16.05.2026, 21:07'}</div>
        </div>
        <div style={{ color: NX.textMute, fontSize: 18 }}>×</div>
      </div>

      <div style={{
        fontFamily: NX.font,
        fontSize: 15,
        lineHeight: 1.5,
        color: NX.text,
        marginBottom: 14,
      }}>{item.body}</div>

      {item.summary && (
        <div style={{
          padding: '12px 14px',
          background: 'rgba(255,255,255,0.04)',
          border: `1px solid ${NX.border}`,
          borderRadius: 10,
          fontFamily: NX.font,
          fontSize: 12,
          lineHeight: 1.5,
          color: NX.textDim,
          marginBottom: 14,
        }}>{item.summary}</div>
      )}

      {item.tags && (
        <div style={{ display: 'flex', flexWrap: 'wrap', gap: 6, marginBottom: 16 }}>
          {item.tags.slice(0, tagsVisible).map((t, i) => (
            <div key={i} style={{
              opacity: 1,
              animation: 'none',
            }}>
              <NxTag>{t}</NxTag>
            </div>
          ))}
        </div>
      )}

      {item.linked && linkedVisible && (
        <div>
          <div style={{
            fontFamily: NX.font,
            fontSize: 10,
            fontWeight: 700,
            letterSpacing: '0.12em',
            color: NX.textMute,
            marginBottom: 8,
          }}>VERKNÜPFT MIT</div>
          <div style={{
            padding: '12px 14px',
            background: NX.bgCard,
            border: `1px solid ${NX.border}`,
            borderRadius: 10,
            fontFamily: NX.font,
            fontSize: 13,
            color: NX.text,
            display: 'flex', alignItems: 'center', gap: 8,
          }}>
            <span>📋</span>
            <span>{item.linked}</span>
          </div>
        </div>
      )}
    </div>
  );
}

// ── Camera capture sheet (viewfinder + analyzed text) ──────────────────────
function NxCameraSheet({ progress = 1, captured = false, ocrLines = [], saved = false }) {
  const ty = (1 - progress) * 100;
  return (
    <div style={{
      position: 'absolute', left: 0, right: 0, bottom: 0, top: 0,
      transform: `translateY(${ty}%)`,
      background: NX.bg,
      display: 'flex', flexDirection: 'column',
    }}>
      <div style={{
        padding: '20px 22px',
        display: 'flex', justifyContent: 'space-between', alignItems: 'center',
        borderBottom: `1px solid ${NX.border}`,
      }}>
        <div style={{ color: NX.textDim, fontSize: 20 }}>←</div>
        <div style={{ fontFamily: NX.font, fontWeight: 600, fontSize: 14, color: NX.text }}>Foto-Braindump</div>
        <div style={{ width: 20 }}/>
      </div>

      {/* Viewfinder */}
      <div style={{
        flex: 1,
        position: 'relative',
        margin: '20px 22px',
        background: '#000',
        borderRadius: 18,
        overflow: 'hidden',
        border: `1px solid ${NX.border}`,
      }}>
        {/* Simulated paper note photo */}
        <div style={{
          position: 'absolute', inset: 0,
          background: 'linear-gradient(135deg, #2a2419 0%, #1a1610 100%)',
        }}>
          <div style={{
            position: 'absolute',
            top: '14%', left: '12%', right: '12%', bottom: '14%',
            background: '#f3ecd6',
            transform: 'rotate(-3deg)',
            boxShadow: '0 16px 40px rgba(0,0,0,0.55)',
            padding: '24px 22px',
            fontFamily: '"Caveat", "Comic Sans MS", cursive',
            fontSize: 17,
            lineHeight: 1.55,
            color: '#3a2c1c',
          }}>
            <div style={{ fontWeight: 700, fontSize: 19, marginBottom: 10 }}>Sprint Planning</div>
            <div>• Mustafa → USB-Stick</div>
            <div>• Cloud: 3 Projekte IHK</div>
            <div>• Demo Freitag 14h</div>
            <div>• Kamera-OCR testen</div>
            <div style={{ marginTop: 8, fontSize: 14 }}>!! Tags automatisch</div>
          </div>
        </div>

        {/* Scan overlay */}
        {!captured && (
          <>
            <NxScanCorners/>
            <div style={{
              position: 'absolute', left: 0, right: 0,
              top: '50%', height: 2,
              background: 'linear-gradient(to right, transparent, ' + NX.purple + ', transparent)',
              boxShadow: `0 0 12px ${NX.purple}`,
            }}/>
          </>
        )}

        {/* Analyzing chip */}
        {captured && !saved && (
          <div style={{
            position: 'absolute', bottom: 16, left: '50%',
            transform: 'translateX(-50%)',
            padding: '8px 16px',
            background: 'rgba(11,12,17,0.85)',
            border: `1px solid ${NX.purple}`,
            borderRadius: 999,
            color: NX.purpleText,
            fontFamily: NX.font,
            fontWeight: 600,
            fontSize: 12,
            letterSpacing: '0.06em',
            display: 'flex', alignItems: 'center', gap: 8,
          }}>
            <NxSpinner/>
            Analysiere…
          </div>
        )}

        {saved && (
          <div style={{
            position: 'absolute', bottom: 16, left: '50%',
            transform: 'translateX(-50%)',
            padding: '8px 16px',
            background: 'rgba(54,201,122,0.18)',
            border: `1px solid ${NX.green}`,
            borderRadius: 999,
            color: NX.green,
            fontFamily: NX.font,
            fontWeight: 700,
            fontSize: 12,
            letterSpacing: '0.06em',
          }}>✓ Gespeichert</div>
        )}
      </div>

      {/* OCR result panel */}
      <div style={{
        margin: '0 22px 22px',
        padding: '14px 16px',
        background: NX.bgCard,
        border: `1px solid ${NX.border}`,
        borderRadius: 12,
        minHeight: 110,
      }}>
        <div style={{
          fontFamily: NX.font,
          fontSize: 10,
          fontWeight: 700,
          letterSpacing: '0.12em',
          color: NX.textMute,
          marginBottom: 10,
        }}>EXTRAHIERTER TEXT</div>
        <div style={{
          fontFamily: NX.mono,
          fontSize: 12,
          lineHeight: 1.55,
          color: NX.text,
          whiteSpace: 'pre-wrap',
          minHeight: 60,
        }}>{ocrLines.join('\n')}<span style={{ opacity: 0.6, color: NX.purple }}>{captured && !saved && ocrLines.length < 5 ? '▌' : ''}</span></div>
      </div>

      {/* Shutter */}
      {!captured && (
        <div style={{ display: 'flex', justifyContent: 'center', padding: '0 0 28px' }}>
          <div style={{
            width: 68, height: 68,
            borderRadius: 999,
            border: `4px solid ${NX.purple}`,
            background: '#fff',
            boxShadow: `0 0 24px rgba(124,121,255,0.6)`,
          }}/>
        </div>
      )}
    </div>
  );
}

function NxScanCorners() {
  const c = { position: 'absolute', width: 28, height: 28, borderColor: NX.purple, borderStyle: 'solid', borderWidth: 0 };
  return (
    <>
      <div style={{ ...c, top: 12, left: 12,    borderTopWidth: 3, borderLeftWidth: 3,  borderTopLeftRadius: 8  }}/>
      <div style={{ ...c, top: 12, right: 12,   borderTopWidth: 3, borderRightWidth: 3, borderTopRightRadius: 8 }}/>
      <div style={{ ...c, bottom: 12, left: 12, borderBottomWidth: 3, borderLeftWidth: 3, borderBottomLeftRadius: 8 }}/>
      <div style={{ ...c, bottom: 12, right: 12,borderBottomWidth: 3, borderRightWidth: 3, borderBottomRightRadius: 8 }}/>
    </>
  );
}

function NxSpinner() {
  return (
    <div style={{
      width: 12, height: 12,
      borderRadius: 999,
      border: `2px solid ${NX.purpleText}`,
      borderTopColor: 'transparent',
      animation: 'nx-spin 0.8s linear infinite',
    }}/>
  );
}

// ── Cursor (tap indicator) ────────────────────────────────────────────────
function NxCursor({ x, y, tapping = false }) {
  return (
    <div style={{
      position: 'absolute', left: x, top: y,
      transform: 'translate(-30%, -30%)',
      pointerEvents: 'none',
      zIndex: 50,
      filter: 'drop-shadow(0 2px 4px rgba(0,0,0,0.5))',
    }}>
      <svg width="28" height="32" viewBox="0 0 28 32">
        <path d="M3 2 L3 24 L9 19 L13 28 L17 27 L13 18 L21 18 Z" fill="#fff" stroke="#0b0c11" strokeWidth="1.5" strokeLinejoin="round"/>
      </svg>
      {tapping && (
        <div style={{
          position: 'absolute', left: 6, top: 6,
          width: 36, height: 36,
          borderRadius: 999,
          border: `2px solid ${NX.purple}`,
          animation: 'nx-tap 0.5s ease-out',
        }}/>
      )}
    </div>
  );
}

// ── Dashboard tiles (overview view) ────────────────────────────────────────
function NxDashboard({ statsReveal = 1 }) {
  const stats = [
    { label: 'Heute',          value: '7',  sub: 'Braindumps' },
    { label: 'Offen',          value: '23', sub: 'Aufgaben' },
    { label: 'Aktive',         value: '4',  sub: 'Projekte' },
    { label: 'Diese Woche',    value: '12', sub: 'Erledigt' },
  ];
  return (
    <div style={{ padding: '14px 24px' }}>
      <div style={{
        fontFamily: NX.font, fontSize: 24, fontWeight: 700, color: NX.text,
        marginBottom: 4,
      }}>Guten Abend.</div>
      <div style={{
        fontFamily: NX.font, fontSize: 13, color: NX.textDim, marginBottom: 18,
      }}>Du hast 23 offene Aufgaben.</div>

      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 10, marginBottom: 16 }}>
        {stats.map((s, i) => (
          <div key={i} style={{
            padding: '16px 14px',
            background: NX.bgCard,
            border: `1px solid ${NX.border}`,
            borderRadius: 14,
            opacity: i < Math.ceil(statsReveal * 4) ? 1 : 0,
            transform: i < Math.ceil(statsReveal * 4) ? 'translateY(0)' : 'translateY(8px)',
            transition: 'all 220ms',
          }}>
            <div style={{ fontFamily: NX.font, fontSize: 10, fontWeight: 700, letterSpacing: '0.10em', color: NX.textMute }}>{s.label.toUpperCase()}</div>
            <div style={{ fontFamily: NX.font, fontSize: 32, fontWeight: 700, color: i === 0 ? NX.purple : NX.text, lineHeight: 1.1, margin: '6px 0 2px' }}>{s.value}</div>
            <div style={{ fontFamily: NX.font, fontSize: 12, color: NX.textDim }}>{s.sub}</div>
          </div>
        ))}
      </div>

      <div style={{
        padding: 16,
        background: NX.bgCard,
        border: `1px solid ${NX.border}`,
        borderRadius: 14,
      }}>
        <div style={{ fontFamily: NX.font, fontSize: 10, fontWeight: 700, letterSpacing: '0.10em', color: NX.textMute, marginBottom: 10 }}>NÄCHSTER FOKUS</div>
        <div style={{ fontFamily: NX.font, fontSize: 14, color: NX.text, lineHeight: 1.45 }}>
          Sprint Planning vorbereiten — Freitag, 14:00
        </div>
      </div>
    </div>
  );
}

// ── Tasks view ─────────────────────────────────────────────────────────────
function NxTasksView({ checked = [] }) {
  const tasks = [
    'Mustafa am Montag wegen USB-Stick',
    'Cloud: drei IHK-Projekte durchgehen',
    'Sprint Planning vorbereiten',
    'Steuererklärung finalisieren',
    'Kamera-OCR im Beta-Build testen',
  ];
  return (
    <div style={{ padding: '14px 24px' }}>
      <div style={{ fontFamily: NX.font, fontSize: 22, fontWeight: 700, color: NX.text, marginBottom: 16 }}>Aufgaben</div>
      <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
        {tasks.map((t, i) => {
          const on = checked.includes(i);
          return (
            <div key={i} style={{
              display: 'flex', alignItems: 'center', gap: 12,
              padding: '14px 14px',
              background: NX.bgCard,
              border: `1px solid ${NX.border}`,
              borderRadius: 12,
            }}>
              <div style={{
                width: 22, height: 22,
                borderRadius: 7,
                border: `2px solid ${on ? NX.green : NX.borderHi}`,
                background: on ? NX.green : 'transparent',
                display: 'flex', alignItems: 'center', justifyContent: 'center',
                transition: 'all 200ms',
              }}>
                {on && <svg width="12" height="12" viewBox="0 0 12 12"><path d="M2 6 L5 9 L10 3" stroke="#0b0c11" strokeWidth="2.4" fill="none" strokeLinecap="round" strokeLinejoin="round"/></svg>}
              </div>
              <div style={{
                fontFamily: NX.font,
                fontSize: 13,
                color: on ? NX.textMute : NX.text,
                textDecoration: on ? 'line-through' : 'none',
                lineHeight: 1.4,
                flex: 1,
              }}>{t}</div>
            </div>
          );
        })}
      </div>
    </div>
  );
}

// ── Projects view ─────────────────────────────────────────────────────────
function NxProjectsView({ reveal = 4 }) {
  const projects = [
    { name: 'IHK Zertifizierung', tag: 'AKTIV',   color: NX.purple, pct: 80 },
    { name: 'Windows-IOT Pilot',  tag: 'AKTIV',   color: NX.amber,  pct: 45 },
    { name: 'NEXUS v0.4',         tag: 'AKTIV',   color: NX.green,  pct: 62 },
    { name: 'Steuer 2025',        tag: 'PAUSIERT',color: NX.coral,  pct: 30 },
  ];
  return (
    <div style={{ padding: '14px 24px' }}>
      <div style={{ fontFamily: NX.font, fontSize: 22, fontWeight: 700, color: NX.text, marginBottom: 16 }}>Projekte</div>
      <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
        {projects.map((p, i) => {
          const shown = i < reveal;
          return (
            <div key={i} style={{
              padding: '14px 16px',
              background: NX.bgCard,
              border: `1px solid ${NX.border}`,
              borderRadius: 12,
              opacity: shown ? 1 : 0,
              transform: shown ? 'translateX(0)' : 'translateX(20px)',
              transition: 'all 260ms',
            }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 10 }}>
                <div style={{ fontFamily: NX.font, fontSize: 14, fontWeight: 600, color: NX.text }}>{p.name}</div>
                <div style={{ fontFamily: NX.font, fontSize: 9, fontWeight: 700, letterSpacing: '0.10em', color: p.color }}>{p.tag}</div>
              </div>
              <div style={{ height: 4, background: NX.border, borderRadius: 999, overflow: 'hidden' }}>
                <div style={{ width: `${p.pct}%`, height: '100%', background: p.color, borderRadius: 999, transition: 'width 400ms' }}/>
              </div>
              <div style={{ marginTop: 6, display: 'flex', justifyContent: 'space-between', fontFamily: NX.font, fontSize: 10, color: NX.textMute }}>
                <span>{p.pct}%</span>
                <span>{Math.round(p.pct / 10)} / 10 Schritte</span>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}

// ── Settings view with dark-mode toggle ───────────────────────────────────
function NxSettingsView({ darkOn = true, togglePct = 1 }) {
  return (
    <div style={{ padding: '14px 24px' }}>
      <div style={{ fontFamily: NX.font, fontSize: 22, fontWeight: 700, color: NX.text, marginBottom: 16 }}>Einstellungen</div>

      <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
        <NxSetRow label="Erscheinungsbild" sub={darkOn ? 'Dunkel' : 'Hell'}>
          <NxToggle on={darkOn} pct={togglePct}/>
        </NxSetRow>
        <NxSetRow label="Kamera-Analyse" sub="Aktiv"><NxToggle on={true} pct={1}/></NxSetRow>
        <NxSetRow label="Auto-Tags" sub="KI-Vorschläge"><NxToggle on={true} pct={1}/></NxSetRow>
        <NxSetRow label="Benachrichtigungen" sub="Nur Aufgaben"><NxToggle on={false} pct={0}/></NxSetRow>
      </div>
    </div>
  );
}

function NxSetRow({ label, sub, children }) {
  return (
    <div style={{
      padding: '14px 16px',
      background: NX.bgCard,
      border: `1px solid ${NX.border}`,
      borderRadius: 12,
      display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: 12,
    }}>
      <div>
        <div style={{ fontFamily: NX.font, fontSize: 14, fontWeight: 600, color: NX.text }}>{label}</div>
        <div style={{ fontFamily: NX.font, fontSize: 12, color: NX.textDim, marginTop: 2 }}>{sub}</div>
      </div>
      {children}
    </div>
  );
}

function NxToggle({ on = true, pct = 1 }) {
  // pct 0..1 — animates between off and on positions/colors
  const knobX = pct * 22;
  const bg = `oklch(${0.30 + 0.20 * pct}  ${0.05 + 0.10 * pct}  ${280})`;
  return (
    <div style={{
      width: 50, height: 28,
      borderRadius: 999,
      background: pct > 0.5 ? NX.purple : '#2a2d3a',
      transition: 'background 200ms',
      position: 'relative',
      border: `1px solid ${NX.border}`,
    }}>
      <div style={{
        position: 'absolute', top: 2, left: 2,
        width: 22, height: 22,
        borderRadius: 999,
        background: '#fff',
        transform: `translateX(${knobX}px)`,
        boxShadow: '0 2px 6px rgba(0,0,0,0.5)',
      }}/>
    </div>
  );
}

// ── Icons (simple stroke SVG) ─────────────────────────────────────────────
const ic = { fill: 'none', stroke: 'currentColor', strokeWidth: 1.8, strokeLinecap: 'round', strokeLinejoin: 'round' };
function NxHomeIcon()   { return <svg viewBox="0 0 24 24" {...{}}><path d="M3 11 L12 3 L21 11 V20 H14 V14 H10 V20 H3 Z" {...ic}/></svg>; }
function NxBrainIcon()  { return <svg viewBox="0 0 24 24"><path d="M9 4 a3 3 0 0 0 0 6 a3 3 0 0 0 0 6 a2 2 0 0 0 3 2 a2 2 0 0 0 3 -2 a3 3 0 0 0 0 -6 a3 3 0 0 0 0 -6 a2 2 0 0 0 -3 -1 a2 2 0 0 0 -3 1z M12 5 V19" {...ic}/></svg>; }
function NxCheckIcon()  { return <svg viewBox="0 0 24 24"><rect x="4" y="4" width="16" height="16" rx="3" {...ic}/><path d="M8 12 L11 15 L16 9" {...ic}/></svg>; }
function NxFolderIcon() { return <svg viewBox="0 0 24 24"><path d="M3 6 a2 2 0 0 1 2 -2 H9 L11 6 H19 a2 2 0 0 1 2 2 V18 a2 2 0 0 1 -2 2 H5 a2 2 0 0 1 -2 -2 Z" {...ic}/></svg>; }
function NxMoreIcon()   { return <svg viewBox="0 0 24 24"><circle cx="6" cy="12" r="1.5" fill="currentColor"/><circle cx="12" cy="12" r="1.5" fill="currentColor"/><circle cx="18" cy="12" r="1.5" fill="currentColor"/></svg>; }
function NxMoonIcon()   { return <svg viewBox="0 0 24 24" width="18" height="18"><path d="M20 14 A8 8 0 1 1 10 4 A6 6 0 0 0 20 14 Z" {...ic}/></svg>; }
function NxSunIcon()    { return <svg viewBox="0 0 24 24" width="18" height="18"><circle cx="12" cy="12" r="4" {...ic}/><path d="M12 3 V5 M12 19 V21 M3 12 H5 M19 12 H21 M5.6 5.6 L7 7 M17 17 L18.4 18.4 M5.6 18.4 L7 17 M17 7 L18.4 5.6" {...ic}/></svg>; }
function NxGearIcon()   { return <svg viewBox="0 0 24 24" width="18" height="18"><circle cx="12" cy="12" r="3" {...ic}/><path d="M12 2 V5 M12 19 V22 M2 12 H5 M19 12 H22 M5 5 L7 7 M17 17 L19 19 M5 19 L7 17 M17 7 L19 5" {...ic}/></svg>; }

// ── Phone shell wrapping the canvas ───────────────────────────────────────
function NxScreen({ children, style = {} }) {
  return (
    <div style={{
      position: 'absolute', inset: 0,
      background: NX.bg,
      color: NX.text,
      fontFamily: NX.font,
      overflow: 'hidden',
      ...style,
    }}>
      {children}
    </div>
  );
}

// Status bar (faux iOS)
function NxStatusBar() {
  return (
    <div style={{
      display: 'flex', justifyContent: 'space-between', alignItems: 'center',
      padding: '14px 28px 0',
      fontFamily: NX.font,
      fontSize: 13,
      fontWeight: 600,
      color: NX.text,
    }}>
      <div>21:07</div>
      <div style={{ display: 'flex', gap: 6, alignItems: 'center' }}>
        <svg width="16" height="11" viewBox="0 0 16 11"><path d="M0 11 L3 11 L3 8 L0 8 Z M4 11 L7 11 L7 6 L4 6 Z M8 11 L11 11 L11 4 L8 4 Z M12 11 L15 11 L15 2 L12 2 Z" fill={NX.text}/></svg>
        <svg width="16" height="11" viewBox="0 0 16 11"><path d="M8 8 a2 2 0 1 1 0 4 a2 2 0 0 1 0 -4 M4 5 a6 6 0 0 1 8 0 M1 2 a10 10 0 0 1 14 0" stroke={NX.text} strokeWidth="1.5" fill="none"/></svg>
        <div style={{
          width: 24, height: 11, border: `1px solid ${NX.text}`, borderRadius: 3,
          padding: 1,
        }}>
          <div style={{ width: '75%', height: '100%', background: NX.text, borderRadius: 1 }}/>
        </div>
      </div>
    </div>
  );
}

// Inject global keyframes once
if (typeof document !== 'undefined' && !document.getElementById('nx-keyframes')) {
  const s = document.createElement('style');
  s.id = 'nx-keyframes';
  s.textContent = `
    @keyframes nx-spin { from { transform: rotate(0); } to { transform: rotate(360deg); } }
    @keyframes nx-tap  { 0% { transform: scale(0.5); opacity: 1; } 100% { transform: scale(2.4); opacity: 0; } }
  `;
  document.head.appendChild(s);
}

Object.assign(window, {
  NX,
  NxTopBar, NxFilterPills, NxSearch, NxAddButton, NxCard, NxTag,
  NxTabBar, NxDetailSheet, NxCameraSheet, NxCursor,
  NxDashboard, NxTasksView, NxProjectsView, NxSettingsView,
  NxToggle, NxScreen, NxStatusBar,
});
