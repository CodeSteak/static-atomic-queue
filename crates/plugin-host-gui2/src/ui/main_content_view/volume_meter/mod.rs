use audio_garbage_collector::Shared;
use audio_processor_iced_design_system::colors::Colors;
use audio_processor_iced_design_system::spacing::Spacing;
use iced::canvas::{Cursor, Frame, Geometry, Program};
use iced::widget::canvas::Fill;
use iced::{Canvas, Container, Element, Length, Point, Rectangle, Size};
use plugin_host_lib::processors::volume_meter_processor::VolumeMeterProcessorHandle;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct VolumeInfo {
    left: f32,
    right: f32,
    left_peak: f32,
    right_peak: f32,
}

impl Default for VolumeInfo {
    fn default() -> Self {
        Self {
            left: 0.0,
            right: 0.0,
            left_peak: 0.0,
            right_peak: 0.0,
        }
    }
}

impl From<&Option<Shared<VolumeMeterProcessorHandle>>> for VolumeInfo {
    fn from(handle: &Option<Shared<VolumeMeterProcessorHandle>>) -> Self {
        match handle {
            None => VolumeInfo::default(),
            Some(handle) => VolumeInfo {
                left: handle.volume_left.get(),
                left_peak: handle.peak_left.get(),
                right: handle.volume_right.get(),
                right_peak: handle.peak_right.get(),
            },
        }
    }
}

type Message = ();

pub struct VolumeMeter {
    volume_info: VolumeInfo,
}

impl VolumeMeter {
    pub fn new(volume_info: VolumeInfo) -> Self {
        Self { volume_info }
    }

    pub fn view<'a>(self) -> Element<'a, ()> {
        Container::new(
            Canvas::new(VolumeMeterProgram::new(self.volume_info))
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Spacing::medium_spacing())
        .into()
    }
}

struct VolumeMeterProgram {
    volume: VolumeInfo,
}

impl VolumeMeterProgram {
    pub fn new(volume: VolumeInfo) -> Self {
        VolumeMeterProgram { volume }
    }
}

impl Program<Message> for VolumeMeterProgram {
    fn draw(&self, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
        let mut frame = Frame::new(bounds.size());

        let spacing = Spacing::small_spacing() as f32 / 2.;
        let bar_width = bounds.width / 2. - spacing / 2.;
        VolumeMeterProgram::draw_volume_bar(
            &mut frame,
            self.volume.left,
            self.volume.left_peak,
            bar_width,
            0.0,
        );
        VolumeMeterProgram::draw_volume_bar(
            &mut frame,
            self.volume.right,
            self.volume.right_peak,
            bar_width,
            bar_width + spacing,
        );

        vec![frame.into_geometry()]
    }
}

impl VolumeMeterProgram {
    /// Draw a rectangle for volume
    fn draw_volume_bar(
        frame: &mut Frame,
        volume: f32,
        peak_volume: f32,
        bar_width: f32,
        offset_x: f32,
    ) {
        // Maybe don't calculate these things on draw?
        // Also: how to get to the reference power magic nº?
        // let reference_amplitude = 0.1;
        // let volume_db = 20.0 * (volume / reference_amplitude).log10();
        // let peak_volume_db = 20.0 * (peak_volume / reference_amplitude).log10();

        let bar_height = volume * frame.height() * 5.;
        let peak_bar_height = peak_volume * frame.height() * 5.;

        let y_coord = frame.height() - bar_height;
        let peak_y_coord = frame.height() - peak_bar_height;
        // Background
        frame.fill_rectangle(
            Point::new(offset_x, 0.0),
            Size::new(bar_width, frame.height()),
            Fill::from(Colors::background_level0()),
        );
        // Peak Volume
        frame.fill_rectangle(
            Point::new(offset_x, peak_y_coord),
            Size::new(bar_width, peak_bar_height),
            Fill::from(Colors::success().darken(0.4)),
        );
        // RMS Volume
        frame.fill_rectangle(
            Point::new(offset_x, y_coord),
            Size::new(bar_width, bar_height),
            Fill::from(Colors::success()),
        );
    }
}