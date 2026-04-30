use es_fluent::EsFluent;

es_fluent_manager_embedded::define_i18n_module!();

#[derive(Clone, Debug, EsFluent)]
#[fluent(namespace = "date_picker")]
pub(crate) enum DatePickerComponentText {
    LaunchPlaceholder,
}

#[derive(Clone, Debug, EsFluent)]
#[fluent(namespace = "file_picker")]
pub(crate) enum FilePickerComponentText {
    SourcePlaceholder,
    OutputPlaceholder,
    ChooseFiles,
}
