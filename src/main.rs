mod bounds;
mod dag;
mod image;
mod jit;
mod vec2;

use core::mem;
use cranelift_module::ModuleError;
use dag::{Dag, Intrinsic, Node, NodeKind};
use iced::{
    mouse,
    widget::{
        canvas::{
            self,
            event::{self, Event},
            Canvas, Frame, Geometry, Path, Stroke,
        },
        column,
    },
    Alignment, Element, Length, Point, Rectangle, Renderer, Sandbox, Settings, Theme,
};
use vec2::Vec2;

pub fn main() -> iced::Result {
    Ui::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
}

#[derive(Default)]
struct Ui {
    graph: Graph,
    dag: Dag,
}

impl Sandbox for Ui {
    type Message = Message;

    fn new() -> Self {
        Ui::default()
    }

    fn title(&self) -> String {
        String::from("Madeline")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::CreateNode { kind, position } => {
                self.dag
                    .add_node(Node::with_kind(kind).positioned(position));
            }
        }
    }

    fn view(&self) -> Element<Message> {
        column![self.graph.view(&self.dag)]
            .padding(20)
            .spacing(20)
            .align_items(Alignment::Center)
            .into()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    CreateNode { kind: NodeKind, position: Vec2<i32> },
}

#[derive(Default)]
pub struct Graph {
    cache: canvas::Cache,
}

impl Graph {
    pub fn view<'a>(&'a self, dag: &'a Dag) -> Element<'a, Message> {
        Canvas::new(Program {
            cache: &self.cache,
            dag,
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    pub fn request_redraw(&mut self) {
        self.cache.clear();
    }
}

#[derive(Debug, Default, Clone)]
struct State {
    zoom: i8,
    position: Vec2<i32>,
    selection: u32,
    pending_node_create: Option<String>,
}

struct Program<'a> {
    cache: &'a canvas::Cache,
    dag: &'a Dag,
}

impl<'a> canvas::Program<Message> for Program<'a> {
    type State = State;

    fn update(
        &self,
        state: &mut Self::State,
        event: Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> (event::Status, Option<Message>) {
        let Some(cursor_position) = cursor.position_in(bounds) else {
            return (event::Status::Ignored, None);
        };

        match event {
            Event::Mouse(mouse_event) => (event::Status::Captured, None),
            _ => (event::Status::Ignored, None),
        }
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let content = self
            .cache
            .draw(renderer, bounds.size(), |frame: &mut Frame| {
                frame.stroke(
                    &Path::rectangle(Point::ORIGIN, frame.size()),
                    Stroke::default().with_width(2.0),
                );
            });

        vec![content]
    }
}

fn jit_main() -> Result<(), ModuleError> {
    let mut jit = jit::Jit::default();
    let mut program = Dag::new();
    let one = program.add_node(Node::with_kind(NodeKind::Constant(1.)));
    let two = program.add_node(Node::with_kind(NodeKind::Constant(2.)));
    let three = program.add_node(Node::with_kind(NodeKind::Constant(3.)));
    let add = program.add_node(Node::with_kind(NodeKind::Intrinsic(Intrinsic::Add(
        one, two,
    ))));
    let mul = program.add_node(Node::with_kind(NodeKind::Intrinsic(Intrinsic::Mul(
        add, three,
    ))));
    program.set_out_node(mul);
    let v: f32 = unsafe { run_code(&mut jit, &program, ()) }?;
    println!("{v}");
    Ok(())
}

unsafe fn run_code<I, O>(jit: &mut jit::Jit, code: &Dag, input: I) -> Result<O, ModuleError> {
    let code_ptr = jit.compile(code)?;
    let code_fn = mem::transmute::<_, fn(I) -> O>(code_ptr);
    Ok(code_fn(input))
}
