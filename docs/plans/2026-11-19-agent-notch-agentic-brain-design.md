# Agentic Brain design (app: agent-notch)

## Problem

agent-notch is today a passive Dynamic Notch: media, notifications, stats, timers. The
user wants it to be a "Jarvis": it should be *aware of everything on screen*
(public or private), be able to *control the desktop and the browser*, with a
stack of *providers* (local + online with automatic failover), and later speak
to and listen to the user (voice is out of scope for this run).

## Solution

Add an **agent brain** inside the agent-notch app:

- **Chat surface**: new `SlideKind::Agent` — a chat panel on the notch.
- **Brain**: new `src/agent/` module (providers, conversation, desktop tools,
  vision, guard, storage).
- **Providers**: OpenAI-compatible chain (Ollama local + OpenRouter + Groq +
  OpenAI + any OpenAI-compatible server) with per-provider health tracking and
  automatic failover.
- **Decision model**: flagged provider that decides the next action when the
  user speaks (answer vs tool call vs planning).
- **Desktop tools** (via windows-rs): screenshot, click, drag, type, keys,
  scroll, activate/focus window, list windows, clipboard, launch app, shell.
- **Screen awareness**: foreground window + title, clipboard, screenshots
  (GDI BitBlt + `image` crate → base64 PNG when the model has vision support).
- **Approval**: default ask for mutations; autonomous mode opt-in; irreversible
  actions never auto-run.

## Key decisions

1. **Agent brain inside agent-notch**, not a separate Jarvis app (user asked).
2. **HTTP provider clients**: `ureq` (blocking, minimal) — no async runtime.
   Chat loop runs in a background thread; egui polls with
   `request_repaint_later`.
3. **Provider chain + health**: `ChatProvider` trait; `ProviderChain` holds
   `Vec<ProviderEntry{config, health}>`; errors classified
   (rate_limit/quota/overloaded/5xx/connection → failover; 401/403 → provider
   dead for session). Cooldown per provider, backoff, restart on success.
4. **Decision model**: `AgentConfig.decision_provider` flag (one provider in
   the chain). Decision response is strict JSON:
   `{"type":"answer","text":...}` or `{"type":"tool","tool":...,"args":...}`.
5. **Screen awareness**: `ScreenContext { foreground_title, foreground_app,
   clipboard, screenshots }`; screens captured via GDI `GetDC(NULL)` +
   `BitBlt` → `image::save` → base64 PNG when provider `vision=true`.
6. **Desktop tools**: `desktop.rs` uses windows-rs (already a dep). Mouse via
   `SetCursorPos`+`mouse_event`; typing via `SendInput`; keys via `keybd_event`;
   windows via `EnumWindows`/`SetForegroundWindow`; launch via `ShellExecuteW`.
7. **Approvals**: `AgentApprovalMode { Ask (default), Autonomous }`. Read-only
   tools auto-allow. In Ask mode mutations queue for the UI; irreversible
   (delete/payments/credentials) never auto-run.
8. **Storage**: `dirs::config_dir()/agent-notch/agent_history.json` — serde_json
   history (OpenAI-style messages) + session list.
9. **Voice (stage 4)**: config section, speech engine traits, and
   "Piper-style"/whisper.cpp server adapters — decided in docs.

## Module map

| File | Responsibility |
|------|----------------|
| `src/agent/mod.rs` | Module root, state, agent loop entry |
| `src/agent/providers.rs` | Provider trait, chain, health, failover, HTTP clients (Ollama/OpenAI-compat) |
| `src/agent/message.rs` | Message types (text/image), tool-call parsing, decision response parsing |
| `src/agent/agent.rs` | AgentEngine: turn loop, context gathering, tool execution, turn limits |
| `src/agent/guard.rs` | Approval policy: read-only allowlist, irreversible blocks, UI queue |
| `src/agent/desktop.rs` | Windows desktop tools (mouse/keys/windows/clipboard/launch/shell) |
| `src/agent/vision.rs` | GDI screen capture, foreground context, clipboard, window listing |
| `src/agent/storage.rs` | History/session JSON persistence |
| `src/notch/agent_panel.rs` | Chat panel for the new slide (new file) |
| `src/gui/agent_settings.rs` | Settings page section (providers, keys, voice hooks) (new file) |
| `src/config.rs` | `SlideKind::Agent`, `AppConfig.agent`, name/layout wiring |
| `src/notch/layout.rs` | `segment_width` + dispatcher for new slide |
| `Cargo.toml` | Add `ureq`, `base64`, `chrono` |

## Stages (fable stage map)

- **Stage 0** — design + plan (this run). *Artifact*: this doc + plan file.
- **Stage 1** — agent core: providers, message types, agent loop, guard,
  storage, desktop tools, vision, config wiring, chat panel. *Verify:*
  `cargo check --target x86_64-pc-windows-msvc` + unit tests.
- **Stage 2** — settings page + integration into existing screens + README +
  smoke path documented. *Verify:* check + format; manual smoke on Windows.
- **Stage 3** — voice stage design + docs for the future: `src/agent/voice.rs`
  plan section, local TTS/STT adapter plan (see voice sketch below).

### Voice sketch (stage 3, not delivered this run)

`trait VoiceEngine { fn speak(&self, text: &str); fn transcribe(&self, audio: &[u8]) -> Option<String>; }`
Config: `AgentConfig.voice { enabled, stt_url, tts_url, wake_word? }`.
Adapters: `WhisperCppServer` (STT, POST /inference), `PiperTtsServer` or
OpenAI-compatible TTS endpoint; mic toggle on the Agent slide.

## Failable checks per stage

- Stage 1: new Rust modules type-check on the Windows target; unit tests
  (`cargo test` equivalent) exist and are listed in the plan; manual smoke:
  set up Ollama locally, add OpenRouter key via env, ask the agent one
  simple desktop action, watch it execute or ask for approval.
- Stage 2: settings page shows the Agent section; toggles update config;
  existing screens unchanged (run full existing test suite).

## Scope guardrails

- YAGNI: no voice this run; no OCR (accessibility of focused window only);
  no proactive/autonomous behavior without the user asking first
  (proactive mode = explicit setting).
- Each stage: turn loop max 8 rounds; history cap 200 messages; no
  irreversible action without approval.
