# Assets

This folder holds everything the app draws or plays at runtime. The tool and
button icons that ship with the repo live alongside user-supplied canvas
content. Drop your own files into the folders below; the app reads them at
startup and gracefully no-ops when a folder is empty.

## Shipped with the repo (covered by the project's MIT license)

- `buttons/` - icons for save, load, trash, undo, redo.
- `tools/` - icons for each drawing tool.
- `music_buttons/` - icons for play, pause, next, previous.

## User-supplied (add your own; respect copyright)

- `backgrounds/` - full-screen background images (PNG/JPG). Create this folder
  and add images; the forward/back preview buttons cycle through them. When
  empty, the app paints a solid backdrop.
- `stamps/` - images you can stamp onto the canvas (PNG with transparency works
  best). When empty, the stamp previews show a placeholder.
- `music/` - audio tracks (MP3/OGG/WAV). When empty (or on a machine with no
  audio device), the player shows a placeholder and the rest of the app keeps
  working.

## Optional

- `title.png` (place at `assets/title.png`, next to this folder's siblings) -
  image drawn in the header. If absent, a text title is drawn instead.

Only commit media you have the rights to distribute. Do not commit copyrighted
music, characters, or artwork to a public fork of this project.
