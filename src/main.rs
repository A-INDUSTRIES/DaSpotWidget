use daspotwidget::image::get_image;
use daspotwidget::player_ctl::*;

use iced::widget::{
    button, checkbox, column, container, image::Handle, row, slider, text, vertical_slider, Column,
    Container, Image, Row, Slider, VerticalSlider,
};
use iced::{
    alignment::Horizontal, program, settings::Settings, theme::Theme, time, window,
    window::Position::SpecificWith, Alignment, Command, Font, Length, Point, Renderer, Size,
    Subscription,
};

use std::{cmp::max, fs::File, io::Read};

#[cfg(feature = "hyprland")]
use std::process::Command as ProcCommand;

#[derive(Clone, Debug)]
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
    current_width: usize,
    current_song: String,
    image_buffer: Vec<u8>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            volume: get_volume(),
            position: get_position(),
            shuffle: get_shuffle(),
            current_width: 0,
            current_song: "".to_string(),
            image_buffer: Vec::new(),
        }
    }
}

impl App {
    fn view(&self) -> Container<Message> {
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
        .spacing(5)
        .width(Length::Shrink);
        let song_progress: Slider<u32, Message, Theme> =
            slider(0..=get_length(), self.position, Message::PositionChange)
                .width(self.current_width as f32 - 350f32);
        let volume: VerticalSlider<f32, Message, Theme> =
            vertical_slider(0.0..=1.0, self.volume, Message::VolumeChange)
                .height(100)
                .step(0.1);
        let mut row = Row::new();
        if !self.image_buffer.is_empty() {
            let handle = Handle::from_bytes(self.image_buffer.clone());
            let image = Image::new(handle).width(200).height(200);
            row = row.push(container(image).padding(10).align_x(Horizontal::Center));
        }
        let col = column![song_info, buttons, song_progress]
            .align_items(Alignment::Center)
            .padding(30);
        row = row
            .push(col)
            .push(volume)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_items(Alignment::Center);
        container(row).center(Length::Fill)
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
                let new_title = get_title();
                if new_title != self.current_song {
                    let image_path = get_image();
                    if image_path.is_some() {
                        let mut file = File::open(image_path.unwrap()).unwrap();
                        self.image_buffer = Vec::new();
                        let _ = file.read_to_end(&mut self.image_buffer);
                    }
                    self.current_song = new_title;
                    let new_width = compute_size();
                    self.current_width = new_width;
                    resize_and_move(new_width)
                } else {
                    Command::none()
                }
            }
        }
    }

    fn theme(&self) -> Theme {
        Theme::CatppuccinMacchiato
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(time::Duration::from_millis(500)).map(|_| Message::Update)
    }
}

#[cfg(feature = "hyprland")]
fn resize_and_move(width: usize) -> Command<Message> {
    let arg = format!("dispatch resizewindowpixel exact {} 220,title:DaSpotWidget ; dispatch movewindowpixel exact 50% 50,title:DaSpotWidget ; dispatch movewindowpixel -{} 0,title:DaSpotWidget", width, width/2);
    let _ = ProcCommand::new("hyprctl")
        .args(["--batch", arg.as_str()])
        .output();
    Command::none()
}

#[cfg(not(feature = "hyprland"))]
fn resize_and_move(&self, width: usize) -> Command<Message> {
    let resize_command =
        window::resize::<Message>(window::Id::MAIN, Size::new(width as f32, 220f32));
    let move_command =
        window::move_to::<Message>(window::Id::MAIN, Point::new((width / 2) as f32, 50f32));
    Command::batch([resize_command, move_command])
}

fn compute_size() -> usize {
    let title = get_title();
    let artist = format!("from \"{}\"", get_artist());
    let album = format!("in album \"{}\"", get_album());
    let titlesize = title.len() * 19;
    let artistsize = artist.len() * 11;
    let albumsize = album.len() * 11;
    let maxsize = max(max(titlesize, artistsize), albumsize);
    max(maxsize + 300, 530)
}

fn main() -> iced::Result {
    let width = compute_size();
    program("DaSpotWidget", App::update, App::view)
        .theme(App::theme)
        .subscription(App::subscription)
        .settings(Settings {
            window: window::Settings {
                size: Size::new(width as f32, 220f32),
                resizable: true,
                position: SpecificWith(|win, dis| {
                    Point::new(dis.width - (win.width) / 2f32, 50f32)
                }),
                ..Default::default()
            },
            default_font: Font::with_name("Hack Nerd Font Mono"),
            ..Default::default()
        })
        .load(move || resize_and_move(width))
        .run()
}
