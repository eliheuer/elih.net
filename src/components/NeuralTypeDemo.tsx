/**
 * NeuralTypeDemo — the NeuralType (.ntf) generative-font demo (React island).
 *
 * The font file loaded here contains no glyph outlines: it is the weights
 * of a 53,600-parameter MLP. Every shape on the canvas is generated live — each
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
const CORNER = '#ef4444' // structure view: corner points (red)
const CARET = '#ef4444' // text cursor (red)
// Selection cloud (field fonts): the gold band of manuscript
// illumination, hugging the ink instead of boxing it.
const CLOUD_FILL = 'rgba(160,160,160,0.22)'
const NODE_ACTIVE = '#f97316' // the selected strand node (orange)
const RING = '#facc15' // the rotating half-ring (yellow)
const CLOUD_STROKE = '#f97316' // same orange as the active node

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
  /// Field fonts: one 2D point per caret index, on the displacement
  /// chain. The cursor lives on these nodes, not between boxes.
  nodes?: { i: number; x: number; y: number }[]
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

/// A natural cubic spline through the strand nodes: C2 continuous
/// (curvature never jumps), chord-length parameterized so uneven
/// node spacing does not kink the curve. Coincident nodes (ligature
/// interiors) collapse to one spline point but keep their params.
function buildStrand(pts: { x: number; y: number }[]) {
  const keep: number[] = []
  const tOfIndex: number[] = new Array(pts.length).fill(0)
  for (let i = 0; i < pts.length; i++) {
    const last = keep.length ? pts[keep[keep.length - 1]] : null
    if (!last || Math.hypot(pts[i].x - last.x, pts[i].y - last.y) > 0.75) keep.push(i)
    tOfIndex[i] = keep.length - 1
  }
  const n = keep.length
  const xs = keep.map((i) => pts[i].x)
  const ys = keep.map((i) => pts[i].y)
  // chord-length params
  const t: number[] = [0]
  for (let k = 1; k < n; k++) {
    t.push(t[k - 1] + Math.max(1e-6, Math.hypot(xs[k] - xs[k - 1], ys[k] - ys[k - 1])))
  }
  // natural spline second derivatives (Thomas algorithm), per axis
  const second = (v: number[]): number[] => {
    if (n < 3) return new Array(n).fill(0)
    const a = new Array(n).fill(0)
    const b = new Array(n).fill(0)
    const c = new Array(n).fill(0)
    const d = new Array(n).fill(0)
    b[0] = 1
    b[n - 1] = 1
    for (let k = 1; k < n - 1; k++) {
      const h0 = t[k] - t[k - 1]
      const h1 = t[k + 1] - t[k]
      a[k] = h0
      b[k] = 2 * (h0 + h1)
      c[k] = h1
      d[k] = 6 * ((v[k + 1] - v[k]) / h1 - (v[k] - v[k - 1]) / h0)
    }
    for (let k = 1; k < n; k++) {
      const m = a[k] / b[k - 1]
      b[k] -= m * c[k - 1]
      d[k] -= m * d[k - 1]
    }
    const m2 = new Array(n).fill(0)
    m2[n - 1] = d[n - 1] / b[n - 1]
    for (let k = n - 2; k >= 0; k--) m2[k] = (d[k] - c[k] * m2[k + 1]) / b[k]
    return m2
  }
  const mx = second(xs)
  const my = second(ys)
  const evalAxis = (v: number[], m2: number[], k: number, u: number) => {
    const h = t[k + 1] - t[k]
    const A = (t[k + 1] - u) / h
    const B = (u - t[k]) / h
    return (
      A * v[k] +
      B * v[k + 1] +
      (((A * A * A - A) * m2[k] + (B * B * B - B) * m2[k + 1]) * h * h) / 6
    )
  }
  const sample = (u: number) => {
    if (n === 1) return { x: xs[0], y: ys[0] }
    const uu = Math.max(t[0], Math.min(t[n - 1], u))
    let k = 0
    while (k < n - 2 && t[k + 1] < uu) k++
    return { x: evalAxis(xs, mx, k, uu), y: evalAxis(ys, my, k, uu) }
  }
  return { sample, tOf: (i: number) => t[tOfIndex[Math.max(0, Math.min(i, pts.length - 1))]], tEnd: n ? t[n - 1] : 0 }
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

const fontCache = new Map<string, Promise<NtfFont>>()
function ensureFont(fontUrl: string): Promise<NtfFont> {
  let p = fontCache.get(fontUrl)
  if (!p) {
    p = (async () => {
      await init({ module_or_path: wasmUrl })
      const bytes = new Uint8Array(await (await fetch(fontUrl)).arrayBuffer())
      return new NtfFont(bytes)
    })()
    fontCache.set(fontUrl, p)
  }
  return p
}

type Props = { text?: string; font?: string; samples?: string }

export default function NeuralTypeDemo({
  // قلم (qalam, "pen") by default: small enough to read the grid details.
  text: initialText = 'قلم',
  font: fontUrl = '/demos/neuraltype/kufic.ntf',
  samples: samplesProp,
}: Props) {
  const samples = samplesProp ? samplesProp.split(',') : SAMPLES
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const hiddenRef = useRef<HTMLInputElement>(null)
  const fontRef = useRef<NtfFont | null>(null)
  // Latest layout + view transform, for pointer hit-testing.
  const viewRef = useRef<{
    line: Line
    caretXs: number[]
    nodes: { x: number; y: number }[] | null
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
  // Field fonts (nastaliq) have no elongation axis; detected from the
  // engine's shape output.
  const [isField, setIsField] = useState(false)
  const [showStrand, setShowStrand] = useState(false)
  const [hideStrand, setHideStrand] = useState(false)
  const [lockStrand, setLockStrand] = useState(false)
  // Node dragging: editing the displacement chain by hand. Offsets
  // are per caret index, in field px, and clear when the text edits.
  const nodeDrag = useRef<{
    i: number
    startX: number
    startY: number
    baseDx: number
    baseDy: number
  } | null>(null)
  const nodeOffsets = useRef<Map<number, { dx: number; dy: number }>>(new Map())
  // debug handle for headless tests
  if (typeof window !== 'undefined') {
    ;(window as any).__ntf = { viewRef, fontRef, nodeOffsets, nodeDrag, sel }
  }
  // Cache the shaped line per (text, elong, dir): caret-blink frames
  // redraw without re-running the model.
  const lineCache = useRef<{ key: string; line: Line; path2d: Path2D } | null>(null)
  const selCache = useRef<{ key: string; path: Path2D | null } | null>(null)
  const hintCache = useRef<{ key: string; path: Path2D | null } | null>(null)

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
      lineCache.current = { key: cacheKey, line, path2d: new Path2D(line.path) }
      if ((line as any).field && !isField) {
        setIsField(true)
        // Field demos start clean: ink and caret only.
        setShowStructure(false)
        setShowGrid(false)
      }
    }

    // Fit the line — right-aligned for RTL, left-aligned for LTR —
    // snapped to whole device pixels for crisp edges.
    // enough margin that the cursor node and ring never clip at the
    // canvas edge (ring radius 15 + halo, plus edge nodes sitting
    // slightly outside the ink)
    const pad = 44
    let cell = Math.min(
      (W - 2 * pad) / Math.max(line.width, 1),
      (H - 2 * pad) / line.grid_h,
    )
    // Kufic grids clamp to whole device pixels for crisp cells; field
    // lines are 64 px/em, so fractional scales are the normal case.
    cell = (line as any).field
      ? Math.max(0.05, cell)
      : Math.max(2, Math.floor(cell * dpr)) / dpr
    const ox = line.rtl
      ? Math.round((W - pad - line.width * cell) * dpr) / dpr
      : Math.round(pad * dpr) / dpr
    const oy = Math.round(((H - line.grid_h * cell) / 2) * dpr) / dpr
    const caretXs = caretPositions(line, text)
    const isFieldLine = !!(line as any).field
    const nodes = line.nodes ?? null
    viewRef.current = { line, caretXs, nodes, ox, oy, cell }

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

    // Selection, behind the glyphs. Field fonts get the cloud: the
    // union of the selected letters' distance fields, traced at a
    // raised iso level, like the cloud bands around manuscript text.
    const [a, b] = [Math.min(sel.start, sel.end), Math.max(sel.start, sel.end)]
    if (b > a) {
      if (isFieldLine && fontRef.current) {
        const key = `${text}\u0000${a}\u0000${b}`
        if (selCache.current?.key !== key) {
          const svg = (fontRef.current as any).selection_path(text, a, b) as string
          selCache.current = { key, path: svg.trim() ? new Path2D(svg) : null }
        }
        const cloud = selCache.current.path
        if (cloud) {
          ctx.save()
          ctx.translate(ox, oy)
          ctx.scale(cell, cell)
          ctx.fillStyle = CLOUD_FILL
          ctx.fill(cloud)
          // 1px background outline under the gold, so the cloud
          // separates from the ink it crosses
          ctx.strokeStyle = BG
          ctx.lineWidth = 3.5 / cell
          ctx.stroke(cloud)
          ctx.strokeStyle = CLOUD_STROKE
          ctx.lineWidth = 1.5 / cell
          ctx.stroke(cloud)
          ctx.restore()
        }
      } else {
        ctx.fillStyle = SELECTION
        for (const s of line.spans) {
          if (s.i >= a && s.i < b) {
            ctx.fillRect(ox + s.x * cell, oy, s.w * cell, line.grid_h * cell)
          }
        }
      }
    }

    ctx.save()
    ctx.translate(ox, oy)
    ctx.scale(cell, cell)
    const path = lineCache.current!.path2d
    ctx.fillStyle = INK
    ctx.fill(path)
    if (showStructure) {
      // Structure view over the normal fill: blue contours, red corner
      // points — the vectors the model actually produced. One
      // continuous outline per connected group, not per-letter boxes.
      ctx.strokeStyle = OUTLINE
      ctx.lineWidth = 1.0 / cell
      ctx.stroke(path)
    }
    ctx.restore()

    if (showStructure) {
      // Corner points, drawn in device space so they stay a crisp
      // fixed size at any zoom.
      // Point size follows the cell size, so long lines with small
      // cells keep readable letterforms under the markers.
      ctx.fillStyle = CORNER
      const s = Math.max(2.5, Math.min(6, cell * 0.35))
      for (const [px, py] of pathPoints(line.path)) {
        ctx.fillRect(ox + px * cell - s / 2, oy + py * cell - s / 2, s, s)
      }
    }

    // Field caret: the strand and its nodes. The strand is the curve
    // that flows through the string (the displacement chain); the
    // nodes are the caret positions on it. The active node carries a
    // rotating half-ring; neighbor nodes run three steps in each
    // direction and halve in size at every step. All opaque: the
    // cursor is geometry, not a glow.
    if (isFieldLine && nodes && nodes.length && !hideStrand) {
      const focusI = Math.max(0, Math.min(sel.end, nodes.length - 1))
      const P = (i: number) => ({
        x: ox + nodes[i].x * cell,
        y: oy + nodes[i].y * cell,
      })
      const p0 = P(focusI)
      const chars = [...text]
      // a node is a "gap slot" when an edit there touches a word
      // boundary: adjacent to a space or the ends of the line
      const isGap = (i: number) =>
        i === 0 || i >= chars.length || chars[i - 1] === ' ' || chars[i] === ' '
      const drawNode = (x: number, y: number, r: number, hollow: boolean) => {
        ctx.fillStyle = BG
        ctx.beginPath()
        ctx.arc(x, y, r + 1, 0, Math.PI * 2)
        ctx.fill()
        if (hollow) {
          ctx.strokeStyle = CARET
          ctx.lineWidth = 2.2
          ctx.beginPath()
          ctx.arc(x, y, r - 1.1, 0, Math.PI * 2)
          ctx.stroke()
        } else {
          ctx.fillStyle = CARET
          ctx.beginPath()
          ctx.arc(x, y, r, 0, Math.PI * 2)
          ctx.fill()
        }
      }
      // insertion hint: outline the cluster the caret sits after, so
      // an edit's landing place is visible even inside ligatures
      if (a === b && focusI > 0 && focusI <= chars.length && chars[focusI - 1] !== ' ') {
        const hkey = `${text}\u0000${focusI}`
        if (hintCache.current?.key !== hkey) {
          const svg = (fontRef.current as any)?.selection_path?.(text, focusI - 1, focusI) as
            | string
            | undefined
          hintCache.current = { key: hkey, path: svg && svg.trim() ? new Path2D(svg) : null }
        }
        const hint = hintCache.current.path
        if (hint) {
          ctx.save()
          ctx.translate(ox, oy)
          ctx.scale(cell, cell)
          ctx.lineWidth = 4.5 / cell
          ctx.strokeStyle = BG
          ctx.stroke(hint)
          ctx.lineWidth = 2.5 / cell
          ctx.strokeStyle = NODE_ACTIVE
          ctx.stroke(hint)
          ctx.restore()
        }
      }
      ctx.lineCap = 'round'
      const strand = buildStrand(nodes.map((_, i) => P(i)))
      const strokeStrand = (u0: number, u1: number, w: number, color = CARET) => {
        const steps = Math.max(2, Math.ceil(Math.abs(u1 - u0) / 3))
        ctx.beginPath()
        for (let k = 0; k <= steps; k++) {
          const q = strand.sample(u0 + ((u1 - u0) * k) / steps)
          if (k === 0) ctx.moveTo(q.x, q.y)
          else ctx.lineTo(q.x, q.y)
        }
        ctx.strokeStyle = BG
        ctx.lineWidth = w + 2
        ctx.stroke()
        ctx.strokeStyle = color
        ctx.lineWidth = w
        ctx.stroke()
      }
      if (showStrand) {
        // the full strand and every node, end to end
        strokeStrand(0, strand.tEnd, 2)
        for (let i = 0; i < nodes.length; i++) {
          const q = P(i)
          drawNode(q.x, q.y, 5.5, isGap(i))
        }
      }
      ctx.strokeStyle = CARET
      ctx.fillStyle = CARET
      for (const dir of [-1, 1]) {
        for (let step = 1; step <= 3; step++) {
          const i0 = focusI + dir * (step - 1)
          const i1 = focusI + dir * step
          if (i1 < 0 || i1 >= nodes.length || i0 < 0 || i0 >= nodes.length) break
          const q1 = P(i1)
          // strand segment along the shared spline, with a halo; the
          // incoming segment is orange, flowing out of the hinted
          // letter into the active node
          const incoming = dir === -1 && step === 1 && a === b
          strokeStrand(strand.tOf(i0), strand.tOf(i1), 2.5, incoming ? RING : CARET)
          // node, one visible notch smaller per step: 5.5 -> 4.5 -> 3.5;
          // hollow when the slot touches a word boundary
          drawNode(q1.x, q1.y, 9.5 - 1.5 * step, isGap(i1))
        }
      }
      // the active node, orange on a 1px background rim
      ctx.fillStyle = BG
      ctx.beginPath()
      ctx.arc(p0.x, p0.y, 11, 0, Math.PI * 2)
      ctx.fill()
      ctx.fillStyle = NODE_ACTIVE
      ctx.beginPath()
      ctx.arc(p0.x, p0.y, 10, 0, Math.PI * 2)
      ctx.fill()
      // rotating half-ring, yellow over a background halo
      const theta = ((performance.now() % 1600) / 1600) * Math.PI * 2
      ctx.beginPath()
      ctx.arc(p0.x, p0.y, 15, theta, theta + Math.PI)
      ctx.strokeStyle = BG
      ctx.lineWidth = 5.5
      ctx.stroke()
      ctx.strokeStyle = RING
      ctx.lineWidth = 3.5
      ctx.beginPath()
      ctx.arc(p0.x, p0.y, 15, theta, theta + Math.PI)
      ctx.stroke()
    } else if (caretOn && a === b) {
      const x = Math.round(ox + (caretXs[Math.min(a, caretXs.length - 1)] ?? 0) * cell)
      const top = oy
      const bot = oy + line.grid_h * cell
      const tw = 5 // triangle half-width
      const th = 7 // triangle height
      ctx.fillStyle = CARET
      ctx.fillRect(x - 1, top, 2, bot - top)
      ctx.beginPath()
      ctx.moveTo(x - tw, top)
      ctx.lineTo(x + tw, top)
      ctx.lineTo(x, top + th)
      ctx.closePath()
      ctx.fill()
      ctx.beginPath()
      ctx.moveTo(x - tw, bot)
      ctx.lineTo(x + tw, bot)
      ctx.lineTo(x, bot - th)
      ctx.closePath()
      ctx.fill()
    }
  }, [text, elong, showGrid, showStructure, showStrand, hideStrand, sel, focused, caretOn])

  // Dragged-node offsets are edits to one layout of one string:
  // they clear when the text changes.
  useEffect(() => {
    if (!nodeOffsets.current.size) return
    nodeOffsets.current.clear()
    ;(fontRef.current as any)?.clear_node_offsets?.()
    lineCache.current = null
    selCache.current = null
  }, [text])

  // Field fonts animate the caret ring continuously.
  useEffect(() => {
    if (!ready || !isField) return
    let raf = 0
    const loop = () => {
      draw()
      raf = requestAnimationFrame(loop)
    }
    raf = requestAnimationFrame(loop)
    return () => cancelAnimationFrame(raf)
  }, [ready, isField, draw])

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
    const gy = (clientY - rect.top - view.oy) / view.cell
    let best = 0
    let bestD = Infinity
    if (view.nodes && view.nodes.length) {
      // 2D nearest chain node: clicks follow the cascade, not a
      // horizontal ruler
      view.nodes.forEach((pt, i) => {
        const d = (gx - pt.x) ** 2 + (gy - pt.y) ** 2
        if (d < bestD) {
          bestD = d
          best = i
        }
      })
      return best
    }
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
      // grabbing the active node starts a chain edit instead of a
      // caret move
      const view = viewRef.current
      const canvas = canvasRef.current
      if (view?.nodes && canvas && !lockStrand) {
        const rect = canvas.getBoundingClientRect()
        const fi = Math.max(0, Math.min(sel.end, view.nodes.length - 1))
        const nx = view.ox + view.nodes[fi].x * view.cell
        const ny = view.oy + view.nodes[fi].y * view.cell
        const dx = e.clientX - rect.left - nx
        const dy = e.clientY - rect.top - ny
        if (dx * dx + dy * dy < 16 * 16) {
          const base = nodeOffsets.current.get(fi) ?? { dx: 0, dy: 0 }
          nodeDrag.current = {
            i: fi,
            startX: e.clientX,
            startY: e.clientY,
            baseDx: base.dx,
            baseDy: base.dy,
          }
          e.currentTarget.setPointerCapture(e.pointerId)
          el?.focus()
          return
        }
      }
      const i = indexAt(e.clientX, e.clientY)
      if (!el || i === null) return
      el.focus()
      el.setSelectionRange(i, i)
      dragAnchor.current = i
      setSel({ start: i, end: i })
      e.currentTarget.setPointerCapture(e.pointerId)
    },
    [indexAt, sel.end, lockStrand],
  )
  const onPointerMove = useCallback(
    (e: React.PointerEvent<HTMLCanvasElement>) => {
      const nd = nodeDrag.current
      if (nd) {
        const view = viewRef.current
        const font = fontRef.current as any
        if (!view || !font?.set_node_offset) return
        const dx = nd.baseDx + (e.clientX - nd.startX) / view.cell
        const dy = nd.baseDy + (e.clientY - nd.startY) / view.cell
        nodeOffsets.current.set(nd.i, { dx, dy })
        font.set_node_offset(nd.i, dx, dy)
        lineCache.current = null
        selCache.current = null
        return
      }
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
    nodeDrag.current = null
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
        // Clipboard, handled explicitly: browsers are inconsistent
        // about default copy/cut/paste on a visually hidden input.
        onCopy={(e) => {
          const el = hiddenRef.current
          if (!el) return
          e.preventDefault()
          const a = el.selectionStart ?? 0
          const b = el.selectionEnd ?? 0
          e.clipboardData.setData('text/plain', el.value.slice(a, b))
        }}
        onCut={(e) => {
          const el = hiddenRef.current
          if (!el) return
          e.preventDefault()
          const a = el.selectionStart ?? 0
          const b = el.selectionEnd ?? 0
          e.clipboardData.setData('text/plain', el.value.slice(a, b))
          el.setRangeText('', a, b, 'start')
          syncFromInput()
        }}
        onPaste={(e) => {
          const el = hiddenRef.current
          if (!el) return
          e.preventDefault()
          const t = e.clipboardData.getData('text/plain').replace(/\s*\n\s*/g, ' ')
          const a = el.selectionStart ?? 0
          const b = el.selectionEnd ?? 0
          el.setRangeText(t, a, b, 'end')
          syncFromInput()
        }}
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
            {samples.map((s) => (
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

        <label style={{ display: isField ? 'none' : 'block' }}>
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

        {isField && (
          <label style={{ display: 'flex', gap: 6, alignItems: 'center', cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={showStrand}
              onChange={(e) => {
                setShowStrand(e.target.checked)
                focusText()
              }}
              style={{ accentColor: INK }}
            />
            show full strand chain
          </label>
        )}

        {isField && (
          <label style={{ display: 'flex', gap: 6, alignItems: 'center', cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={hideStrand}
              onChange={(e) => {
                setHideStrand(e.target.checked)
                focusText()
              }}
              style={{ accentColor: INK }}
            />
            hide strand chain
          </label>
        )}

        {isField && (
          <label style={{ display: 'flex', gap: 6, alignItems: 'center', cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={lockStrand}
              onChange={(e) => {
                setLockStrand(e.target.checked)
                focusText()
              }}
              style={{ accentColor: INK }}
            />
            lock strand chain
          </label>
        )}

        <div style={{ marginTop: 'auto', fontSize: 11.5, color: '#8a8a8a', lineHeight: 1.6 }}>
          {failed ? (
            'failed to load the font model'
          ) : ready ? (
            <>
              {isField && (
                <p style={{ margin: '0 0 8px' }}>
                  click and type: real text. the cursor is a node on
                  the strand, the curve through the text. arrows walk
                  it; drag the orange node to move a letter.
                </p>
              )}
              <p style={{ margin: 0 }}>
                {`the font is a ${stats.params.toLocaleString()}-weight model, ` +
                  `${stats.bytes.toLocaleString()} bytes. no glyph tables; every ` +
                  `letterform is generated per keystroke.`}
              </p>
            </>
          ) : (
            'loading model…'
          )}
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
