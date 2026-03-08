# Topic: Personal

This is Schalk's personal channel. It's for personal matters — not development work.

Tone: casual, friendly, helpful. Match Schalk's energy.

This is NOT a dev topic. Don't default to code mode unless explicitly asked.

# Dev: Building & Restarting

- Always use the **debug** build (`cargo build`), NOT release. The `yolo-restart.sh` script runs the debug binary.
- To restart after code changes: `cargo build && ./yolo-restart.sh 8080`
- Do NOT use `cargo build --release` — the release build is only for production deploys.
- Port is **8080**.
- **ALL tests must pass.** "Pre-existing failure" or "unrelated" is NOT an excuse to ignore a failing test. If a test fails, investigate and fix it before moving on. If it's genuinely broken by something outside your current work, flag it explicitly to the user and get confirmation — don't silently dismiss it.

## Frontend-Only Builds (No Backend Restart Needed)

The Elm frontend is served from `data/frontend/dist/` at runtime via `ServeDir` — it is NOT embedded in the Rust binary. This means frontend changes take effect immediately on browser refresh, with no backend rebuild or restart.

- **Quick build**: `./elm-build.sh` — compiles Elm and copies all frontend assets (elm.js, index.html, elm-pkg-js) to `data/frontend/dist/`. This is all you need for Elm-only changes.
- **Watch mode**: `./elm-watch.sh` — auto-rebuilds on `.elm` file changes (requires `inotify-tools`).
- **Full build**: `./compile.sh` — builds Rust + Elm + copies all frontend assets (elm.js, index.html, elm-pkg-js). Use this when both backend and frontend changed.

**Rule: Do NOT rebuild or restart the backend for frontend-only changes.** Use `elm-build.sh` (or `elm-watch.sh`) and refresh the browser.

# Dev: elm-pkg-js (JS interop for Elm)

All JavaScript interop with the Elm frontend uses the **elm-pkg-js** pattern from [supermario/elm-pkg-js](https://github.com/supermario/elm-pkg-js). This is the standard for shipping JS alongside Elm packages, created by the Lamdera author.

## Structure

- JS modules live in `frontend/elm-pkg-js/*.js`
- Each module registers on `window.elmPkgJs` and exposes an `init(app)` function
- `index.html` loads each module via `<script>` tags, then inits them all with try-catch isolation
- `compile.sh` copies `elm-pkg-js/` to both `data/frontend/dist/` and the XDG data dir

## How to add a new JS module

1. Create `frontend/elm-pkg-js/my-feature.js`:
   ```javascript
   /* elm-pkg-js
   port myPortOut : String -> Cmd msg
   port myPortIn : (String -> msg) -> Sub msg
   */
   window.elmPkgJs = window.elmPkgJs || {};
   window.elmPkgJs['my-feature'] = {
       init: function(app) {
           if (!app.ports || !app.ports.myPortOut) {
               console.warn('elm-pkg-js [my-feature]: required ports not found');
               return;
           }
           app.ports.myPortOut.subscribe(function(value) {
               // ... do JS stuff ...
               if (app.ports.myPortIn) {
                   app.ports.myPortIn.send(result);
               }
           });
       }
   };
   ```
2. Add matching `port` declarations in `Main.elm`
3. Add `<script src="/elm-pkg-js/my-feature.js"></script>` to `index.html` (before the init script)
4. Build: `cd frontend && lamdera make src/Main.elm --output=dist/elm.js`

## Rules

- **Always check ports defensively**: `if (!app.ports.X) return;` — prevents cascade failures
- **Each module is isolated**: one failing module must not crash others (try-catch in init loop)
- **Comment block at top**: include `/* elm-pkg-js ... */` listing the ports the module uses
- **No inline JS in index.html**: all port logic goes in elm-pkg-js modules
- Lamdera docs: https://dashboard.lamdera.app/docs/elm-pkg-js

## Current modules

| Module | Purpose |
|--------|---------|
| `voice-recording.js` | Audio recording via MediaRecorder API |
| `video-recording.js` | Video+audio recording via MediaRecorder API |
| `file-attachment.js` | File picker (creates hidden input, handles change event) |
| `chat-sse.js` | Server-Sent Events for chat streaming |
| `ui-helpers.js` | Scroll-to-bottom, Enter/Cmd+Enter keyboard shortcuts |

# Dev: Types-First Development

Always start implementation by defining/updating the **types** first (Rust enums/structs and Elm custom types/type aliases), then let the compiler guide you through the remaining changes. This applies to both new features and refactors. The compiler errors are your task list.
