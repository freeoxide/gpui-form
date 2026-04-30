use std::path::{Path, PathBuf};

use es_fluent::ToFluentString as _;
use gpui::{
    App, AppContext as _, Context, Entity, Focusable, IntoElement, ParentElement as _, Render,
    SharedString, Styled as _, Subscription, Window, div,
};
use gpui_component::form::v_form;

use crate::i18n::FilePickerStoryText;
use gpui_form_component::file_picker::{
    FilePicker, FilePickerEvent, FilePickerMode, FilePickerState,
};

use super::common::{story_field, story_panel};

#[gpui_storybook::story]
pub struct FilePickerStory {
    file_picker: Entity<FilePickerState>,
    directory_picker: Entity<FilePickerState>,
    multiple_picker: Entity<FilePickerState>,
    last_event: SharedString,
    _subscriptions: Vec<Subscription>,
}

impl gpui_storybook::Story for FilePickerStory {
    fn title() -> String {
        FilePickerStoryText::Title.to_fluent_string()
    }

    fn description() -> String {
        FilePickerStoryText::Description.to_fluent_string()
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
        cx.new(|cx| Self::new(window, cx))
    }
}

impl Focusable for FilePickerStory {
    fn focus_handle(&self, cx: &App) -> gpui::FocusHandle {
        self.file_picker.read(cx).focus_handle().clone()
    }
}

impl FilePickerStory {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let file_picker = cx.new(|cx| FilePickerState::new(window, cx));
        let directory_picker = cx.new(|cx| {
            let mut state = FilePickerState::new(window, cx);
            state.set_path(PathBuf::from("/tmp/gpui-form"), window, cx);
            state
        });
        let multiple_picker = cx.new(|cx| FilePickerState::new(window, cx));

        let subscriptions = vec![
            cx.subscribe_in(&file_picker, window, Self::on_picker_event),
            cx.subscribe_in(&directory_picker, window, Self::on_picker_event),
            cx.subscribe_in(&multiple_picker, window, Self::on_picker_event),
        ];

        Self {
            file_picker,
            directory_picker,
            multiple_picker,
            last_event: FilePickerStoryText::InitialEvent.to_fluent_string().into(),
            _subscriptions: subscriptions,
        }
    }

    fn picker_label(&self, picker: &Entity<FilePickerState>) -> String {
        if picker == &self.file_picker {
            FilePickerStoryText::FileLabel.to_fluent_string()
        } else if picker == &self.directory_picker {
            FilePickerStoryText::DirectoryLabel.to_fluent_string()
        } else if picker == &self.multiple_picker {
            FilePickerStoryText::MultipleLabel.to_fluent_string()
        } else {
            FilePickerStoryText::UnknownLabel.to_fluent_string()
        }
    }

    fn on_picker_event(
        &mut self,
        picker: &Entity<FilePickerState>,
        event: &FilePickerEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.last_event = match event {
            FilePickerEvent::Change(paths) => FilePickerStoryText::Changed {
                picker: self.picker_label(picker),
                paths: describe_paths(paths),
            }
            .to_fluent_string(),
            FilePickerEvent::Cancel => FilePickerStoryText::Cancelled {
                picker: self.picker_label(picker),
            }
            .to_fluent_string(),
            FilePickerEvent::Error(message) => FilePickerStoryText::Error {
                picker: self.picker_label(picker),
                message: message.to_string(),
            }
            .to_fluent_string(),
        }
        .into();
        cx.notify();
    }
}

impl Render for FilePickerStory {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let file_value = describe_paths(self.file_picker.read(cx).paths());
        let directory_value = describe_paths(self.directory_picker.read(cx).paths());
        let multiple_value = describe_paths(self.multiple_picker.read(cx).paths());

        let form = v_form()
            .child(story_field(
                FilePickerStoryText::FileFieldLabel.to_fluent_string(),
                FilePickerStoryText::FileFieldDescription.to_fluent_string(),
                FilePicker::new(&self.file_picker)
                    .placeholder(FilePickerStoryText::SourcePlaceholder.to_fluent_string())
                    .prompt(FilePickerStoryText::SourcePlaceholder.to_fluent_string())
                    .cleanable(true),
            ))
            .child(story_field(
                FilePickerStoryText::DirectoryFieldLabel.to_fluent_string(),
                FilePickerStoryText::DirectoryFieldDescription.to_fluent_string(),
                FilePicker::new(&self.directory_picker)
                    .mode(FilePickerMode::Directory)
                    .placeholder(FilePickerStoryText::OutputPlaceholder.to_fluent_string())
                    .prompt(FilePickerStoryText::OutputPlaceholder.to_fluent_string())
                    .cleanable(true),
            ))
            .child(story_field(
                FilePickerStoryText::MultipleFieldLabel.to_fluent_string(),
                FilePickerStoryText::MultipleFieldDescription.to_fluent_string(),
                FilePicker::new(&self.multiple_picker)
                    .multiple(true)
                    .browse_label(FilePickerStoryText::ChooseFiles.to_fluent_string())
                    .cleanable(true),
            ));

        story_panel(
            FilePickerStoryText::PanelTitle.to_fluent_string(),
            FilePickerStoryText::PanelDescription.to_fluent_string(),
            div().flex().flex_col().gap_4().child(form).child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .mt_2()
                    .text_sm()
                    .child(FilePickerStoryText::FileValue { value: file_value }.to_fluent_string())
                    .child(
                        FilePickerStoryText::DirectoryValue {
                            value: directory_value,
                        }
                        .to_fluent_string(),
                    )
                    .child(
                        FilePickerStoryText::MultipleValue {
                            value: multiple_value,
                        }
                        .to_fluent_string(),
                    )
                    .child(
                        FilePickerStoryText::LastEvent {
                            value: self.last_event.to_string(),
                        }
                        .to_fluent_string(),
                    ),
            ),
        )
    }
}

fn describe_paths(paths: &[PathBuf]) -> String {
    match paths {
        [] => FilePickerStoryText::None.to_fluent_string(),
        [path] => display_path(path),
        _ => paths
            .iter()
            .map(|path| display_path(path))
            .collect::<Vec<_>>()
            .join(", "),
    }
}

fn display_path(path: &Path) -> String {
    path.display().to_string()
}
