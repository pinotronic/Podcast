# Podcast Console

Aplicación de producción en vivo para podcast construida con React, Vite, TypeScript, Tauri y Rust.

## Stack

- Frontend: React 19 + Vite + Zustand
- Desktop shell: Tauri 2
- Audio engine: Rust + CPAL + Hound + Symphonia

## Comandos

Desde [podcast-console](podcast-console):

```bash
npm run build
npm run dev
npm run tauri dev
```

Desde [podcast-console/src-tauri](podcast-console/src-tauri):

```bash
cargo check
cargo test
```

## Estado actual

- El frontend puede correr en navegador con un fallback local sin Tauri.
- La app Tauri levanta el motor de audio, mezcla pads y ruta de micrófono.
- Los dispositivos de entrada y salida se pueden aplicar desde la UI y el engine se recrea.
- Los proyectos preservan la selección de dispositivos.
- Al abrir un proyecto, los assets guardados se rehidratan en memoria para que los pads vuelvan a dispararse.

## Límites actuales

- La validación acústica final del cambio físico de dispositivos depende del hardware real del equipo.
- El comportamiento final de entrada/salida todavía requiere validación con hardware real, especialmente al cambiar entre interfaces USB o Bluetooth.

## Recomendado en VS Code

- Tauri
- rust-analyzer
