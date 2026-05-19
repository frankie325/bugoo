# Local System TTS API Design

## Summary

Implement the first slice of Bugoo's local text-to-speech feature by wiring a Tauri API from the frontend to the operating system's built-in speech command. This version only exposes a callable API; it does not add playback buttons, settings UI, auto-speak behavior, audio caching, or cloud TTS support.

## Goals

- Provide a frontend API `speakText(text, lang?)`.
- Implement the existing Tauri command `speak_text`.
- Use local system TTS without API keys or network access.
- Return quickly after starting speech playback.
- Keep the design small enough to extend later with UI controls or cloud engines.

## Non-Goals

- No new UI in the home page, word cards, list rows, or detail panel.
- No use of the existing `autoSpeak` setting yet.
- No voice picker, volume, rate, or pitch controls.
- No generated audio files and no `audio_url` updates.
- No cloud TTS API integration.

## Architecture

The frontend will call a new API helper:

```ts
speakText(text: string, lang?: string): Promise<void>
```

The helper invokes the existing Tauri command name:

```ts
invoke("speak_text", { text, lang })
```

The Tauri command in `src-tauri/src/commands/tts.rs` will stay thin. It validates the command boundary and delegates to `crate::tts::speak_text`.

The platform implementation in `src-tauri/src/tts/mod.rs` will select a system command:

- macOS: `say`
- Windows: `powershell` with `System.Speech.Synthesis.SpeechSynthesizer`
- Linux: `spd-say`

Speech starts with `spawn` rather than waiting for process completion. This keeps the app responsive and lets future UI decide its own pending state.

## Data Flow

1. A future caller passes text and optional language to `speakText`.
2. `src/lib/api/tts.ts` invokes `speak_text`.
3. `commands::tts::speak_text` forwards the request to the TTS module.
4. `tts::speak_text` trims the text and returns success for empty input.
5. For non-empty input, the module starts the platform TTS command.
6. If the command cannot be started, the error is returned to the frontend.

## Language Handling

The first version accepts `lang?: string` as a lightweight hint. It will not promise exact voice matching across systems.

- macOS may map common Chinese hints such as `zh` and `zh-CN` to a Chinese system voice if available.
- Other platforms may ignore `lang` initially and let the OS choose the default voice.

If a requested voice is not available, the implementation should fall back to the OS default instead of failing.

## Error Handling

- Empty or whitespace-only text returns `Ok(())`.
- Failure to spawn the system TTS command returns a descriptive error string.
- Playback failures after the child process starts are not tracked in this slice.
- Missing Linux `spd-say` returns an error that can be surfaced by future UI.

## Files

- `src-tauri/src/tts/mod.rs`: implement platform-specific local TTS start logic.
- `src-tauri/src/commands/tts.rs`: delegate the Tauri command to the TTS module.
- `src/lib/api/tts.ts`: add the frontend invoke helper.
- `src/lib/api/index.ts`: re-export the helper.

## Testing

- Run Rust checks for the Tauri backend.
- Run the frontend TypeScript and Vite build.
- Manually test through a direct invoke or temporary call in development after implementation, without committing any temporary UI.

## Future Extensions

- Add a speaker button to `DetailPanel`.
- Add optional card/list row playback controls.
- Wire `autoSpeak` into translation or review flows.
- Add voice, rate, and cloud TTS settings.
- Store generated audio in `audio_url` only if a later design chooses caching.
