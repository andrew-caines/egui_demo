# `ViewportBuilder`

[Back to the tutorial](../chapters/03-hello-world.md)

## In one paragraph

`ViewportBuilder` describes a native window: its title, size, size limits, position, icon,
whether it has a border and title bar, whether it is transparent, resizable, maximized,
fullscreen or always on top. You build one with `with_*` methods and put it in the `viewport`
field of [`NativeOptions`](native-options.md). eframe uses it once, to create the window. To
change any of these while the app runs, you send the matching `ViewportCommand` with
`ctx.send_viewport_cmd`. The same type also describes every extra window you open with
`ctx.show_viewport_deferred` or `ctx.show_viewport_immediate`.

## The mental model

Think of `ViewportBuilder` as a list of requests, each one optional. Every field is an
`Option`, and `None` means "I have no opinion, use the default". `ViewportBuilder::default()`
asks for nothing, so the operating system and eframe choose everything. Each `with_*` call
fills in one request. Nothing happens until eframe reads the list and creates the window.

`ViewportCommand` is the same list, sent later, one change at a time. egui calls it
"essentially a way to diff `ViewportBuilder`s", and does exactly that for extra windows: it
compares this frame's builder with last frame's and turns each difference into a command
(`with_title` into `Title`, `with_decorations` into `Decorations`). A few settings have no
command, because the windowing library cannot change them on a live window; for those, egui
recreates the window.

The builder is the window; the rest of `NativeOptions` is everything around it (renderer,
vsync, saving, centring). [`NativeOptions`](native-options.md#where-a-setting-goes) has the
full table.

> **If you come from the web.** A web page does not own its window, so a browser offers only
> a few pieces of this: the features string of `window.open(url, name, "width=460,height=220")`
> for a popup's size, `document.title` for the title, `<link rel="icon">` for the icon and
> `element.requestFullscreen()` for fullscreen. The builder is all of those and more, in one
> place, and `ViewportCommand` is how you change them later.
>
> Where the comparison breaks:
>
> - **There is no window object.** You send a command from inside `ui`, and it is carried
>   out after the frame.
> - **You cannot ask the window for its settings.** egui reports some state back (size,
>   position, maximized, fullscreen, focused) through `ui.input(|i| i.viewport())`, but not
>   the title, the window level or whether it is resizable. Keep those in your app's state.
>
> Below, wherever the browser has a real equivalent, the web way (CSS, HTML or JS) is shown
> first and the egui way after it. Most settings have none.

## How it works

1. **You fill in the builder** and put it in `NativeOptions::viewport`.
2. **eframe adjusts it.** It fills in a missing title from the app name. With persistence on,
   a saved position and size replace yours. It clamps the starting size to the largest
   monitor (turn that off with `with_clamp_size_to_monitor_size(false)`). With
   `centered: true`, it replaces your position with one in the middle of the screen.
3. **egui-winit turns it into window attributes.** Sizes and positions are in points, scaled
   by the zoom factor and the screen's scale. Anything left at `None` gets the default in the
   table below.
4. **The window is created**, and the builder is not read again.
5. **Every frame, `App::ui` runs.** A `ctx.send_viewport_cmd(..)` inside it only queues the
   command.
6. **After the frame, egui-winit carries out the queued commands** on the window, in order.
7. **The operating system answers.** Some commands take effect at once, others (a resize on
   Windows, for example) arrive later as an event. Either way, the new size, position or
   state shows up in `ui.input(|i| i.viewport())` on a later frame.

```text
main            eframe / egui-winit                  operating system
 |  ViewportBuilder
 |  in NativeOptions --> fill in title, apply saved size,
 |                       clamp to monitor, centre
 |                       builder -> window attributes --> create the window
 |                                                         |
 |                 every frame:                            |
 |                 App::ui                                 |
 |                   send_viewport_cmd(cmd)  (queued)      |
 |                 after the frame:                        |
 |                   carry out each command  -----------> change the window
 |                 next frame:                             |
 |                   i.viewport() has the new state <----- size, position, state
```

```mermaid
sequenceDiagram
  participant M as main
  participant E as eframe and egui-winit
  participant A as your App
  participant O as operating system
  M->>E: NativeOptions with viewport (ViewportBuilder)
  E->>E: fill in title, saved size, clamp, centre
  E->>O: create the window from the builder
  loop every frame
    E->>A: App::ui
    A-->>E: send_viewport_cmd (queued)
    E->>O: after the frame, carry out each command
    O-->>E: new size, position, state
    E->>A: next frame, i.viewport() shows it
  end
```

### Settings and their commands

| Setting | Builder method | When left unset | Changed while running with |
|---|---|---|---|
| Title | `with_title` | The app name (filled in by eframe) | `Title(String)` |
| Size of the content area | `with_inner_size` | Chosen by the platform | `InnerSize(Vec2)` |
| Size limits | `with_min_inner_size`, `with_max_inner_size` | The platform's own limits | `MinInnerSize`, `MaxInnerSize` |
| Position | `with_position` | Chosen by the operating system | `OuterPosition(Pos2)` |
| Border and title bar | `with_decorations` | On | `Decorations(bool)` |
| Transparent background | `with_transparent` | Off | `Transparent(bool)` |
| Resizable by the edges | `with_resizable` | Yes | `Resizable(bool)` |
| Title bar buttons | `with_close_button`, `with_minimize_button`, `with_maximize_button` | All enabled | `EnableButtons { close, minimized, maximize }` |
| Always on top | `with_always_on_top`, `with_window_level` | `WindowLevel::Normal` | `WindowLevel(WindowLevel)` |
| Icon | `with_icon` | eframe's own: a white "e" on black | `Icon(Option<Arc<IconData>>)` |
| Maximized | `with_maximized` | No | `Maximized(bool)` |
| Fullscreen | `with_fullscreen` | No | `Fullscreen(bool)` |
| Visible | `with_visible` | Yes | `Visible(bool)` |

Some commands are actions, with no builder method: `Close`, `Minimized(bool)`, `StartDrag`,
`BeginResize`, `Focus`. Some builder methods work on one platform only: `with_taskbar` on
Windows, `with_titlebar_shown` and `with_has_shadow` on macOS. `with_app_id` reaches the
window on Wayland only, but through eframe it also names the save folder on every platform
(see [`NativeOptions`](native-options.md)).

## Standard usage

Most windows need a title, a starting size and a minimum size. Here the example also sets a
maximum size and a starting position.

**The JS way:** a page can set its title at any time, but it can only size and place a
window that its own script opens, and the browser may ignore the request. There is no
minimum or maximum size.

```js
document.title = "Viewport builder: size";
window.open("app.html", "app", "width=460,height=220,left=120,top=120");
```

**The egui way:** the program owns its window, so the size and limits it asks for are the ones
it gets.

```rust
/// Size and place the window
fn main() -> eframe::Result {
    let viewport = egui::ViewportBuilder::default()
        .with_title("Viewport builder: size")
        // The area your `ui` draws in, in points. The title bar and border are extra.
        .with_inner_size([460.0, 220.0])
        // The user can resize the window, but only between these two sizes.
        .with_min_inner_size([340.0, 160.0])
        .with_max_inner_size([800.0, 400.0])
        // The top-left corner of the whole window, title bar included, on the screen.
        .with_position([120.0, 120.0]);
```

Run: `cargo run -p concepts --example viewport_builder_size`

![A window titled "Viewport builder: size" reporting an inner size of 460 by 220 and its top-left corner at 120, 120](../images/concepts/viewport-builder-size.png)

The inner size is the area your `ui` draws in. The outer size adds the title bar and border
(on Windows 11 also an invisible resize border, which is why the outer size is wider than the
visible window). `with_position` places the outer top-left corner, title bar included. All of
them are in points, not physical pixels, so the window looks the same size on a high-DPI
screen. Wayland and Android ignore the position.

## Micro-examples

Every example on this page is a whole program in
[`code/concepts/examples/`](../code/concepts/examples/), because each one changes the window
itself, which the gallery cannot show. Run them from `tutorial/code` with the command under
each.

### Borderless window with its own title bar

**Use this when** the app draws its own title bar: a custom look, or tabs and toolbars in the
title bar as in many editors and browsers.

**The CSS way:** only Chromium has this, in installed web apps that use the window controls
overlay. A CSS property marks what drags the window, and the buttons must opt out again.

```css
.title-bar { app-region: drag; }
.title-bar button { app-region: no-drag; }
```

**The egui way:** turn the decorations off in the builder.

```rust
        viewport: egui::ViewportBuilder::default()
            .with_title(TITLE)
            .with_decorations(false)
            .with_inner_size([440.0, 220.0])
            .with_min_inner_size([320.0, 150.0]),
```

Then draw a title bar. Dragging it hands the move to the operating system with `StartDrag`;
the buttons send `Close` and `Minimized`.

```rust
/// A title bar drawn by the app. With decorations off, this is the only way to move, maximize,
/// minimize or close the window.
fn title_bar(ui: &mut egui::Ui, title: &str) {
    egui::Panel::top("title_bar")
        .exact_size(32.0)
        .show(ui, |ui| {
            // The whole bar is the drag region. It is added first, so the buttons added after it
            // sit on top and get their own clicks.
            let bar = ui.interact(
                ui.max_rect(),
                ui.scope_id().with("drag_region"),
                egui::Sense::click_and_drag(),
            );
            if bar.double_clicked() {
                let maximized = ui.input(|input| input.viewport().maximized.unwrap_or(false));
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized));
            } else if bar.drag_started_by(egui::PointerButton::Primary) {
                // Hand the drag to the operating system, which moves the window.
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }

            ui.horizontal_centered(|ui| {
                // Not selectable, so a press on the title drags the window instead of
                // selecting the text.
                ui.add(egui::Label::new(egui::RichText::new(title).strong()).selectable(false));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Close").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    if ui.button("Minimize").clicked() {
                        ui.ctx()
                            .send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    }
                });
            });
        });
}
```

Run: `cargo run -p concepts --example viewport_builder_borderless`

![A borderless window with a title bar drawn by egui: the title on the left, Minimize and Close buttons on the right](../images/concepts/viewport-builder-borderless.png)

The bar is one `ui.interact` over the whole strip, added before the label and buttons so that
they sit on top and keep their own clicks. The title is a label with `selectable(false)`:
a selectable label would take the press for selecting text. Double-clicking the bar toggles
maximized, as a real title bar does. `StartDrag` only works while the left button is held
down, so it is sent when a drag starts (the pointer has moved a little with the button still
down), and egui-winit ignores it if the window does not have focus. The window has no resize
border either; `ViewportCommand::BeginResize` lets you build resize handles of your own. The
tests in this example check that each part sends the right command.

### Transparent window

**Use this when** the window should not be a rectangle: a rounded card, a splash screen, an
overlay.

The browser has no equivalent: a page cannot make the window behind it see-through.

```rust
            // Both are needed for a window of any shape: no border, and no background.
            .with_decorations(false)
            .with_transparent(true),
```

`with_transparent(true)` only gives the window an alpha channel. Two things still paint over
it. eframe clears every frame to `App::clear_color`, a partly see-through dark grey by default,
so return a fully transparent colour:

```rust
    /// The colour eframe clears the window to before each frame
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        // eframe's default is a dark grey that is partly see-through. Clear to nothing instead.
        egui::Color32::TRANSPARENT.to_normalized_gamma_f32()
    }
```

and the default `CentralPanel` paints the whole window, so use `CentralPanel::no_frame()` and
paint only the shape you want:

```rust
        // The usual central panel would paint the whole window. Paint a rounded card instead,
        // and leave the corners outside it empty.
        let card = egui::Frame::NONE
            .fill(ui.visuals().window_fill())
            .stroke(ui.visuals().window_stroke())
            .corner_radius(24.0)
            .inner_margin(20.0);
        egui::CentralPanel::no_frame().show(ui, |ui| {
            card.show(ui, |ui| {
```

The card is also the drag region, the same way as the title bar above.

Run: `cargo run -p concepts --example viewport_builder_transparent`

There is no screenshot: the tool that takes them asks the window to paint itself into an
image, and that turns the see-through corners white. On screen, the desktop shows through
them. Whether transparency works at all depends on the platform and the graphics driver;
egui's documentation calls it "more of a hint" on some systems. On macOS, its documentation
also suggests `with_has_shadow(false)` to avoid ghosting around the shape.

### Always on top

**Use this when** a small window must stay visible over others: a timer, a colour picker, a
video call.

**The JS way:** Chromium's Document Picture-in-Picture API opens a small window that stays
above the others. It needs a click to start it, and the page cannot turn "on top" off.

```js
button.onclick = async () => {
  const pip = await documentPictureInPicture.requestWindow({ width: 400, height: 140 });
  pip.document.body.append(timer);
};
```

**The egui way:** a window level in the builder, which the example's checkbox changes with
`ViewportCommand::WindowLevel(WindowLevel::Normal)` and back.

```rust
            // Short for `.with_window_level(egui::WindowLevel::AlwaysOnTop)`.
            .with_always_on_top(),
```

Run: `cargo run -p concepts --example viewport_builder_always_on_top`

The app keeps the level in its own field, because egui does not report it back. There is
also `WindowLevel::AlwaysOnBottom`, for a window that stays behind all others. The exact
behaviour is the operating system's; winit's documentation lists the platform differences.

### Window icon

**Use this when** the app should have its own icon in the title bar, the taskbar and the
Alt+Tab list, instead of eframe's white "e" on black.

**The JS way:** a link in the page's head, and a script that sets a new `href` to change it.

```html
<link rel="icon" href="red.png">
<script>document.querySelector("link[rel=icon]").href = "blue.png";</script>
```

**The egui way:** `IconData` is raw pixels, four bytes per pixel (red, green, blue, alpha, the
alpha not premultiplied), with a width and height. The example's `round_icon` fills them in
to draw a disc, so it needs no image file:

```rust
            // Without this, eframe uses its own icon: a white "e" on black.
            .with_icon(round_icon(RED)),
```

and the `Icon` command changes it:

```rust
            if ui.button("Change the icon").clicked() {
                self.red = !self.red;
                let colour = if self.red { RED } else { BLUE };
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::Icon(Some(Arc::new(round_icon(
                        colour,
                    )))));
            }
```

Run: `cargo run -p concepts --example viewport_builder_icon`

![A window with a red disc as its icon in the title bar](../images/concepts/viewport-builder-icon.png)

A real app usually embeds a PNG: `eframe::icon_data::from_png_bytes(include_bytes!(..))`
returns an `IconData`. egui asks for a square image whose width and height are multiples of 4,
such as 256 by 256. `IconData::default()`, which is empty, means "the operating system's
default icon" instead of eframe's. On Windows, eframe sets both the large icon (taskbar,
Alt+Tab) and the small one (title bar) at start-up. The `Icon` command goes through winit,
which, according to a comment in eframe's source, sets only the small one, so the taskbar may
keep the start-up icon.

### Maximized, minimized and fullscreen

**Use this when** the app has a fullscreen mode (a presentation, a video, a game), or a button
that maximizes the window.

**The JS way:** the Fullscreen API. The browser leaves fullscreen on Esc by itself. There is
nothing for maximizing or minimizing.

```js
button.onclick = () => document.documentElement.requestFullscreen();
const isFullscreen = document.fullscreenElement !== null;
```

**The egui way:** `with_maximized(true)` or `with_fullscreen(true)` in the builder to start
that way; at run time, read the state from the viewport info and send a command:

```rust
        let (maximized, fullscreen) = ui.input(|input| {
            let viewport = input.viewport();
            (
                viewport.maximized.unwrap_or(false),
                viewport.fullscreen.unwrap_or(false),
            )
        });
```

```rust
                let maximize = if maximized { "Restore" } else { "Maximize" };
                if ui.button(maximize).clicked() {
                    ui.ctx()
                        .send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized));
                }
                let full = if fullscreen {
                    "Leave fullscreen (Esc)"
                } else {
                    "Fullscreen"
                };
                if ui.button(full).clicked() {
                    ui.ctx()
                        .send_viewport_cmd(egui::ViewportCommand::Fullscreen(!fullscreen));
                }
```

Run: `cargo run -p concepts --example viewport_builder_window_state`

egui's fullscreen is always borderless fullscreen: the window covers the monitor it is on,
without changing the screen's resolution. Unlike a browser, eframe does not leave fullscreen
on Esc, so the example sends `Fullscreen(false)` when Esc is pressed, as any app with a
fullscreen mode should. `maximized` and `fullscreen` in the viewport info are `Option`s:
`None` until the platform has reported them.
`Minimized(true)` has no builder method, so the builder cannot open a window minimized.

### Fixed-size window

**Use this when** the window is a dialog or a tool whose layout has one size.

The browser has no equivalent: the `resizable` feature of `window.open` is ignored by current
browsers.

```rust
            // No resizing by the edges, and no maximize button in the title bar.
            .with_resizable(false)
            .with_maximize_button(false),
```

Run: `cargo run -p concepts --example viewport_builder_resizable`

![A window whose maximize button is greyed out, with a Resizable checkbox](../images/concepts/viewport-builder-resizable.png)

`with_resizable(false)` stops resizing by the edges, but the maximize button would still
change the size, so it is turned off too (it shows greyed out). The example's checkbox sends
`Resizable(true)` and `EnableButtons { close: true, minimized: true, maximize: true }`: the
three buttons share one command, which sets all three at once. In the builder they are three
methods, and for an extra window, changing one of them in the builder makes egui recreate the
window rather than send `EnableButtons`.

## Best practices

- **Set a title, an inner size and a minimum inner size**, and leave the rest unset unless you
  need it. The defaults are the ones the user expects of a normal window.
- **Change the window with `ViewportCommand`.** Building a new `ViewportBuilder` for the root
  window does nothing after start-up.
- **Keep your own copy of what egui does not report**, such as the window level or whether
  the window is resizable, and read the rest from `ui.input(|i| i.viewport())`.
- **Give a borderless window every job of the title bar**: dragging, closing, minimizing,
  double-click to maximize, and resizing if it is resizable.
- **Give fullscreen a keyboard way out.** Esc is what users try first.

## Common mistakes

**Turning decorations off and losing the window.** Without a title bar there is nothing to drag
or close the window with. Draw a title bar, or at least send `StartDrag` when the
background is dragged, and give a way to close.

**Seeing a dark grey background in a transparent window.** `with_transparent(true)` is not
enough: return a transparent `clear_color`, and use `CentralPanel::no_frame()` or a frame
with a transparent fill.

**Sending `StartDrag` from `clicked()`.** A click is reported when the button is released,
and by then there is no drag to hand over. Send it while the button is down: from
`drag_started_by(PointerButton::Primary)`, as the example does.

**Expecting `with_position` to place the content area.** It places the outer corner of the
window, title bar included. And with `centered: true` in `NativeOptions`, eframe replaces it.

**Changing `with_inner_size` and seeing no difference.** With persistence on, the saved size
from the last run wins; see [`NativeOptions`](native-options.md#common-mistakes).

**Reading `viewport().maximized` as a `bool`.** It is an `Option<bool>`, `None` until the
platform has said; `unwrap_or(false)` is the usual answer.

## See also

- [Hello World](../chapters/03-hello-world.md), where `ViewportBuilder` first appears.
- [`NativeOptions`](native-options.md), the start-up settings that hold the builder, and the
  table of what goes where.
- [`CentralPanel`](central-panel.md), and its frame, which a transparent window replaces.
- docs.rs: [`ViewportBuilder`](https://docs.rs/egui/0.37.0/egui/viewport/struct.ViewportBuilder.html),
  [`ViewportCommand`](https://docs.rs/egui/0.37.0/egui/viewport/enum.ViewportCommand.html),
  [`ViewportInfo`](https://docs.rs/egui/0.37.0/egui/struct.ViewportInfo.html),
  [`IconData`](https://docs.rs/egui/0.37.0/egui/viewport/struct.IconData.html), and egui's
  [viewport module](https://docs.rs/egui/0.37.0/egui/viewport/index.html) on extra windows.
