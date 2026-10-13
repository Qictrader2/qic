import { createRequire } from "node:module"
import { dirname, resolve, join } from "node:path"
import { existsSync } from "node:fs"
import { fileURLToPath, pathToFileURL } from "node:url"

export const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..")
const requireFrontend = createRequire(join(root, "frontend/package.json"))
export const puppeteer = (await import(pathToFileURL(requireFrontend.resolve("puppeteer")).href)).default

export async function browser() {
  let executablePath = process.env.QIC_BROWSER_PATH
  if (!executablePath && process.platform === "win32") {
    executablePath = [process.env["PROGRAMFILES(X86)"], process.env.PROGRAMFILES]
      .filter(Boolean).map(p => join(p!, "Microsoft/Edge/Application/msedge.exe")).find(existsSync)
    if (!executablePath) throw new Error("Microsoft Edge is required on the MSI; set QIC_BROWSER_PATH to its executable")
  }
  executablePath ??= puppeteer.executablePath()
  if (!existsSync(executablePath)) throw new Error("Configured browser is missing; installation requires owner authorization")
  return puppeteer.launch({ executablePath, headless: true })
}
