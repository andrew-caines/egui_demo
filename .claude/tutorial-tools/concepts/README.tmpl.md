# egui concepts

[Back to the tutorial](../chapters/03-hello-world.md)

In-depth pages for the types named in the tutorial's Hello World table. The table gives each
one a line; these pages explain how it works, show the forms you will need, and list the
mistakes people make with it. Each page stands on its own.

| Page | What it covers |
|---|---|
| [`CentralPanel`](central-panel.md) | Why the root `Ui` needs it, its frame, and how it shares the window with other panels |
| [`Ui`](ui.md) | The region widgets are added to: its cursor, asking for space, spacing, width limits, disabling, scoped style and ids, and why some sizes arrive a frame late |
| [`NativeOptions`](native-options.md) | The start-up settings for `run_native`: the window, renderer, vsync, centring, saving between runs, and what to change later with `ViewportCommand` instead |
| [`ViewportBuilder`](viewport-builder.md) | The window itself: size and limits, position, title, a borderless window with its own title bar, transparency, always on top, resizing, the icon, maximized and fullscreen, and changing each with `ViewportCommand` |
| [`run_native`](run-native.md) | Start-up in order: what the creator is given, setting fonts, style and theme, restoring state, returning an error, threads beside the UI, why it blocks and what the app name is for |
| [`eframe::App`](app-trait.md) | The trait your app implements: when `ui` runs, `request_repaint` and `request_repaint_after`, keeping `ui` thin, the clear colour, closing, filtering input, saving and `on_exit` |
| [`Layout`](layout.md) | Where widgets go: rows and columns, right-aligned widgets, a title and buttons on the two sides with `Sides`, wrapping, a fixed-height bar, justified widgets, equal columns, `Grid` forms, spacing, and centring a group whose size is not known yet |
| [`Response`](response.md) | What a widget reports each frame: clicks, double- and right-clicks, hover and tooltips, `changed()`, focus and Enter, drags, its rectangle, making a label or container clickable with a `Sense`, and combining responses |

Every example on these pages is compiled and tested code in [`code/concepts`](../code/concepts/).
To click through them all in one window, run the gallery from `tutorial/code`:

```powershell
cargo run -p concepts
```
