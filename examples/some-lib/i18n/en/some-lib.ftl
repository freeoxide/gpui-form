# some-lib Fluent bundle (fallback locale).
#
# This example crate has never carried translations in this repository's
# history (its assets_dir previously pointed at a nonexistent '../i18n',
# which es-fluent treated as unmanaged). The bundle is intentionally empty:
# es-fluent's embedded-manager macro requires the fallback-locale directory
# to exist at compile time; message keys resolve to their identifiers at
# runtime. This crate is ignored by the FTL CI check for the same reason.
