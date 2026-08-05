/// Web Worker: img2bez word refinement off the main thread. Loads
/// its own copy of the wasm engine and font (both hit the browser
/// cache), traces requested words, and posts back SVG outlines for
/// the main thread to install into its word cache.
import init, { NtfFont } from './neuraltype-wasm/neuraltype_wasm'
import wasmUrl from './neuraltype-wasm/neuraltype_wasm_bg.wasm?url'

const fonts = new Map<string, Promise<NtfFont>>()

function ensure(fontUrl: string): Promise<NtfFont> {
  let p = fonts.get(fontUrl)
  if (!p) {
    p = (async () => {
      await init({ module_or_path: wasmUrl })
      const bytes = new Uint8Array(await (await fetch(fontUrl)).arrayBuffer())
      return new NtfFont(bytes)
    })()
    fonts.set(fontUrl, p)
  }
  return p
}

self.onmessage = async (e: MessageEvent<{ fontUrl: string; word: string }>) => {
  const { fontUrl, word } = e.data
  try {
    const font = await ensure(fontUrl)
    const svg = font.trace_word_svg(word)
    ;(self as any).postMessage({ fontUrl, word, svg })
  } catch {
    ;(self as any).postMessage({ fontUrl, word, svg: '' })
  }
}
