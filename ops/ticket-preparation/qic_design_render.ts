import { browser, root } from "./qic_browser.ts"
import { mkdirSync, existsSync, readFileSync } from "node:fs"
import { dirname, resolve } from "node:path"
import { pathToFileURL } from "node:url"

const [input, prefix, widthArg = "1440", heightArg = "1100"] = process.argv.slice(2)
if (!input || !prefix) throw new Error("usage: bun qic_design_render.ts <html> <output-prefix> [width] [height]")
const width = Number(widthArg), height = Number(heightArg)
if (!Number.isInteger(width) || !Number.isInteger(height) || width < 320 || width > 2560 || height < 320 || height > 3000) throw new Error("Invalid design viewport")
if (!existsSync(input) || !readFileSync(input, "utf8").includes("Poppins")) throw new Error("Design source must exist and use QicTrader's Poppins typography")
mkdirSync(dirname(resolve(prefix)), { recursive: true })
const b = await browser()
try {
  const page = await b.newPage()
  await page.setViewport({ width, height, deviceScaleFactor: 1 })
  await page.setRequestInterception(true)
  page.on("request", r => {
    const url = new URL(r.url())
    if (["file:", "data:", "blob:"].includes(url.protocol)) void r.continue()
    else void r.abort("blockedbyclient")
  })
  await page.goto(pathToFileURL(resolve(input)).href, { waitUntil: "networkidle0" })
  const font = resolve(root, "frontend/public/fonts/Poppins-Regular.ttf")
  const semibold = resolve(root, "frontend/public/fonts/Poppins-SemiBold.ttf")
  if (!existsSync(font) || !existsSync(semibold)) throw new Error("Canonical local Poppins font files are missing")
  await page.addStyleTag({ content: `@font-face{font-family:Poppins;src:url('${pathToFileURL(font).href}');font-weight:400;font-display:block}@font-face{font-family:Poppins;src:url('${pathToFileURL(semibold).href}');font-weight:600;font-display:block}` })
  await page.evaluate(() => document.fonts.load("16px Poppins"))
  await page.evaluate(() => document.fonts.ready)
  const layout = await page.evaluate(() => ({ family: getComputedStyle(document.body).fontFamily, width: document.documentElement.scrollWidth, viewport: innerWidth }))
  if (!layout.family.includes("Poppins")) throw new Error("Design body must render with Poppins")
  if (layout.width > layout.viewport + 1) throw new Error("Design overflows its viewport horizontally; fix the responsive layout before exporting")
  await page.screenshot({ path: `${prefix}.png`, fullPage: true })
  await page.pdf({ path: `${prefix}.pdf`, width: `${width}px`, height: `${height}px`, printBackground: true })
  console.log(JSON.stringify({ png: `${prefix}.png`, pdf: `${prefix}.pdf`, width, height }))
} finally {
  await b.close()
}
