//! An animated solar system.
//!
//! This example showcases how to use a `Canvas` widget with transforms to draw
//! using different coordinate systems.
//!
//! Inspired by the example found in the MDN docs[1].
//!
//! [1]: https://developer.mozilla.org/en-US/docs/Web/API/Canvas_API/Tutorial/Basic_animations#An_animated_solar_system
use iced::{
    canvas, executor, window, Application, Canvas, Color, Command, Container,
    Element, Length, Point, Settings, Size, Subscription, Vector,
};
use iced_native::input::{self, mouse};

use std::time::Instant;

pub fn main() {
    SolarSystem::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
}

struct SolarSystem {
    state: State,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Tick(Instant),
}

impl Application for SolarSystem {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            SolarSystem {
                state: State::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Solar system - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Tick(instant) => {
                self.state.update(instant);
            }
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(std::time::Duration::from_millis(10))
            .map(|instant| Message::Tick(instant))
    }

    fn view(&mut self) -> Element<Message> {
        let canvas = Canvas::new(&mut self.state)
            .width(Length::Fill)
            .height(Length::Fill);

        Container::new(canvas)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

#[derive(Debug)]
struct State {
    space_cache: canvas::Cache,
    system_cache: canvas::Cache,
    cursor_position: Point,
    start: Instant,
    now: Instant,
    stars: Vec<(Point, f32)>,
}

impl State {
    pub fn new() -> State {
        let now = Instant::now();
        let (width, height) = window::Settings::default().size;

        State {
            space_cache: Default::default(),
            system_cache: Default::default(),
            cursor_position: Point::ORIGIN,
            start: now,
            now,
            stars: Self::generate_stars(width, height),
        }
    }

    pub fn space(&self) -> Space<'_> {
        Space { stars: &self.stars }
    }

    pub fn system(&self) -> System {
        System {
            start: self.start,
            now: self.now,
        }
    }

    pub fn update(&mut self, now: Instant) {
        self.now = now;
        self.system_cache.clear();
    }

    fn generate_stars(width: u32, height: u32) -> Vec<(Point, f32)> {
        use rand::Rng;

        let mut rng = rand::thread_rng();

        (0..100)
            .map(|_| {
                (
                    Point::new(
                        rng.gen_range(0.0, width as f32),
                        rng.gen_range(0.0, height as f32),
                    ),
                    rng.gen_range(0.5, 1.0),
                )
            })
            .collect()
    }
}

impl canvas::State for State {
    fn update(&mut self, event: canvas::Event, _bounds: Size) {
        match event {
            canvas::Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::CursorMoved { x, y } => {
                    self.cursor_position = Point::new(x, y);
                }
                mouse::Event::Input {
                    button: mouse::Button::Left,
                    state: input::ButtonState::Released,
                } => {
                    self.stars.push((self.cursor_position, 2.0));
                    self.space_cache.clear();
                }
                _ => {}
            },
        }
    }

    fn draw(&self, bounds: Size) -> Vec<canvas::Geometry> {
        vec![
            self.space_cache.draw(bounds, self.space()),
            self.system_cache.draw(bounds, self.system()),
        ]
    }
}

#[derive(Debug)]
struct Space<'a> {
    stars: &'a [(Point, f32)],
}

impl canvas::Drawable for Space<'_> {
    fn draw(&self, frame: &mut canvas::Frame) {
        use canvas::Path;

        let space = Path::rectangle(Point::new(0.0, 0.0), frame.size());

        let stars = Path::new(|path| {
            for (p, size) in self.stars {
                path.rectangle(*p, Size::new(*size, *size));
            }
        });

        frame.fill(&space, Color::BLACK);
        frame.fill(&stars, Color::WHITE);
    }
}

#[derive(Debug)]
struct System {
    start: Instant,
    now: Instant,
}

impl System {
    const SUN_RADIUS: f32 = 70.0;
    const ORBIT_RADIUS: f32 = 150.0;
    const EARTH_RADIUS: f32 = 12.0;
    const MOON_RADIUS: f32 = 4.0;
    const MOON_DISTANCE: f32 = 28.0;
}

impl canvas::Drawable for System {
    fn draw(&self, frame: &mut canvas::Frame) {
        use canvas::{Path, Stroke};
        use std::f32::consts::PI;

        let center = frame.center();

        let sun = Path::circle(center, Self::SUN_RADIUS);
        let orbit = Path::circle(center, Self::ORBIT_RADIUS);

        frame.fill(&sun, Color::from_rgb8(0xF9, 0xD7, 0x1C));
        frame.stroke(
            &orbit,
            Stroke {
                width: 1.0,
                color: Color::from_rgba8(0, 153, 255, 0.1),
                ..Stroke::default()
            },
        );

        let elapsed = self.now - self.start;
        let elapsed_seconds = elapsed.as_secs() as f32;
        let elapsed_millis = elapsed.subsec_millis() as f32;

        frame.with_save(|frame| {
            frame.translate(Vector::new(center.x, center.y));
            frame.rotate(
                (2.0 * PI / 60.0) * elapsed_seconds
                    + (2.0 * PI / 60_000.0) * elapsed_millis,
            );
            frame.translate(Vector::new(Self::ORBIT_RADIUS, 0.0));

            let earth = Path::circle(Point::ORIGIN, Self::EARTH_RADIUS);
            let shadow = Path::rectangle(
                Point::new(0.0, -Self::EARTH_RADIUS),
                Size::new(Self::EARTH_RADIUS * 4.0, Self::EARTH_RADIUS * 2.0),
            );

            frame.fill(&earth, Color::from_rgb8(0x6B, 0x93, 0xD6));

            frame.with_save(|frame| {
                frame.rotate(
                    ((2.0 * PI) / 6.0) * elapsed_seconds
                        + ((2.0 * PI) / 6_000.0) * elapsed_millis,
                );
                frame.translate(Vector::new(0.0, Self::MOON_DISTANCE));

                let moon = Path::circle(Point::ORIGIN, Self::MOON_RADIUS);
                frame.fill(&moon, Color::WHITE);
            });

            frame.fill(
                &shadow,
                Color {
                    a: 0.7,
                    ..Color::BLACK
                },
            );
        });
    }
}

mod time {
    use iced::futures;
    use std::time::Instant;

    pub fn every(duration: std::time::Duration) -> iced::Subscription<Instant> {
        iced::Subscription::from_recipe(Every(duration))
    }

    struct Every(std::time::Duration);

    impl<H, I> iced_native::subscription::Recipe<H, I> for Every
    where
        H: std::hash::Hasher,
    {
        type Output = Instant;

        fn hash(&self, state: &mut H) {
            use std::hash::Hash;

            std::any::TypeId::of::<Self>().hash(state);
            self.0.hash(state);
        }

        fn stream(
            self: Box<Self>,
            _input: futures::stream::BoxStream<'static, I>,
        ) -> futures::stream::BoxStream<'static, Self::Output> {
            use futures::stream::StreamExt;

            async_std::stream::interval(self.0)
                .map(|_| Instant::now())
                .boxed()
        }
    }
}
