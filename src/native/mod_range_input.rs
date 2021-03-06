//! Display an interactive dot that controls an [`NormalParam`]
//!
//! [`NormalParam`]: ../core/normal_param/struct.NormalParam.html

use std::fmt::Debug;

use iced_native::{
    event, keyboard, layout, mouse, Clipboard, Element, Event, Hasher, Layout,
    Length, Point, Rectangle, Size, Widget,
};

use std::hash::Hash;

use crate::core::{Normal, NormalParam};

static DEFAULT_SIZE: u16 = 10;
static DEFAULT_SCALAR: f32 = 0.00385 / 2.0;
static DEFAULT_MODIFIER_SCALAR: f32 = 0.02;

/// An interactive dot that controls an [`NormalParam`]
///
/// [`NormalParam`]: ../core/normal_param/struct.NormalParam.html
#[allow(missing_debug_implementations)]
pub struct ModRangeInput<'a, Message, Renderer: self::Renderer> {
    state: &'a mut State,
    size: Length,
    on_change: Box<dyn Fn(Normal) -> Message>,
    scalar: f32,
    modifier_scalar: f32,
    modifier_keys: keyboard::Modifiers,
    style: Renderer::Style,
}

impl<'a, Message, Renderer: self::Renderer>
    ModRangeInput<'a, Message, Renderer>
{
    /// Creates a new [`ModRangeInput`].
    ///
    /// It expects:
    ///   * the local [`State`] of the [`ModRangeInput`]
    ///   * a function that will be called when the [`ModRangeInput`] is turned.
    ///
    /// [`State`]: struct.State.html
    /// [`ModRangeInput`]: struct.ModRangeInput.html
    pub fn new<F>(state: &'a mut State, on_change: F) -> Self
    where
        F: 'static + Fn(Normal) -> Message,
    {
        ModRangeInput {
            state,
            size: Length::from(Length::Units(DEFAULT_SIZE)),
            on_change: Box::new(on_change),
            scalar: DEFAULT_SCALAR,
            modifier_scalar: DEFAULT_MODIFIER_SCALAR,
            modifier_keys: keyboard::Modifiers {
                control: true,
                ..Default::default()
            },
            style: Renderer::Style::default(),
        }
    }

    /// Sets the diameter of the [`ModRangeInput`]. The default size is
    /// `Length::from(Length::Units(31))`.
    ///
    /// [`ModRangeInput`]: struct.ModRangeInput.html
    pub fn size(mut self, size: Length) -> Self {
        self.size = size;
        self
    }

    /// Sets the style of the [`ModRangeInput`].
    ///
    /// [`ModRangeInput`]: struct.ModRangeInput.html
    pub fn style(mut self, style: impl Into<Renderer::Style>) -> Self {
        self.style = style.into();
        self
    }

    /// Sets how much the [`Normal`] value will change for the [`ModRangeInput`] per `y`
    /// pixel movement of the mouse.
    ///
    /// The default value is `0.001925`
    ///
    /// [`ModRangeInput`]: struct.ModRangeInput.html
    /// [`Normal`]: ../../core/struct.Normal.html
    pub fn scalar(mut self, scalar: f32) -> Self {
        self.scalar = scalar;
        self
    }

    /// Sets the modifier keys of the [`ModRangeInput`].
    ///
    /// The default modifier key is `Ctrl`.
    ///
    /// [`ModRangeInput`]: struct.ModRangeInput.html
    pub fn modifier_keys(mut self, modifier_keys: keyboard::Modifiers) -> Self {
        self.modifier_keys = modifier_keys;
        self
    }

    /// Sets the scalar to use when the user drags the ModRangeInputs while holding down
    /// the modifier key. This is multiplied to the value set by
    /// `ModRangeInput::scalar()` (which the default is `0.001925`).
    ///
    /// For example, a `modifier_scalar` of `0.5` will cause the ModRangeInput to turn
    /// half as fast when the modifier key is down.
    ///
    /// The default `modifier_scalar` is `0.02`, and the default modifier key
    /// is `Ctrl`.
    ///
    /// [`ModRangeInput`]: struct.ModRangeInput.html
    pub fn modifier_scalar(mut self, scalar: f32) -> Self {
        self.modifier_scalar = scalar;
        self
    }
}

/// The local state of an [`ModRangeInput`].
///
/// [`ModRangeInput`]: struct.ModRangeInput.html
#[derive(Debug, Copy, Clone)]
pub struct State {
    /// The [`NormalParam`] assigned to this widget
    ///
    /// [`NormalParam`]: ../../core/normal_param/struct.NormalParam.html
    pub normal_param: NormalParam,
    is_dragging: bool,
    prev_drag_y: f32,
    continuous_normal: f32,
    pressed_modifiers: keyboard::Modifiers,
    last_click: Option<mouse::Click>,
}

impl State {
    /// Creates a new [`ModRangeInput`] state.
    ///
    /// It expects:
    /// * a [`NormalParam`] to assign to this widget
    ///
    /// [`NormalParam`]: ../../core/normal_param/struct.NormalParam.html
    /// [`ModRangeInput`]: struct.ModRangeInput.html
    pub fn new(normal_param: NormalParam) -> Self {
        Self {
            normal_param,
            is_dragging: false,
            prev_drag_y: 0.0,
            continuous_normal: normal_param.value.as_f32(),
            pressed_modifiers: Default::default(),
            last_click: None,
        }
    }

    /// Set the `normal_param.value` of the [`ModRangeInput`].
    pub fn set(&mut self, normal: Normal) {
        self.normal_param.value = normal;
        self.continuous_normal = normal.into();
    }

    /// Is the [`ModRangeInput`] currently in the dragging state?
    ///
    /// [`ModRangeInput`]: struct.ModRangeInput.html
    pub fn is_dragging(&self) -> bool {
        self.is_dragging
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for ModRangeInput<'a, Message, Renderer>
where
    Renderer: self::Renderer,
{
    fn width(&self) -> Length {
        self.size
    }

    fn height(&self) -> Length {
        self.size
    }

    fn layout(
        &self,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(self.size).height(self.size);

        let size = limits.resolve(Size::ZERO);

        layout::Node::new(size)
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        messages: &mut Vec<Message>,
        _renderer: &Renderer,
        _clipboard: Option<&dyn Clipboard>,
    ) -> event::Status {
        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::CursorMoved { .. } => {
                    if self.state.is_dragging && cursor_position.y != -1.0 {
                        let mut movement_y = (cursor_position.y
                            - self.state.prev_drag_y)
                            * self.scalar;

                        if self
                            .state
                            .pressed_modifiers
                            .matches(self.modifier_keys)
                        {
                            movement_y *= self.modifier_scalar;
                        }

                        let mut normal =
                            self.state.continuous_normal - movement_y;

                        if normal < 0.0 {
                            normal = 0.0;
                        } else if normal > 1.0 {
                            normal = 1.0;
                        }

                        self.state.continuous_normal = normal;
                        self.state.prev_drag_y = cursor_position.y;

                        self.state.normal_param.value = normal.into();

                        messages.push((self.on_change)(
                            self.state.normal_param.value,
                        ));

                        return event::Status::Captured;
                    }
                }
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    if layout.bounds().contains(cursor_position) {
                        let click = mouse::Click::new(
                            cursor_position,
                            self.state.last_click,
                        );

                        match click.kind() {
                            mouse::click::Kind::Single => {
                                self.state.is_dragging = true;
                                self.state.prev_drag_y = cursor_position.y;
                            }
                            _ => {
                                self.state.is_dragging = false;

                                self.state.normal_param.value =
                                    self.state.normal_param.default;

                                messages.push((self.on_change)(
                                    self.state.normal_param.value,
                                ));
                            }
                        }

                        self.state.last_click = Some(click);

                        return event::Status::Captured;
                    }
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    self.state.is_dragging = false;
                    self.state.continuous_normal =
                        self.state.normal_param.value.as_f32();

                    return event::Status::Captured;
                }
                _ => {}
            },
            Event::Keyboard(keyboard_event) => match keyboard_event {
                keyboard::Event::KeyPressed { modifiers, .. } => {
                    self.state.pressed_modifiers = modifiers;

                    return event::Status::Captured;
                }
                keyboard::Event::KeyReleased { modifiers, .. } => {
                    self.state.pressed_modifiers = modifiers;

                    return event::Status::Captured;
                }
                _ => {}
            },
            _ => {}
        }

        event::Status::Ignored
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) -> Renderer::Output {
        renderer.draw(
            layout.bounds(),
            cursor_position,
            self.state.is_dragging,
            &self.style,
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.size.hash(state);
    }
}

/// The renderer of an [`ModRangeInput`].
///
/// Your renderer will need to implement this trait before being
/// able to use an [`ModRangeInput`] in your user interface.
///
/// [`ModRangeInput`]: struct.ModRangeInput.html
pub trait Renderer: iced_native::Renderer {
    /// The style supported by this renderer.
    type Style: Default;

    /// Draws an [`ModRangeInput`].
    ///
    /// It receives:
    ///   * the bounds of the [`ModRangeInput`]
    ///   * the current cursor position
    ///   * whether the ModRangeInput is currently being dragged
    ///   * the style of the [`ModRangeInput`]
    ///
    /// [`ModRangeInput`]: struct.ModRangeInput.html
    fn draw(
        &mut self,
        bounds: Rectangle,
        cursor_position: Point,
        is_dragging: bool,
        style: &Self::Style,
    ) -> Self::Output;
}

impl<'a, Message, Renderer> From<ModRangeInput<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: 'a + self::Renderer,
    Message: 'a,
{
    fn from(
        mod_range_input: ModRangeInput<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(mod_range_input)
    }
}
