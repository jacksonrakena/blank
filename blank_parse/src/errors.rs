use thiserror::Error;
use miette::{Diagnostic, NamedSource, SourceOffset, SourceSpan};

#[derive(Error, Debug, Diagnostic)]
#[error("rule missing valid target")]
#[diagnostic(
    code("rule_missing_valid_target_error")
)]
pub struct RuleMissingValidTargetError {
    #[source_code]
    pub src: NamedSource<String>,

    #[label("broken rule defined here")]
    pub broken_target_ref: SourceSpan,

    #[label("define a URL or other target in this rule")]
    pub position_to_insert_target: SourceOffset,
}

#[derive(Error, Debug, Diagnostic)]
#[error("cannot read rule manifest")]
#[diagnostic(
    code("cannot_read_target_manifest_error"),
    help("ensure that the 'targets.kdl' file exists and is readable")
)]
pub struct CannotReadTargetManifestError;

#[derive(Error, Debug, Diagnostic)]
#[error("cannot parse target")]
#[diagnostic(
    code("cannot_parse_target_error"),
)]
pub struct InvalidTargetError {
    #[source_code]
    pub src: NamedSource<String>,

    #[source]
    pub error: snailquote::UnescapeError,

    #[label(primary, "invalid target defined here")]
    pub broken_target_ref: SourceSpan,
}