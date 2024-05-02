use std::{borrow::Cow, io, path::PathBuf, rc::Rc, sync::Arc};

use iced::{
    executor,
    widget::{column, container, scrollable, slider, text, vertical_slider, Button, Text},
    Application, Command, Theme,
};
use iced::{Element, Length, Sandbox, Settings};

use crate::{
    gui::treeview::TreeView,
    parse_tree::{parallel::parse_tree, Config, Dir, FileError},
};

mod treeview;
mod dir_walk;

pub fn main() -> iced::Result {
    RustDirStat::run(Settings::default())
}

#[derive(Debug, Clone)]
enum Message {
    PickDir,
    DirPicked(Option<PathBuf>),
    DirWalked(Result<(Dir, Vec<FileError>), FileError>),
}

#[derive(Debug)]
enum Page {
    Landing,
    PickingDir,
    Loading(PathBuf),
    Displaying(Dir, Vec<FileError>),
}

struct RustDirStat {
    page: Page,
}

impl Application for RustDirStat {
    type Message = Message;
    type Executor = executor::Default;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (RustDirStat, iced::Command<Message>) {
        (RustDirStat {
            page: Page::Loading("Test".into())
        },
        Command::perform(run_parse_tree("/home/robot_rover/Projects".into()), Message::DirWalked))
    }

    fn title(&self) -> String {
        String::from("RustDirStat - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match (&self.page, message) {
            (Page::Landing | Page::Displaying(_, _), Message::PickDir) => {
                self.page = Page::PickingDir;
                Command::perform(pick_dir(), Message::DirPicked)
            }
            (Page::PickingDir, Message::DirPicked(option)) => {
                if let Some(dir_path) = option {
                    self.page = Page::Loading(dir_path.clone());
                    println!("Dir Picked: {:?}", &dir_path);
                    Command::perform(run_parse_tree(dir_path), Message::DirWalked)
                } else {
                    self.page = Page::Landing;
                    Command::none()
                }
            }
            (Page::Loading(_), Message::DirWalked(result)) => {
                match result {
                    Ok((dir, errors)) => {
                        self.page = Page::Displaying(dir, errors);
                    }
                    Err(err) => {
                        eprintln!("Error walking directory: {:?}", err);
                        self.page = Page::Landing;
                    }
                }
                Command::none()
            }
            (page, message) => {
                eprintln!("Unhandled message: {:?} in page: {:?}", message, page);
                self.page = Page::Landing;
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let status_message: Cow<str> = match &self.page {
            Page::Landing => "Select a directory to scan...".into(),
            Page::PickingDir => "Picking directory...".into(),
            Page::Loading(path) => format!("Reading Subtree of {}", path.display()).into(),
            Page::Displaying(dir, errors) => format!(
                "Finished reading {}, found {} errors",
                dir.get_name(),
                errors.len()
            )
            .into(),
        };
        let display = Text::new(status_message);
        let open_picker = Button::new("Open Folder").on_press(Message::PickDir);
        let content = if let Page::Displaying(dir, errors) = &self.page {
            // let list = dir
            //     .get_dirs()
            //     .iter()
            //     .map(|d| Text::new(d.get_name()).into());
            // let scroll = scrollable(column(list))
            //     .width(Length::Fill)
            //     .height(Length::Fill);
            column![
                container(display).center_x(),
                container(TreeView::new(dir)).center_x(),
                container(open_picker).center_x(),
            ]
        } else {
            column![
                container(display).center_x(),
                container(open_picker).center_x(),
            ]
        };
        container(content.spacing(25))
            .height(Length::Fill)
            .width(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

async fn pick_dir() -> Option<PathBuf> {
    let path = rfd::AsyncFileDialog::new()
        .set_title("Pick a directory to scan...")
        .pick_folder()
        .await
        .map(|handle| handle.path().to_owned());
    path
}

async fn run_parse_tree(path: PathBuf) -> Result<(Dir, Vec<FileError>), FileError> {
    let config = Config {
        same_filesystem: true,
        follow_symlinks: false,
    };
    parse_tree(path.clone(), config)
}
