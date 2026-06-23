/**
 * TraceDemo — an in-browser img2bez tracing mini-app (React island).
 *
 * A single self-contained rectangle, like a tiny font editor: the source
 * raster sits behind the work area as a dimmed background template (the way
 * you trace over an image in Runebender), a Trace button runs img2bez
 * (compiled to WASM) on it, and the resulting outline is drawn on top with its
 * on-curve (green) / off-curve (purple) points. Drop your own image onto the
 * app to trace it instead. Scroll to zoom, drag to pan.
 *
 * Reuses only the trace core (one WASM call); the viewer is custom so it fits
 * the blog column and needs no WebGPU.
 */
import { useCallback, useEffect, useRef, useState } from "react";
import init, { traceToGlif } from "../lib/img2bez-wasm/img2bez_wasm.js";
import wasmUrl from "../lib/img2bez-wasm/img2bez_wasm_bg.wasm?url";

// autoresearch / Runebender-web palette
const BG = "#0c0c0c";
const GREEN = "#66ee88"; // on-curve
const PURPLE = "#8b6cff"; // off-curve
const HANDLE = "#7a7a7a";
const OUTLINE = "#e6e6e6";

let wasmReady: Promise<unknown> | null = null;
function ensureWasm() {
  if (!wasmReady) wasmReady = init({ module_or_path: wasmUrl });
  return wasmReady;
}

type Pt = { x: number; y: number; on: boolean };
type Box = { minX: number; minY: number; maxX: number; maxY: number };

function parseGlif(glif: string): Pt[][] {
  const doc = new DOMParser().parseFromString(glif, "application/xml");
  const contours: Pt[][] = [];
  doc.querySelectorAll("contour").forEach((c) => {
    const pts: Pt[] = [];
    c.querySelectorAll("point").forEach((p) => {
      pts.push({
        x: parseFloat(p.getAttribute("x") || "0"),
        y: parseFloat(p.getAttribute("y") || "0"),
        on: p.getAttribute("type") !== null,
      });
    });
    if (pts.length) contours.push(pts);
  });
  return contours;
}

// Bounding box of the glyph within a raster (pixels that differ from the
// corner background colour), so the traced outline can be aligned to it.
function glyphBoxOf(img: HTMLImageElement): Box {
  const scale = Math.min(1, 320 / Math.max(img.width, img.height));
  const w = Math.max(1, Math.round(img.width * scale));
  const h = Math.max(1, Math.round(img.height * scale));
  const c = document.createElement("canvas");
  c.width = w;
  c.height = h;
  const cx = c.getContext("2d", { willReadFrequently: true })!;
  cx.drawImage(img, 0, 0, w, h);
  const d = cx.getImageData(0, 0, w, h).data;
  const luma = (x: number, y: number) => {
    const i = (y * w + x) * 4;
    return (d[i] + d[i + 1] + d[i + 2]) / 3;
  };
  const bg = (luma(0, 0) + luma(w - 1, 0) + luma(0, h - 1) + luma(w - 1, h - 1)) / 4;
  let minX = w, minY = h, maxX = 0, maxY = 0, found = false;
  for (let y = 0; y < h; y++)
    for (let x = 0; x < w; x++) {
      if (Math.abs(luma(x, y) - bg) > 40) {
        found = true;
        if (x < minX) minX = x;
        if (x > maxX) maxX = x;
        if (y < minY) minY = y;
        if (y > maxY) maxY = y;
      }
    }
  if (!found) return { minX: 0, minY: 0, maxX: img.width, maxY: img.height };
  return { minX: minX / scale, minY: minY / scale, maxX: (maxX + 1) / scale, maxY: (maxY + 1) / scale };
}

type Props = { image?: string; glyph?: string; unicode?: string };

export default function TraceDemo({ image = "/demos/img2bez/a.png", glyph = "a", unicode = "0061" }: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const imgRef = useRef<HTMLImageElement | null>(null);
  const fileRef = useRef<HTMLInputElement>(null);
  const [src, setSrc] = useState(image);
  const [box, setBox] = useState<Box | null>(null);
  const [contours, setContours] = useState<Pt[][] | null>(null);
  const [busy, setBusy] = useState(false);
  const [zoom, setZoom] = useState(1);
  const [pan, setPan] = useState({ x: 0, y: 0 });
  const [showImage, setShowImage] = useState(true);
  const [dropping, setDropping] = useState(false);
  const drag = useRef<{ x: number; y: number } | null>(null);

  // Load the current image and measure its glyph box.
  useEffect(() => {
    let revoke: string | null = src.startsWith("blob:") ? src : null;
    const img = new Image();
    img.crossOrigin = "anonymous";
    img.onload = () => {
      imgRef.current = img;
      setBox(glyphBoxOf(img));
      setContours(null);
      setShowImage(true);
      setZoom(1);
      setPan({ x: 0, y: 0 });
    };
    img.src = src;
    return () => {
      if (revoke) URL.revokeObjectURL(revoke);
    };
  }, [src]);

  const trace = useCallback(async () => {
    setBusy(true);
    try {
      await ensureWasm();
      const bytes = new Uint8Array(await (await fetch(src)).arrayBuffer());
      const glif = traceToGlif(bytes, JSON.stringify({ glyph, unicode }));
      setContours(parseGlif(glif));
      setShowImage(false); // focus on the result; toggle back with the button
    } catch {
      /* leave the template visible on failure */
    } finally {
      setBusy(false);
    }
  }, [src, glyph, unicode]);

  const draw = useCallback(() => {
    const canvas = canvasRef.current;
    const img = imgRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    const dpr = window.devicePixelRatio || 1;
    const W = canvas.clientWidth;
    const H = canvas.clientHeight;
    if (canvas.width !== W * dpr || canvas.height !== H * dpr) {
      canvas.width = W * dpr;
      canvas.height = H * dpr;
    }
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.fillStyle = BG;
    ctx.fillRect(0, 0, W, H);
    if (!img || !box) return;

    // View transform: fit the glyph box to ~64% of the rectangle (a little
    // breathing room), anchored on its centre, with user zoom/pan. Image
    // pixels -> canvas pixels.
    const gW = box.maxX - box.minX || 1;
    const gH = box.maxY - box.minY || 1;
    const base = Math.min((W * 0.64) / gW, (H * 0.64) / gH); // scale at zoom 1
    const s = base * zoom;
    const gcx = (box.minX + box.maxX) / 2;
    const gcy = (box.minY + box.maxY) / 2;
    const IX = (x: number) => (x - gcx) * s + W / 2 + pan.x;
    const IY = (y: number) => (y - gcy) * s + H / 2 + pan.y;
    const invX = (c: number) => (c - W / 2 - pan.x) / s + gcx;
    const invY = (c: number) => (c - H / 2 - pan.y) / s + gcy;

    // Grid: an 8x6 division of the island, anchored on the glyph centre (which
    // sits at the island centre by default). At zoom 1 the major lines land
    // exactly on the four edges, and cells stay square because the island is
    // 4:3. It still pans/zooms with the content like an editor.
    // Draw in device-pixel space, snapped, so the lines are a crisp 1px (the
    // same weight as the border). Lines that land on the canvas edge are
    // skipped — the 1px border is itself the outer grid line, so the grid lines
    // up perfectly with the edge by default.
    const drawGrid = (cell: number, alpha: number) => {
      if (cell * s < 4) return; // hide a level when it would get too dense
      ctx.save();
      ctx.setTransform(1, 0, 0, 1, 0, 0);
      ctx.strokeStyle = `rgba(255,255,255,${alpha})`;
      ctx.lineWidth = dpr; // 1 CSS px
      const off = Math.round(dpr) % 2 ? 0.5 : 0; // crisp centring for odd widths
      ctx.beginPath();
      for (let n = Math.ceil((invX(0) - gcx) / cell); n <= (invX(W) - gcx) / cell; n++) {
        const X = IX(gcx + n * cell);
        if (X < 0.5 || X > W - 0.5) continue; // the border is the edge line
        const dX = Math.round(X * dpr) + off;
        ctx.moveTo(dX, 0);
        ctx.lineTo(dX, H * dpr);
      }
      for (let m = Math.ceil((invY(0) - gcy) / cell); m <= (invY(H) - gcy) / cell; m++) {
        const Y = IY(gcy + m * cell);
        if (Y < 0.5 || Y > H - 0.5) continue;
        const dY = Math.round(Y * dpr) + off;
        ctx.moveTo(0, dY);
        ctx.lineTo(W * dpr, dY);
      }
      ctx.stroke();
      ctx.restore();
    };
    const majorCell = W / 8 / base; // content units; W/8 px at zoom 1
    drawGrid(majorCell / 4, 0.035); // dense sub-grid
    drawGrid(majorCell, 0.09); // major lines (land on the edges by default)

    // Dimmed background template.
    if (showImage) {
      ctx.globalAlpha = 0.4;
      ctx.drawImage(img, IX(0), IY(0), img.width * s, img.height * s);
      ctx.globalAlpha = 1;
    }

    if (!contours) return;

    // Outline bbox (font units) -> glyph box (image px, y flipped) -> canvas.
    let oMinX = Infinity, oMinY = Infinity, oMaxX = -Infinity, oMaxY = -Infinity;
    for (const c of contours)
      for (const p of c) {
        oMinX = Math.min(oMinX, p.x); oMaxX = Math.max(oMaxX, p.x);
        oMinY = Math.min(oMinY, p.y); oMaxY = Math.max(oMaxY, p.y);
      }
    const oW = oMaxX - oMinX || 1, oH = oMaxY - oMinY || 1;
    const FX = (fx: number) => IX(box.minX + ((fx - oMinX) / oW) * gW);
    const FY = (fy: number) => IY(box.maxY - ((fy - oMinY) / oH) * gH);

    const path = new Path2D();
    for (const pts of contours) {
      const n = pts.length;
      const start = pts.findIndex((p) => p.on);
      if (start < 0) continue;
      const seq = Array.from({ length: n }, (_, k) => pts[(start + k) % n]);
      path.moveTo(FX(seq[0].x), FY(seq[0].y));
      let i = 1;
      while (i <= n) {
        const p = seq[i % n];
        if (p.on) {
          path.lineTo(FX(p.x), FY(p.y));
          i += 1;
        } else {
          const c1 = seq[i % n], c2 = seq[(i + 1) % n], e = seq[(i + 2) % n];
          path.bezierCurveTo(FX(c1.x), FY(c1.y), FX(c2.x), FY(c2.y), FX(e.x), FY(e.y));
          i += 3;
        }
      }
      path.closePath();
    }
    ctx.fillStyle = "rgba(210,210,210,0.07)"; // neutral light-gray fill, no colour
    ctx.fill(path, "evenodd");
    ctx.strokeStyle = OUTLINE;
    ctx.lineWidth = 1.6;
    ctx.stroke(path);

    // handle lines
    ctx.strokeStyle = HANDLE;
    ctx.lineWidth = 1;
    for (const pts of contours) {
      const n = pts.length;
      for (let i = 0; i < n; i++) {
        if (!pts[i].on) continue;
        for (const j of [(i - 1 + n) % n, (i + 1) % n])
          if (!pts[j].on) {
            ctx.beginPath();
            ctx.moveTo(FX(pts[i].x), FY(pts[i].y));
            ctx.lineTo(FX(pts[j].x), FY(pts[j].y));
            ctx.stroke();
          }
      }
    }
    // points
    for (const pts of contours)
      for (const p of pts) {
        ctx.beginPath();
        ctx.arc(FX(p.x), FY(p.y), p.on ? 4 : 3, 0, Math.PI * 2);
        ctx.fillStyle = BG;
        ctx.fill();
        ctx.lineWidth = 1.8;
        ctx.strokeStyle = p.on ? GREEN : PURPLE;
        ctx.stroke();
      }
  }, [box, contours, zoom, pan, showImage]);

  useEffect(() => {
    draw();
  }, [draw]);
  useEffect(() => {
    const onResize = () => draw();
    window.addEventListener("resize", onResize);
    return () => window.removeEventListener("resize", onResize);
  }, [draw]);
  // Native non-passive wheel listener so we can preventDefault and stop the
  // page from scrolling while zooming over the app (React's onWheel is passive).
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const onWheel = (e: WheelEvent) => {
      e.preventDefault();
      setZoom((z) => Math.min(8, Math.max(0.4, z * (e.deltaY < 0 ? 1.1 : 0.9))));
    };
    canvas.addEventListener("wheel", onWheel, { passive: false });
    return () => canvas.removeEventListener("wheel", onWheel);
  }, []);

  const zoomBy = (f: number) => setZoom((z) => Math.min(8, Math.max(0.4, z * f)));
  const resetView = () => {
    setZoom(1);
    setPan({ x: 0, y: 0 });
  };

  const loadFile = (f?: File | null) => {
    if (f && f.type.startsWith("image/")) setSrc(URL.createObjectURL(f));
  };

  // Shared button style so every control is the same height (border-box keeps
  // the border from changing it). Individual buttons override colour/position.
  const baseBtn: React.CSSProperties = {
    position: "absolute",
    font: "inherit",
    fontSize: "0.78em",
    height: 30,
    boxSizing: "border-box",
    display: "inline-flex",
    alignItems: "center",
    justifyContent: "center",
    padding: "0 0.85em",
    borderRadius: 6,
    border: "1px solid #2a2a2a",
    background: "#1b1b1b",
    color: "#cfcfcf",
    cursor: "pointer",
    lineHeight: 1,
  };

  return (
    <div
      style={{
        position: "relative",
        width: "100%",
        aspectRatio: "4 / 3",
        margin: "1.5rem 0",
        // Match the code snippets' border exactly (Expressive Code: 1px solid
        // var(--border), radius calc(--ec-brdRad + --ec-brdWd)).
        borderRadius: "calc(0.3rem + 1px)",
        overflow: "hidden",
        border: `1px solid ${dropping ? GREEN : "var(--border)"}`,
        background: BG,
      }}
      onDragOver={(e) => {
        e.preventDefault();
        setDropping(true);
      }}
      onDragLeave={() => setDropping(false)}
      onDrop={(e) => {
        e.preventDefault();
        setDropping(false);
        loadFile(e.dataTransfer.files?.[0]);
      }}
    >
      <canvas
        ref={canvasRef}
        onPointerDown={(e) => {
          drag.current = { x: e.clientX - pan.x, y: e.clientY - pan.y };
          (e.target as Element).setPointerCapture(e.pointerId);
        }}
        onPointerMove={(e) => {
          if (drag.current) setPan({ x: e.clientX - drag.current.x, y: e.clientY - drag.current.y });
        }}
        onPointerUp={() => (drag.current = null)}
        style={{ display: "block", width: "100%", height: "100%", cursor: "grab", touchAction: "none" }}
      />

      {/* Trace button, inside the app */}
      <button
        onClick={trace}
        disabled={busy}
        style={{
          ...baseBtn,
          top: 12,
          left: 12,
          fontWeight: 500,
          border: "1px solid #2f7d4f",
          background: busy ? "#1b3b29" : "#18bf73",
          color: busy ? "#9fd9bb" : "#062b18",
          cursor: busy ? "default" : "pointer",
        }}
      >
        {busy ? "Tracing…" : "Trace"}
      </button>

      {/* Toggle the background template, once there is a trace to compare against */}
      {contours && (
        <button onClick={() => setShowImage((v) => !v)} style={{ ...baseBtn, top: 12, right: 12 }}>
          {showImage ? "hide image" : "show image"}
        </button>
      )}

      {/* Zoom controls, inside the app */}
      {(() => {
        const sq: React.CSSProperties = { ...baseBtn, position: "static", width: 30, padding: 0, fontSize: "1em" };
        return (
          <div style={{ position: "absolute", bottom: 12, left: 12, display: "flex", gap: 6 }}>
            <button style={sq} onClick={() => zoomBy(1.25)} aria-label="zoom in">+</button>
            <button style={sq} onClick={() => zoomBy(0.8)} aria-label="zoom out">−</button>
            <button style={{ ...baseBtn, position: "static" }} onClick={resetView}>reset</button>
          </div>
        );
      })()}

      {/* Replace-image affordance */}
      <input
        ref={fileRef}
        type="file"
        accept="image/*"
        style={{ display: "none" }}
        onChange={(e) => loadFile(e.target.files?.[0])}
      />
      <button
        onClick={() => fileRef.current?.click()}
        style={{ ...baseBtn, bottom: 12, right: 12, color: "#9a9a9a" }}
      >
        drop or pick your own image
      </button>
    </div>
  );
}
