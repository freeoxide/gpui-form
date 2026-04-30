use es_fluent::EsFluent;

es_fluent_manager_embedded::define_i18n_module!();

#[derive(Clone, Debug, EsFluent)]
#[fluent(namespace = "date_picker")]
pub(crate) enum DatePickerStoryText {
    None,
    Title,
    Description,
    InitialEvent,
    DefaultLabel,
    LongLabel,
    FrenchLabel,
    CompactLabel,
    UnknownLabel,
    Changed { picker: String, date: String },
    DefaultFieldLabel,
    DefaultFieldDescription,
    LaunchPlaceholder,
    LongFieldLabel,
    LongFieldDescription,
    FrenchFieldLabel,
    FrenchFieldDescription,
    CompactFieldLabel,
    CompactFieldDescription,
    PanelTitle,
    PanelDescription,
    DefaultValue { value: String },
    LongValue { value: String },
    FrenchValue { value: String },
    CompactValue { value: String },
    LastChange { value: String },
}

#[derive(Clone, Debug, EsFluent)]
#[fluent(namespace = "file_picker")]
pub(crate) enum FilePickerStoryText {
    None,
    Title,
    Description,
    InitialEvent,
    FileLabel,
    DirectoryLabel,
    MultipleLabel,
    UnknownLabel,
    Changed { picker: String, paths: String },
    Cancelled { picker: String },
    Error { picker: String, message: String },
    FileFieldLabel,
    FileFieldDescription,
    SourcePlaceholder,
    DirectoryFieldLabel,
    DirectoryFieldDescription,
    OutputPlaceholder,
    MultipleFieldLabel,
    MultipleFieldDescription,
    ChooseFiles,
    PanelTitle,
    PanelDescription,
    FileValue { value: String },
    DirectoryValue { value: String },
    MultipleValue { value: String },
    LastEvent { value: String },
}

#[derive(Clone, Debug, EsFluent)]
#[fluent(namespace = "infinite_select")]
pub(crate) enum InfiniteSelectStoryText {
    None,
    Title,
    Description,
    PanelTitle,
    PanelDescription,
    CurrentSelection { value: String },
    PathIndices { value: String },
    PathKeys { value: String },
    RebuiltFromPath { value: String },
    RebuiltFromKeys { value: String },
    PreviousKeyPath { value: String },
    LastChangedDepth { value: String },
    DeploymentWeb { region: String },
    DeploymentDesktop { platform: String },
    DeploymentDocs,
    WebRegion { region: String, zone: String },
}

#[cfg(test)]
mod tests {
    use es_fluent::ToFluentString as _;

    use super::{DatePickerStoryText, FilePickerStoryText, InfiniteSelectStoryText};

    #[test]
    fn resolves_story_messages() {
        es_fluent_manager_embedded::init_with_language(unic_langid::langid!("en"));

        assert_eq!(DatePickerStoryText::Title.to_fluent_string(), "Date Picker");
        assert_eq!(FilePickerStoryText::Title.to_fluent_string(), "File Picker");
        assert_eq!(
            InfiniteSelectStoryText::Title.to_fluent_string(),
            "Infinite Select"
        );
    }
}
