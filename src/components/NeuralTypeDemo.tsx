/**
 * NeuralTypeDemo — the NeuralType (.ntf) generative-font demo (React island).
 *
 * The font file loaded here contains no glyph outlines: it is the weights of
 * a ~55k-parameter MLP. Every shape on the canvas is generated live — each
 * keystroke re-runs the model per glyph on (letter, joining form, elongation),
 * the whole line is composited and traced to bezier outlines in Rust/WASM
 * (kurbo), and the path is filled here.
 *
 * The canvas is a real text editor: click to place a cursor, type, drag or
 * shift-arrow to select, copy/paste/undo — all backed by a hidden native
 * input, so the text behaves like text even though every visible shape is
 * model output. The engine reports per-character spans (cluster map) for
 * caret placement, hit testing, and selection highlighting.
 *
 * Visual language follows TraceDemo: a single dark editor rectangle with a
 * left control column, matching the code blocks' border and radius.
 */
import { useCallback, useEffect, useRef, useState } from 'react'
import init, { NtfFont } from '../lib/neuraltype-wasm/neuraltype_wasm.js'
import wasmUrl from '../lib/neuraltype-wasm/neuraltype_wasm_bg.wasm?url'

const BG = '#0c0c0c'
const INK = '#2aa35f' // forest green
const SELECTION = 'rgba(42,163,95,0.30)'
const OUTLINE = '#e5e5e5' // structure view: contour strokes (90% gray)
const CORNER = '#ff7057' // structure view: corner points (light tomato)

// Extract the corner points of a rectilinear SVG path ("M x y L x y … Z"),
// for the structure view. There are no curves yet — v0 outlines are traced
// grid contours, so every point is a corner.
function pathPoints(d: string): [number, number][] {
  const pts: [number, number][] = []
  const toks = d.match(/[A-Za-z]|-?\d*\.?\d+(?:e-?\d+)?/g) ?? []
  let i = 0
  let cmd = ''
  while (i < toks.length) {
    const t = toks[i]
    if (/^[A-Za-z]$/.test(t)) {
      cmd = t.toUpperCase()
      i++
      continue
    }
    if (cmd === 'M' || cmd === 'L') {
      pts.push([parseFloat(t), parseFloat(toks[i + 1])])
      i += 2
    } else {
      i++
    }
  }
  return pts
}

type Line = {
  width: number
  grid_h: number
  baseline: number
  rtl: boolean
  /// One SVG path for the whole line: connected letters are one
  /// continuous contour, traced from the composited glyph grids.
  path: string
  glyphs: { ch: string; form: string; x: number; advance: number }[]
  /// Per-logical-character spans for caret/selection/hit-testing.
  spans: { i: number; x: number; w: number }[]
}

/// Caret x (in line units) for every logical caret index 0..=n.
function caretPositions(line: Line, text: string): number[] {
  const chars = [...text]
  const n = chars.length
  const spans = new Map(line.spans.map((s) => [s.i, s]))
  const charRtl = (ch: string) =>
    /[؀-ۿ]/.test(ch) ? true : /[A-Za-z]/.test(ch) ? false : line.rtl
  const pos: number[] = []
  for (let i = 0; i <= n; i++) {
    if (n === 0) {
      pos.push(line.rtl ? line.width : 0)
    } else if (i === 0) {
      const s = spans.get(0)
      pos.push(s ? (charRtl(chars[0]) ? s.x + s.w : s.x) : line.rtl ? line.width : 0)
    } else {
      const s = spans.get(i - 1)
      pos.push(s ? (charRtl(chars[i - 1]) ? s.x : s.x + s.w) : pos[i - 1])
    }
  }
  return pos
}

const SAMPLES = [
  'قلم', // qalam, "pen" — the default
  'كن فيكون', // kun fayakun, "Be, and it is" — shares the first row
  'بسم الله الرحمن الرحيم', // the basmala
  'أشهد يا إلهي', // opening of the Bahá'í short obligatory prayer
  'الذكاء الاصطناعي', // "artificial intelligence"
  'HELLO WORLD',
  'LOREM IPSUM',
]

let fontReady: Promise<NtfFont> | null = null
function ensureFont(fontUrl: string): Promise<NtfFont> {
  if (!fontReady) {
    fontReady = (async () => {
      await init({ module_or_path: wasmUrl })
      const bytes = new Uint8Array(await (await fetch(fontUrl)).arrayBuffer())
      return new NtfFont(bytes)
    })()
  }
  return fontReady
}

type Props = { text?: string; font?: string }

export default function NeuralTypeDemo({
  // قلم (qalam, "pen") by default: small enough to read the grid details.
  text: initialText = 'قلم',
  font: fontUrl = '/demos/neuraltype/kufic.ntf',
}: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const hiddenRef = useRef<HTMLInputElement>(null)
  const fontRef = useRef<NtfFont | null>(null)
  // Latest layout + view transform, for pointer hit-testing.
  const viewRef = useRef<{
    line: Line
    caretXs: number[]
    ox: number
    oy: number
    cell: number
  } | null>(null)
  const dragAnchor = useRef<number | null>(null)

  const [text, setText] = useState(initialText)
  const [sel, setSel] = useState({ start: initialText.length, end: initialText.length })
  const [focused, setFocused] = useState(false)
  const [caretOn, setCaretOn] = useState(true)
  const [elong, setElong] = useState(0)
  const [showGrid, setShowGrid] = useState(true)
  const [showStructure, setShowStructure] = useState(true)
  const [ready, setReady] = useState(false)
  const [failed, setFailed] = useState(false)
  const [stats, setStats] = useState({ bytes: 0, params: 0 })
  const [fullscreen, setFullscreen] = useState(false)
  const [narrow, setNarrow] = useState(false)
  // Cache the shaped line per (text, elong, dir): caret-blink frames
  // redraw without re-running the model.
  const lineCache = useRef<{ key: string; line: Line } | null>(null)

  // Load engine + font once.
  useEffect(() => {
    let alive = true
    ;(async () => {
      try {
        const font = await ensureFont(fontUrl)
        if (!alive) return
        fontRef.current = font
        const bytes = new Uint8Array(await (await fetch(fontUrl)).arrayBuffer())
        setStats({ bytes: bytes.length, params: font.n_params() })
        setReady(true)
      } catch {
        if (alive) setFailed(true)
      }
    })()
    return () => {
      alive = false
    }
  }, [fontUrl])

  // Narrow (phone) layout: stack controls under the canvas.
  useEffect(() => {
    const mq = window.matchMedia('(max-width: 640px)')
    const update = () => setNarrow(mq.matches)
    update()
    mq.addEventListener('change', update)
    return () => mq.removeEventListener('change', update)
  }, [])

  // Esc exits fullscreen; lock page scroll while expanded.
  useEffect(() => {
    if (!fullscreen) return
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') setFullscreen(false)
    }
    window.addEventListener('keydown', onKey)
    const prev = document.body.style.overflow
    document.body.style.overflow = 'hidden'
    return () => {
      window.removeEventListener('keydown', onKey)
      document.body.style.overflow = prev
    }
  }, [fullscreen])

  // Caret blink — always on, so the demo reads as an editor before
  // anyone clicks. Restarts (visible) on every edit.
  useEffect(() => {
    setCaretOn(true)
    const id = window.setInterval(() => setCaretOn((v) => !v), 530)
    return () => window.clearInterval(id)
  }, [focused, sel.start, sel.end, text])

  // Mirror the hidden input's value + selection into state.
  const syncFromInput = useCallback(() => {
    const el = hiddenRef.current
    if (!el) return
    setText(el.value)
    setSel({ start: el.selectionStart ?? 0, end: el.selectionEnd ?? 0 })
  }, [])
  useEffect(() => {
    const onSelChange = () => {
      if (document.activeElement === hiddenRef.current) syncFromInput()
    }
    document.addEventListener('selectionchange', onSelChange)
    return () => document.removeEventListener('selectionchange', onSelChange)
  }, [syncFromInput])

  // The text cursor owns keyboard focus: grabbed on load (without
  // scrolling the page) and reclaimed after any control interaction,
  // so arrow keys always move the caret — never the slider.
  const focusText = useCallback(() => {
    hiddenRef.current?.focus({ preventScroll: true })
  }, [])

  // Focus the editor as soon as the model is ready.
  useEffect(() => {
    if (ready) focusText()
  }, [ready, focusText])

  const setTextAndFocus = useCallback((t: string) => {
    const el = hiddenRef.current
    if (!el) return
    el.value = t
    el.focus({ preventScroll: true })
    el.setSelectionRange(t.length, t.length)
    setText(t)
    setSel({ start: t.length, end: t.length })
  }, [])

  const draw = useCallback(() => {
    const canvas = canvasRef.current
    const font = fontRef.current
    if (!canvas) return
    const ctx = canvas.getContext('2d')
    if (!ctx) return
    const dpr = window.devicePixelRatio || 1
    const W = canvas.clientWidth
    const H = canvas.clientHeight
    if (canvas.width !== W * dpr || canvas.height !== H * dpr) {
      canvas.width = W * dpr
      canvas.height = H * dpr
    }
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
    ctx.fillStyle = BG
    ctx.fillRect(0, 0, W, H)
    if (!font) return

    // One model run per glyph on every text/parameter change; blink
    // and selection frames reuse the cached layout.
    const cacheKey = `${text}\u0000${elong}`
    let line: Line
    if (lineCache.current?.key === cacheKey) {
      line = lineCache.current.line
    } else {
      line = JSON.parse(font.shape(text, elong, 'auto'))
      lineCache.current = { key: cacheKey, line }
    }

    // Fit the line — right-aligned for RTL, left-aligned for LTR —
    // snapped to whole device pixels for crisp edges.
    const pad = 28
    let cell = Math.min(
      (W - 2 * pad) / Math.max(line.width, 1),
      (H - 2 * pad) / line.grid_h,
    )
    cell = Math.max(2, Math.floor(cell * dpr)) / dpr
    const ox = line.rtl
      ? Math.round((W - pad - line.width * cell) * dpr) / dpr
      : Math.round(pad * dpr) / dpr
    const oy = Math.round(((H - line.grid_h * cell) / 2) * dpr) / dpr
    const caretXs = caretPositions(line, text)
    viewRef.current = { line, caretXs, ox, oy, cell }

    if (showGrid) {
      ctx.strokeStyle = '#4d4d4d'
      ctx.lineWidth = 1
      ctx.beginPath()
      for (let x = 0; x <= Math.ceil(line.width); x++) {
        ctx.moveTo(ox + x * cell, oy)
        ctx.lineTo(ox + x * cell, oy + line.grid_h * cell)
      }
      for (let y = 0; y <= line.grid_h; y++) {
        ctx.moveTo(ox, oy + y * cell)
        ctx.lineTo(ox + line.width * cell, oy + y * cell)
      }
      ctx.stroke()
    }

    // Selection highlight, behind the glyphs.
    const [a, b] = [Math.min(sel.start, sel.end), Math.max(sel.start, sel.end)]
    if (b > a) {
      ctx.fillStyle = SELECTION
      for (const s of line.spans) {
        if (s.i >= a && s.i < b) {
          ctx.fillRect(ox + s.x * cell, oy, s.w * cell, line.grid_h * cell)
        }
      }
    }

    ctx.save()
    ctx.translate(ox, oy)
    ctx.scale(cell, cell)
    const path = new Path2D(line.path)
    ctx.fillStyle = INK
    ctx.fill(path)
    if (showStructure) {
      // Structure view over the normal fill: blue contours, red corner
      // points — the vectors the model actually produced. One
      // continuous outline per connected group, not per-letter boxes.
      ctx.strokeStyle = OUTLINE
      ctx.lineWidth = 1.5 / cell
      ctx.stroke(path)
    }
    ctx.restore()

    if (showStructure) {
      // Corner points, drawn in device space so they stay a crisp
      // fixed size at any zoom.
      ctx.fillStyle = CORNER
      const s = 6
      for (const [px, py] of pathPoints(line.path)) {
        ctx.fillRect(ox + px * cell - s / 2, oy + py * cell - s / 2, s, s)
      }
    }

    // Caret (collapsed selection only) — drawn even before focus, so
    // the canvas visibly invites editing.
    if (caretOn && a === b) {
      const cx = ox + (caretXs[Math.min(a, caretXs.length - 1)] ?? 0) * cell
      ctx.fillStyle = INK
      ctx.fillRect(Math.round(cx) - 1, oy, 2, line.grid_h * cell)
    }
  }, [text, elong, showGrid, showStructure, sel, focused, caretOn])

  useEffect(() => {
    if (ready) draw()
  }, [ready, draw])
  useEffect(() => {
    const onResize = () => draw()
    window.addEventListener('resize', onResize)
    return () => window.removeEventListener('resize', onResize)
  }, [draw])
  useEffect(() => {
    // Redraw after the fullscreen layout change has landed.
    if (ready) requestAnimationFrame(draw)
  }, [fullscreen, ready, draw])

  // Pointer → caret index, via the engine's cluster map.
  const indexAt = useCallback((clientX: number, clientY: number): number | null => {
    const canvas = canvasRef.current
    const view = viewRef.current
    if (!canvas || !view) return null
    const rect = canvas.getBoundingClientRect()
    const gx = (clientX - rect.left - view.ox) / view.cell
    void clientY
    let best = 0
    let bestD = Infinity
    view.caretXs.forEach((x, i) => {
      const d = Math.abs(gx - x)
      if (d < bestD) {
        bestD = d
        best = i
      }
    })
    return best
  }, [])

  const onPointerDown = useCallback(
    (e: React.PointerEvent<HTMLCanvasElement>) => {
      e.preventDefault()
      const el = hiddenRef.current
      const i = indexAt(e.clientX, e.clientY)
      if (!el || i === null) return
      el.focus()
      el.setSelectionRange(i, i)
      dragAnchor.current = i
      setSel({ start: i, end: i })
      e.currentTarget.setPointerCapture(e.pointerId)
    },
    [indexAt],
  )
  const onPointerMove = useCallback(
    (e: React.PointerEvent<HTMLCanvasElement>) => {
      const anchor = dragAnchor.current
      if (anchor === null) return
      const i = indexAt(e.clientX, e.clientY)
      const el = hiddenRef.current
      if (i === null || !el) return
      const [a, b] = [Math.min(anchor, i), Math.max(anchor, i)]
      el.setSelectionRange(a, b, i < anchor ? 'backward' : 'forward')
      setSel({ start: a, end: b })
    },
    [indexAt],
  )
  const onPointerUp = useCallback(() => {
    dragAnchor.current = null
  }, [])

  const groupLabel: React.CSSProperties = {
    fontSize: 11.5,
    color: '#7a7a7a',
    marginBottom: 5,
  }
  const chip: React.CSSProperties = {
    borderRadius: 6,
    border: '1px solid #2a2a2a',
    background: '#1b1b1b',
    color: '#c9c9c9',
    cursor: 'pointer',
    padding: '4px 8px',
    fontSize: 13,
    lineHeight: 1.3,
  }
  const cornerBtn: React.CSSProperties = {
    width: 30,
    height: 30,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    borderRadius: 6,
    border: '1px solid #2a2a2a',
    background: '#1b1b1bd9',
    color: '#cfcfcf',
    cursor: 'pointer',
    padding: 0,
  }

  return (
    <div
      style={{
        ...(fullscreen
          ? {
              position: 'fixed',
              inset: 0,
              zIndex: 1000,
              width: '100%',
              height: '100%',
              margin: 0,
              borderRadius: 0,
            }
          : {
              position: 'relative',
              width: '100%',
              // Square: room in the control column for future axes
              // (weight, slant, …) without a scrollbar.
              ...(narrow ? {} : { aspectRatio: '3 / 2' }),
              margin: 0,
              borderRadius: 'calc(0.3rem + 1px)',
            }),
        overflow: 'hidden',
        border: '1px solid var(--border)',
        background: BG,
        display: 'flex',
        flexDirection: narrow ? 'column-reverse' : 'row',
      }}
    >
      {/* Minimal slider: one color, a thin line and a circle. */}
      <style>{`
        .ntf-slider { -webkit-appearance: none; appearance: none; width: 100%;
          height: 14px; margin: 0; background: transparent; cursor: pointer; }
        .ntf-slider::-webkit-slider-runnable-track { height: 2px;
          background: ${INK}; border: none; border-radius: 1px; }
        .ntf-slider::-webkit-slider-thumb { -webkit-appearance: none;
          width: 12px; height: 12px; margin-top: -5px; border-radius: 50%;
          background: ${INK}; border: none; box-shadow: none; }
        .ntf-slider::-moz-range-track { height: 2px; background: ${INK};
          border: none; border-radius: 1px; }
        .ntf-slider::-moz-range-thumb { width: 12px; height: 12px;
          border-radius: 50%; background: ${INK}; border: none; }
        .ntf-slider:focus { outline: none; }
      `}</style>

      {/* Hidden native input: the real text buffer. Keyboard input,
          selection, clipboard, and undo are all native; the canvas
          renders its state with model-generated outlines. */}
      <input
        ref={hiddenRef}
        type="text"
        defaultValue={initialText}
        // Mirror the detected base direction (same rule as the engine's
        // auto-detect) so native caret movement is visual: ArrowLeft
        // moves left in RTL text too.
        dir={/[\u0600-\u06FF]/.test(text) ? 'rtl' : 'ltr'}
        aria-label="Demo text editor"
        onInput={syncFromInput}
        onSelect={syncFromInput}
        onKeyUp={syncFromInput}
        onFocus={() => {
          setFocused(true)
          syncFromInput()
        }}
        onBlur={() => setFocused(false)}
        style={{
          position: 'absolute',
          left: 0,
          top: 0,
          width: 1,
          height: 1,
          opacity: 0,
          border: 'none',
          padding: 0,
          pointerEvents: 'none',
        }}
      />

      {/* Expand / return toggle, upper right. */}
      <button
        onClick={() => {
          setFullscreen((f) => !f)
          focusText()
        }}
        title={fullscreen ? 'Return to post (Esc)' : 'Expand to full window'}
        aria-label={fullscreen ? 'Return to post' : 'Expand to full window'}
        style={{
          ...cornerBtn,
          position: 'absolute',
          top: 10,
          right: 10,
          zIndex: 10,
        }}
      >
        {fullscreen ? (
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round">
            <path d="M14 6h-4V2" />
            <path d="M10 6l5-5" />
            <path d="M2 10h4v4" />
            <path d="M6 10l-5 5" />
          </svg>
        ) : (
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round">
            <path d="M10 2h4v4" />
            <path d="M14 2l-5 5" />
            <path d="M6 14H2v-4" />
            <path d="M2 14l5-5" />
          </svg>
        )}
      </button>

      {/* Control column. */}
      <div
        style={{
          width: narrow ? '100%' : 232,
          flex: 'none',
          display: 'flex',
          flexDirection: 'column',
          gap: 10,
          padding: 12,
          fontFamily: 'var(--font-mono)',
          fontSize: 12.5,
          boxSizing: 'border-box',
          borderRight: narrow ? 'none' : '1px solid var(--border)',
          borderTop: narrow ? '1px solid var(--border)' : 'none',
          background: '#141414',
          color: '#c9c9c9',
          overflowY: 'auto',
        }}
      >
        <div>
          <div style={groupLabel}>Text Samples</div>
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: 6 }}>
            {SAMPLES.map((s) => (
              <button
                key={s}
                onClick={() => setTextAndFocus(s)}
                style={{
                  ...chip,
                  ...(s === text ? { borderColor: INK, color: INK } : {}),
                }}
              >
                {s}
              </button>
            ))}
          </div>
        </div>

        <label style={{ display: 'block' }}>
          <div style={{ marginBottom: 4 }}>
            kashida elongation <span style={{ color: INK }}>{elong.toFixed(0)}</span>
          </div>
          <input
            type="range"
            className="ntf-slider"
            min={0}
            max={4}
            step={1}
            value={elong}
            onChange={(e) => setElong(parseFloat(e.target.value))}
            onPointerUp={focusText}
          />
        </label>

        <label style={{ display: 'flex', gap: 6, alignItems: 'center', cursor: 'pointer' }}>
          <input
            type="checkbox"
            checked={showStructure}
            onChange={(e) => {
              setShowStructure(e.target.checked)
              focusText()
            }}
            style={{ accentColor: INK }}
          />
          vector outline
        </label>

        <label style={{ display: 'flex', gap: 6, alignItems: 'center', cursor: 'pointer' }}>
          <input
            type="checkbox"
            checked={showGrid}
            onChange={(e) => {
              setShowGrid(e.target.checked)
              focusText()
            }}
            style={{ accentColor: INK }}
          />
          show grid
        </label>

        <div style={{ marginTop: 'auto', fontSize: 11.5, color: '#8a8a8a', lineHeight: 1.5 }}>
          {failed
            ? 'failed to load the font model'
            : ready
              ? `click the canvas to edit: type, select, copy, paste — it is real text. ` +
                `font file: ${stats.bytes.toLocaleString()} bytes = ${stats.params.toLocaleString()} ` +
                `neural-net weights, no glyph tables. Every outline is generated per keystroke.`
              : 'loading model…'}
        </div>
      </div>

      {/* Canvas: the text editor surface. */}
      <div style={{ flex: 1, minWidth: 0, ...(narrow ? { aspectRatio: '16 / 10' } : {}) }}>
        <canvas
          ref={canvasRef}
          onPointerDown={onPointerDown}
          onPointerMove={onPointerMove}
          onPointerUp={onPointerUp}
          style={{ width: '100%', height: '100%', display: 'block', cursor: 'text', touchAction: 'none' }}
        />
      </div>
    </div>
  )
}
