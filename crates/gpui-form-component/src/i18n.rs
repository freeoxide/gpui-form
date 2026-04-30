use es_fluent::EsFluent;

es_fluent_manager_embedded::define_i18n_module!();

#[derive(Clone, Debug, EsFluent)]
#[fluent(namespace = "date_picker")]
pub(crate) enum DatePickerText {
    SelectDate,
}

#[derive(Clone, Debug, EsFluent)]
#[fluent(namespace = "file_picker")]
pub(crate) enum FilePickerText {
    SelectAFile,
    SelectADirectory,
    SelectAFileOrDirectory,
    SelectFile,
    SelectDirectory,
    SelectFileOrDirectory,
    Browse,
    DialogDropped,
    PathsSelected { count: usize },
}

#[cfg(test)]
mod tests {
    use es_fluent::ToFluentString as _;

    use super::{DatePickerText, FilePickerText};

    #[test]
    fn resolves_runtime_component_messages() {
        es_fluent_manager_embedded::init_with_language(unic_langid::langid!("en"));

        assert_eq!(DatePickerText::SelectDate.to_fluent_string(), "Select date");
        assert_eq!(FilePickerText::Browse.to_fluent_string(), "Browse");
        assert_eq!(
            strip_fluent_isolates(&FilePickerText::PathsSelected { count: 2 }.to_fluent_string()),
            "2 paths selected"
        );
    }

    fn strip_fluent_isolates(value: &str) -> String {
        value.replace(['\u{2068}', '\u{2069}'], "")
    }
}
