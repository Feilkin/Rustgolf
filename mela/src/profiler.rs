//! imgui based frame profiler and stuff

use imgui::Ui;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

pub trait PushTag: Sized {
    type Child: PopTag;

    fn push_tag(self, label: &'static str, color: [f32; 4]) -> Self::Child;
}

pub trait PopTag: Sized {
    type Into;

    fn pop_tag(self) -> Self::Into;
}

#[derive(Default)]
pub struct ProfilerUi {
    selected_frame: Option<Frame>,
}

pub struct Profiler {
    frames: VecDeque<Frame>,
    ui: ProfilerUi,
}

impl Profiler {
    pub fn new(frame_capacity: usize) -> Profiler {
        Profiler {
            frames: VecDeque::with_capacity(frame_capacity),
            ui: ProfilerUi::default(),
        }
    }

    pub fn start_frame(self) -> OpenFrame {
        OpenFrame::new(self)
    }

    fn end_frame(mut self, frame: Frame) -> Profiler {
        // TODO: this is a really stupid way to achieve fixed sized deques
        if self.frames.len() == self.frames.capacity() {
            // remove oldest frame to keep the VecDeque from growing
            self.frames.pop_back();
        }

        self.frames.push_front(frame);
        self
    }

    pub fn draw(&mut self, ui: &Ui) {
        use imgui::*;

        Window::new(im_str!("Profiler"))
            .size([500., 300.], Condition::FirstUseEver)
            .build(ui, || {
                self.draw_frame_times(ui);

                match &self.ui.selected_frame {
                    Some(frame) => self.draw_selected_frame_breakout(ui, frame),
                    None => (),
                }
            });
    }

    fn draw_frame_times(&mut self, ui: &Ui) {
        use imgui::*;

        let frame_times: Vec<f32> = self
            .frames
            .iter()
            .map(|frame| (frame.end_time - frame.start_time).as_secs_f32())
            .collect();

        let histogram_start = ui.cursor_screen_pos();
        let histogram_width = ui.calc_item_width();
        let item_width = histogram_width / frame_times.len() as f32;

        PlotHistogram::new(ui, im_str!("Frame times"), &frame_times)
            .graph_size([0., 64.])
            .build();

        // we try to get the hovered manually here, because imgui doesn't support getting the
        // hovered histogram item at the moment, maybe?
        if ui.is_item_hovered() && ui.is_mouse_down(MouseButton::Left) {
            let mouse_pos = ui.io().mouse_pos;
            let offset = mouse_pos[0] - histogram_start[0];
            let index = (offset/item_width).floor() as usize;

            let frame = self.frames[index].clone();
            self.ui.selected_frame = Some(frame);
        }
    }

    fn draw_selected_frame_breakout(&self, ui: &Ui, frame: &Frame) {
        use imgui::*;

        let duration = frame.end_time - frame.start_time;

        ui.group(|| {
            let single_item_height = 24.;
            let max_width = ui.content_region_avail()[0];

            let one_sec = max_width / duration.as_secs_f32();

            for tree in &frame.tag_trees {
                let node_duration = tree.tag.pop_time - tree.tag.push_time;
                let color_token = ui.push_style_color(StyleColor::Button, tree.tag.color);

                ui.button(&im_str!("{}", tree.tag.label), [one_sec * node_duration.as_secs_f32(), single_item_height]);

                if ui.is_item_hovered() {
                    ui.tooltip(|| {
                        ui.text(im_str!("{}: {}ms", tree.tag.label, node_duration.as_secs_f64() * 1000.));
                    });
                }

                color_token.pop(ui);
                ui.same_line(0.);
            }
        });

        ui.text(im_str!("total: {}ms", duration.as_secs_f64() * 1000.));
    }
}

#[derive(Clone)]
struct Frame {
    tag_trees: Vec<ClosedTagTree>,
    start_time: Instant,
    end_time: Instant,
}

#[must_use]
pub struct OpenFrame {
    closed_tags: Vec<ClosedTagTree>,
    start_time: Instant,
    profiler: Profiler,
}

impl OpenFrame {
    pub fn new(profiler: Profiler) -> OpenFrame {
        OpenFrame {
            closed_tags: Vec::new(),
            start_time: Instant::now(),
            profiler,
        }
    }

    pub fn finish(self) -> Profiler {
        let new_frame = Frame {
            tag_trees: self.closed_tags,
            start_time: self.start_time,
            end_time: Instant::now(),
        };

        self.profiler.end_frame(new_frame)
    }

    pub fn ignore_frame(self) -> Profiler {
        self.profiler
    }
}

impl<'f> PushTag for &'f mut OpenFrame {
    type Child = OpenTagTreeRoot<'f>;

    fn push_tag(self, label: &'static str, color: [f32; 4]) -> OpenTagTreeRoot<'f> {
        OpenTagTreeRoot::new(label, color, self)
    }
}

#[must_use]
struct OpenTag {
    label: &'static str,
    color: [f32; 4],
    push_time: Instant,
}

impl OpenTag {
    pub fn new(label: &'static str, color: [f32; 4]) -> OpenTag {
        OpenTag {
            push_time: Instant::now(),
            label,
            color,
        }
    }

    pub fn close(self) -> ClosedTag {
        ClosedTag {
            label: self.label,
            color: self.color,
            push_time: self.push_time,
            pop_time: Instant::now(),
        }
    }
}

#[derive(Clone)]
struct ClosedTag {
    label: &'static str,
    color: [f32; 4],
    push_time: Instant,
    pop_time: Instant,
}

#[must_use]
pub struct OpenTagTreeRoot<'f> {
    frame: &'f mut OpenFrame,
    tag: OpenTag,
    closed_children: Vec<ClosedTagTree>,
}

impl<'f> OpenTagTreeRoot<'f> {
    pub fn new(label: &'static str, color: [f32; 4], frame: &'f mut OpenFrame) -> OpenTagTreeRoot<'f> {
        let tag = OpenTag::new(label, color);

        OpenTagTreeRoot {
            closed_children: Vec::new(),
            tag,
            frame,
        }
    }
}

impl<'f> PopTag for OpenTagTreeRoot<'f> {
    type Into = &'f mut OpenFrame;

    fn pop_tag(self) -> Self::Into {
        let OpenTagTreeRoot {
            mut frame,
            closed_children,
            tag,
        } = self;

        frame.closed_tags.push(ClosedTagTree {
            children: closed_children,
            tag: tag.close(),
        });

        frame
    }
}

#[derive(Clone)]
struct ClosedTagTree {
    tag: ClosedTag,
    children: Vec<ClosedTagTree>,
}
