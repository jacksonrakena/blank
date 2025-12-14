use std::fs;
use kdl::{KdlDocument, KdlNode};
use miette::{Diagnostic, NamedSource, SourceOffset};
use errors::{CannotReadTargetManifestError, InvalidTargetError, RuleMissingValidTargetError};
use rules::Rule;

pub mod rules;
pub mod errors;

fn parse_url_string(url: String, src: &NamedSource<String>, target: &KdlNode) -> Result<String, InvalidTargetError> {
    snailquote::unescape(&*url).or_else(|e| Err(InvalidTargetError {
        src: src.clone(),
        error: e,
        broken_target_ref: target.span(),
    }))
}
pub fn parse_doc(src: NamedSource<String>, doc: KdlDocument) -> miette::Result<Vec<Rule>> {
    let mut targets = vec![];
    for node in doc {
        match node.children() {
            Some(children) => {
                let target = children.get("url").ok_or_else(
                    || RuleMissingValidTargetError {
                        src: src.clone(),
                        broken_target_ref: node.span(),
                        position_to_insert_target: SourceOffset::from(node.span().offset()),
                    }
                )?;
                targets.push(Rule {
                    name: node.name().to_string(),
                    url: parse_url_string((&*target.entries().first().ok_or_else(
                        || RuleMissingValidTargetError {
                            src: src.clone(),
                            broken_target_ref: target.span(),
                            position_to_insert_target: SourceOffset::from(target.span().offset()),
                        }
                    )?.value().to_string()).parse()?, &src, &node)?,
                });
            },
            None => {
                let name = node.name();

                let value = node.entries().first().ok_or_else(
                    || RuleMissingValidTargetError {
                        src: src.clone(),
                        broken_target_ref: node.span(),
                        position_to_insert_target: SourceOffset::from(node.span().offset()),
                    }
                )?;
                targets.push(Rule {
                    name: name.to_string(),
                    url: parse_url_string(value.value().to_string(), &src, &node)?,
                });
            }
        }
    }
    Ok(targets)
}

pub fn parse_targets() -> miette::Result<Vec<Rule>> {
    let target_text = fs::read_to_string("targets.kdl").or_else(|_| Err(CannotReadTargetManifestError {}))?;
    let src = NamedSource::new(
        "targets.kdl".to_string(),
        target_text.clone()
    );
    let doc: KdlDocument = KdlDocument::parse(&*target_text)?;

    parse_doc(src, doc)
}

// tests
#[cfg(test)]
mod tests {
    use super::*;

    fn test_helper_parse_doc(text: &str) -> Vec<Rule> {
        let src = NamedSource::new("test_doc".to_string(), text.to_string());
        let doc: KdlDocument = text.parse().expect("Could not parse KDL document");
        parse_doc(src, doc).unwrap()
    }

    #[test]
    fn test_parse_simple_targets() {
        let doc_string = r#"
        target1 "https://example.com/target1"
        target2 "https://example.com/target2"
        "#;
        let targets = test_helper_parse_doc(doc_string);
        assert_eq!(targets.len(), 2);
        assert_eq!(targets[0].name, "target1");
        assert_eq!(targets[0].url, "https://example.com/target1");
        assert_eq!(targets[1].name, "target2");
        assert_eq!(targets[1].url, "https://example.com/target2");
    }

    #[test]
    fn test_parse_explicit_targets() {
        let doc_string = r#"
        target1_exp {
            url "https://example.com/target1_exp"
        }
        target2_exp {
            url "https://example.com/target2_exp"
        }
        "#;
        let targets = test_helper_parse_doc(doc_string);
        assert_eq!(targets.len(), 2);
        assert_eq!(targets[0].name, "target1_exp");
        assert_eq!(targets[0].url, "https://example.com/target1_exp");
        assert_eq!(targets[1].name, "target2_exp");
        assert_eq!(targets[1].url, "https://example.com/target2_exp");
    }
}