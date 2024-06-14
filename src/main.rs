use daspotwidget::image::get_image;
use daspotwidget::player_ctl::*;
use iced::settings::Settings;
use iced::theme::Theme;
use iced::widget::{
    button, checkbox, column, image::Handle, row, slider, text, vertical_slider, Column, Image,
    Row, Slider, VerticalSlider,
};
use iced::window;
use iced::Alignment;
use iced::Command;
use iced::Font;
use iced::Length;
use iced::Renderer;
use iced::{program, Subscription};
use iced::{time, Point, Size};
use std::fs::File;
use std::io::Read;

#[derive(Clone, Debug, Copy)]
enum Message {
    PlayPause,
    Next,
    Previous,
    PositionChange(u32),
    VolumeChange(f32),
    Stop,
    Shuffle(bool),
    Update,
}
#[derive(Debug)]
struct App {
    volume: f32,
    position: u32,
    shuffle: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            volume: get_volume(),
            position: get_position(),
            shuffle: get_shuffle(),
        }
    }
}

impl App {
    fn view(&self) -> Row<Message> {
        let song_info: Column<Message, Theme, Renderer> = column![
            text(get_title()).size(30),
            text(format!("from \"{}\"", get_artist())),
            text(format!("in album \"{}\"", get_album())),
        ]
        .align_items(Alignment::Start)
        .width(Length::Shrink)
        .padding(5);
        let buttons: Row<Message, Theme, Renderer> = row![
            button(text("󰙣").size(24)).on_press(Message::Previous),
            button(text("󰐎").size(24)).on_press(Message::PlayPause),
            button(text("󰙡").size(24)).on_press(Message::Next),
            button(text("").size(24)).on_press(Message::Stop),
            checkbox("", self.shuffle).on_toggle(Message::Shuffle),
        ]
        .align_items(Alignment::Center)
        .padding(10)
        .spacing(5);
        let song_progress: Slider<u32, Message, Theme> =
            slider(0..=get_length(), self.position, Message::PositionChange).width(200);
        let volume: VerticalSlider<f32, Message, Theme> =
            vertical_slider(0.0..=1.0, self.volume, Message::VolumeChange)
                .height(100)
                .step(0.1);
        let mut row = Row::new();
        let image_path = get_image();
        if image_path.is_some() {
            let mut file = File::open(image_path.unwrap()).unwrap();
            let mut buf: Vec<u8> = vec![];
            let _ = file.read_to_end(&mut buf);
            let handle = Handle::from_bytes(buf);
            let image = Image::new(handle).width(200).height(200);
            row = row.push(image);
        }
        let col = column![song_info, buttons, song_progress]
            .align_items(Alignment::Center)
            .padding(30);
        row.push(col)
            .push(volume)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_items(Alignment::Center)
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::PlayPause => {
                play_pause();
                Command::none()
            }
            Message::Next => {
                next();
                Command::none()
            }
            Message::Previous => {
                previous();
                Command::none()
            }
            Message::PositionChange(pos) => {
                position(pos);
                Command::none()
            }
            Message::VolumeChange(vol) => {
                volume(vol);
                Command::none()
            }
            Message::Stop => {
                stop();
                Command::none()
            }
            Message::Shuffle(shu) => {
                shuffle(shu);
                Command::none()
            }
            Message::Update => {
                self.position = get_position();
                self.volume = get_volume();
                self.shuffle = get_shuffle();
                window::resize(window::Id::MAIN, Size::new(100f32, 100f32))
            }
        }
    }

    fn theme(&self) -> Theme {
        Theme::CatppuccinMacchiato
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(time::Duration::from_millis(1000)).map(|_| Message::Update)
    }
}

fn main() -> iced::Result {
    program("DaSpotWidget", App::update, App::view)
        .theme(App::theme)
        .subscription(App::subscription)
        .settings(Settings {
            window: window::Settings {
                resizable: true,
                ..Default::default()
            },
            default_font: Font::with_name("Hack Nerd Font Mono"),
            ..Default::default()
        })
        .run()
}
