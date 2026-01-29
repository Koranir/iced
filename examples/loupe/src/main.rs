use iced::widget::{button, center, column, text};
use iced::{Center, Element};

use loupe::loupe;

pub fn main() -> iced::Result {
    iced::run(Loupe::update, Loupe::view)
}

#[derive(Default)]
struct Loupe {
    value: i64,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Increment,
    Decrement,
}

impl Loupe {
    fn update(&mut self, message: Message) {
        match message {
            Message::Increment => {
                self.value += 1;
            }
            Message::Decrement => {
                self.value -= 1;
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        column![
            Element::new(center(wev::LoadingText::new(self.value.to_string()))),
            button("Increment").on_press(Message::Increment),
            button("Decrement").on_press(Message::Decrement)
        ]
        .into()
        // center(loupe(
        //     3.0,
        //     column![
        //     ]
        //     .padding(20)
        //     .align_x(Center),
        // ))
        // .into()
    }
}

mod loupe {
    use iced::advanced::Renderer as _;
    use iced::advanced::layout::{self, Layout};
    use iced::advanced::renderer;
    use iced::advanced::widget::{self, Widget};
    use iced::{Color, Element, Length, Rectangle, Renderer, Size, Theme, Transformation};
    use iced::{Point, mouse};

    pub fn loupe<'a, Message>(
        zoom: f32,
        content: impl Into<Element<'a, Message>>,
    ) -> Loupe<'a, Message>
    where
        Message: 'static,
    {
        Loupe {
            zoom,
            content: content.into().explain(Color::BLACK),
        }
    }

    pub struct Loupe<'a, Message> {
        zoom: f32,
        content: Element<'a, Message>,
    }

    struct State {
        cursor_over: Option<Point>,
    }

    impl<Message> Widget<Message, Theme, Renderer> for Loupe<'_, Message> {
        fn tag(&self) -> widget::tree::Tag {
            widget::tree::Tag::of::<State>()
        }

        fn state(&self) -> widget::tree::State {
            widget::tree::State::new(State { cursor_over: None })
        }

        fn children(&self) -> Vec<widget::Tree> {
            vec![widget::Tree::new(&self.content)]
        }

        fn diff(&self, tree: &mut widget::Tree) {
            tree.diff_children(&[&self.content]);
        }

        fn size(&self) -> Size<Length> {
            self.content.as_widget().size()
        }

        fn layout(
            &mut self,
            tree: &mut widget::Tree,
            renderer: &Renderer,
            limits: &layout::Limits,
        ) -> layout::Node {
            self.content
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, limits)
        }

        fn draw(
            &self,
            tree: &widget::Tree,
            renderer: &mut Renderer,
            theme: &Theme,
            style: &renderer::Style,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            viewport: &Rectangle,
        ) {
            let bounds = layout.bounds();

            let state = tree.state.downcast_ref::<State>();

            if let Some(position) = state.cursor_over {
                renderer.with_layer(bounds, |renderer| {
                    renderer.with_transformation(
                        Transformation::translate(
                            bounds.x + position.x * (1.0 - self.zoom),
                            bounds.y + position.y * (1.0 - self.zoom),
                        ) * Transformation::scale(self.zoom)
                            * Transformation::translate(-bounds.x, -bounds.y),
                        |renderer| {
                            self.content.as_widget().draw(
                                &tree.children[0],
                                renderer,
                                theme,
                                style,
                                layout,
                                mouse::Cursor::Unavailable,
                                viewport,
                            );
                        },
                    );
                });
            } else {
                self.content.as_widget().draw(
                    &tree.children[0],
                    renderer,
                    theme,
                    style,
                    layout,
                    cursor,
                    viewport,
                );
            }
        }

        fn update(
            &mut self,
            tree: &mut widget::Tree,
            _event: &iced::Event,
            layout: Layout<'_>,
            cursor: iced::advanced::mouse::Cursor,
            _renderer: &Renderer,
            _clipboard: &mut dyn iced::advanced::Clipboard,
            shell: &mut iced::advanced::Shell<'_, Message>,
            _viewport: &Rectangle,
        ) {
            let state = tree.state.downcast_mut::<State>();

            let position = cursor.position_in(layout.bounds());

            if position != state.cursor_over {
                shell.request_redraw();
            }

            state.cursor_over = position;
        }

        fn mouse_interaction(
            &self,
            _tree: &widget::Tree,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            _viewport: &Rectangle,
            _renderer: &Renderer,
        ) -> mouse::Interaction {
            if cursor.is_over(layout.bounds()) {
                mouse::Interaction::ZoomIn
            } else {
                mouse::Interaction::None
            }
        }
    }

    impl<'a, Message> From<Loupe<'a, Message>> for Element<'a, Message, Theme, Renderer>
    where
        Message: 'a,
    {
        fn from(loupe: Loupe<'a, Message>) -> Self {
            Self::new(loupe)
        }
    }
}

mod wev {
    use iced::advanced::layout::{Limits, Node};
    use iced::advanced::renderer::Style;
    use iced::advanced::text::{Fragment, IntoFragment};
    use iced::advanced::widget::Operation;
    use iced::advanced::widget::Tree;
    use iced::advanced::widget::text;
    use iced::advanced::widget::tree::{State, Tag};
    use iced::advanced::{Layout, Widget};
    use iced::mouse::Cursor;
    use iced::{Element, Length, Rectangle, Size};
    use iced::{Event, Pixels, advanced::Clipboard, advanced::Shell, alignment, window};
    use std::iter;
    use std::num::NonZero;
    use std::time::{Duration, Instant};

    pub struct LoadingText<'a, Theme, Renderer>
    where
        Theme: text::Catalog,
        Renderer: iced::advanced::text::Renderer,
    {
        text: iced::widget::Text<'a, Theme, Renderer>,
        size: Option<Pixels>,
        align_x: Option<text::Alignment>,
        align_y: Option<alignment::Vertical>,
        width: Option<Length>,
        height: Option<Length>,

        rendered_index: u32,
        str: Fragment<'a>,
        cycle_duration: Duration,
        len_bobber: NonZero<u32>,
    }

    impl<'a, Theme, Renderer> LoadingText<'a, Theme, Renderer>
    where
        Theme: text::Catalog + 'a,
        Renderer: iced::advanced::text::Renderer,
    {
        fn make_text<'b>(
            str: &'b str,
            index: u32,
            len_bobber: NonZero<u32>,
            size: Option<Pixels>,
            align_x: Option<text::Alignment>,
            align_y: Option<alignment::Vertical>,
            width: Option<Length>,
            height: Option<Length>,
        ) -> iced::widget::Text<'a, Theme, Renderer> {
            let lengths = index.checked_add(1).and_then(|extra_len| {
                let empty = len_bobber.get().checked_sub(extra_len)?;
                Some((extra_len, empty))
            });

            let Some((extra_len, empty)) = lengths else {
                panic!("index out of bounds")
            };

            let len_bobber = usize::try_from(len_bobber.get()).expect("capacity overflow");

            let (empty, extra_len) = (empty as usize, extra_len as usize);

            let mut new_str = String::with_capacity(str.len() + len_bobber);

            new_str += str;
            new_str.extend(iter::repeat_n('.', extra_len));
            new_str.extend(iter::repeat_n(' ', empty));
            let mut text = iced::widget::text(new_str);

            if let Some(pixels) = size {
                text = text.size(pixels);
            }

            if let Some(align_x) = align_x {
                text = text.align_x(align_x)
            }

            if let Some(align_y) = align_y {
                text = text.align_y(align_y)
            }

            if let Some(width) = width {
                text = text.width(width)
            }

            if let Some(height) = height {
                text = text.height(height)
            }

            text
        }

        fn new_inner(str: Fragment<'a>) -> Self {
            let len_bobber = const { NonZero::new(3).unwrap() };
            let rendered_index = 0;
            let size = None;
            let align_x = None;
            let align_y = None;
            let width = None;
            let height = None;
            Self {
                text: Self::make_text(
                    &str,
                    rendered_index,
                    len_bobber,
                    size,
                    align_x,
                    align_y,
                    width,
                    height,
                ),
                size,
                align_x,
                align_y,
                width,
                height,

                rendered_index,
                str,
                cycle_duration: Duration::from_millis(650),
                len_bobber,
            }
        }

        fn set_index(&mut self, index: u32) {
            if self.rendered_index != index {
                self.text = Self::make_text(
                    &self.str,
                    index,
                    self.len_bobber,
                    self.size,
                    self.align_x,
                    self.align_y,
                    self.width,
                    self.height,
                );
                self.rendered_index = index;
            }
        }

        pub fn size(mut self, size: impl Into<Pixels>) -> Self {
            let size = size.into();
            self.size = Some(size);
            self.text = self.text.size(size);
            self
        }

        pub fn align_y(mut self, align_x: impl Into<alignment::Vertical>) -> Self {
            let align_x = align_x.into();
            self.align_y = Some(align_x);
            self.text = self.text.align_y(align_x);
            self
        }

        pub fn align_x(mut self, align_x: impl Into<text::Alignment>) -> Self {
            let align_x = align_x.into();
            self.align_x = Some(align_x);
            self.text = self.text.align_x(align_x);
            self
        }

        pub fn width(mut self, width: impl Into<Length>) -> Self {
            let width = width.into();
            self.width = Some(width);
            self.text = self.text.width(width);
            self
        }

        pub fn height(mut self, height: impl Into<Length>) -> Self {
            let height = height.into();
            self.height = Some(height);
            self.text = self.text.height(height);
            self
        }

        pub fn center(self) -> Self {
            self.align_x(alignment::Horizontal::Center)
                .align_y(alignment::Vertical::Center)
        }

        /// Creates a new [`LoadingText`] with the given content.
        pub fn new(str: impl IntoFragment<'a>) -> Self {
            Self::new_inner(str.into_fragment())
        }
    }

    pub struct LoadingState {
        start_tick: Instant,
        index: u32,
    }

    impl<'a, Theme, Renderer> LoadingText<'a, Theme, Renderer>
    where
        Theme: text::Catalog,
        Renderer: iced::advanced::text::Renderer,
    {
        fn text_child<Message>(&self) -> &dyn Widget<Message, Theme, Renderer> {
            (&self.text) as &dyn Widget<Message, Theme, Renderer>
        }
    }

    fn mod_duration(a: Duration, b: Duration) -> Duration {
        let rem_nanos = a.as_nanos() % b.as_nanos();
        const NANOS_PER_SECOND: u128 = Duration::from_secs(1).as_nanos();
        Duration::new(
            (rem_nanos / NANOS_PER_SECOND) as u64,
            (rem_nanos % NANOS_PER_SECOND) as u32,
        )
    }

    fn index_in_cycle(elapsed: Duration, cycle: Duration, n: NonZero<u32>) -> u32 {
        assert!(cycle > Duration::ZERO);

        let n_128 = u128::from(n.get());
        let t = elapsed.as_nanos() % cycle.as_nanos(); // [0, cycle)
        let idx = (t * n_128) / cycle.as_nanos(); // floor in [0, n]
        u32::try_from(idx)
            .unwrap_or(u32::MAX)
            .min(n.get() - 1)
            .into()
    }

    impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
        for LoadingText<'a, Theme, Renderer>
    where
        Theme: text::Catalog + 'a,
        Renderer: iced::advanced::text::Renderer + 'a,
    {
        fn size(&self) -> Size<Length> {
            Widget::<Message, Theme, Renderer>::size(&self.text)
        }

        fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
            Widget::<Message, Theme, Renderer>::layout(
                &mut self.text,
                &mut tree.children[0],
                renderer,
                limits,
            )
        }

        fn draw(
            &self,
            tree: &Tree,
            renderer: &mut Renderer,
            theme: &Theme,
            style: &Style,
            layout: Layout<'_>,
            cursor: Cursor,
            viewport: &Rectangle,
        ) {
            Widget::<Message, Theme, Renderer>::draw(
                &self.text,
                &tree.children[0],
                renderer,
                theme,
                style,
                layout,
                cursor,
                viewport,
            )
        }

        fn tag(&self) -> Tag {
            Tag::of::<LoadingState>()
        }

        fn state(&self) -> State {
            State::new(LoadingState {
                start_tick: Instant::now(),
                index: 0,
            })
        }

        fn children(&self) -> Vec<Tree> {
            vec![Tree::new(self.text_child::<Message>())]
        }

        fn diff(&self, tree: &mut Tree) {
            tree.diff_children(&[self.text_child::<Message>()]);
        }

        fn operate(
            &mut self,
            tree: &mut Tree,
            layout: Layout<'_>,
            renderer: &Renderer,
            operation: &mut dyn Operation,
        ) {
            Widget::<Message, Theme, Renderer>::operate(
                &mut self.text,
                &mut tree.children[0],
                layout,
                renderer,
                operation,
            )
        }

        fn update(
            &mut self,
            tree: &mut Tree,
            event: &Event,
            _layout: Layout<'_>,
            _cursor: Cursor,
            _renderer: &Renderer,
            _clipboard: &mut dyn Clipboard,
            shell: &mut Shell<'_, Message>,
            _viewport: &Rectangle,
        ) {
            if let Event::Window(window::Event::RedrawRequested(now)) = *event {
                let state = tree.state.downcast_mut::<LoadingState>();
                let elapsed = now.duration_since(state.start_tick);
                let time_in_cycle = mod_duration(elapsed, self.cycle_duration);
                let step = self.cycle_duration / self.len_bobber.get();

                let index = index_in_cycle(elapsed, self.cycle_duration, self.len_bobber);

                let extra = mod_duration(time_in_cycle, step);
                let duration_from_start_till_redraw = elapsed + extra;
                dbg!(duration_from_start_till_redraw);
                let redraw_at = state.start_tick + duration_from_start_till_redraw;

                if index != state.index {
                    state.index = index;
                    self.set_index(index);
                    shell.invalidate_layout()
                }

                shell.request_redraw_at(redraw_at);
            }
        }
    }

    impl<'a, Message, Theme, Renderer> From<LoadingText<'a, Theme, Renderer>>
        for Element<'a, Message, Theme, Renderer>
    where
        Message: Clone + 'a,
        Theme: text::Catalog + 'a,
        Renderer: iced::advanced::text::Renderer + 'a,
    {
        fn from(linear: LoadingText<'a, Theme, Renderer>) -> Self {
            Element::new(linear)
        }
    }
}
