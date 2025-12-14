use std::collections::HashMap;
use std::fs;
use kdl::{KdlDocument, KdlEntry, KdlNode, KdlValue, NodeKey};
use miette::{IntoDiagnostic, NamedSource, SourceOffset};
use errors::{CannotReadTargetManifestError, InvalidTargetError, RuleMissingValidTargetError};
use rules::Rule;
use crate::errors::ExpectedStringError;
use crate::rules::RuleOptions;

pub mod rules;
pub mod errors;

struct ParseContext {
    source: NamedSource<String>,
    document: KdlDocument,
}

impl ParseContext {
    fn try_parse(&self) -> miette::Result<HashMap<String, Rule>> {
        let mut targets = HashMap::new();
        for node in self.document.nodes() {
            let rule = self.try_parse_rule(node)?;
            targets.insert(rule.name.clone(), rule);
        }
        Ok(targets)
    }

    fn parse_string(&self, target: &KdlValue, parent: &KdlNode) -> miette::Result<String> {
        let str = target
            .as_string()
            .ok_or(ExpectedStringError {
                src: self.source.clone(),
                reference: parent.span()
            })
            .into_diagnostic()?;

        snailquote::unescape(str).or_else(|e| Err(InvalidTargetError {
            src: self.source.clone(),
            error: e,
            broken_target_ref: parent.span(),
        })).into_diagnostic()
    }

    fn try_parse_opts(&self, node: &KdlNode) -> miette::Result<RuleOptions> {
        let mut opts = RuleOptions::default();

        let entries = node.entries();

        if let Some(desc) = entries.iter()
            .find(|e|e.name().is_some() && e.name().unwrap().to_string().eq("description")) {
            opts.description = Some(self.parse_string(desc.value(), node)?);
        }

        Ok(opts)
    }
    fn try_parse_rule(&self, node: &KdlNode) -> miette::Result<Rule> {
        match node.children() {
            Some(children) => {
                let target = children.get("target").ok_or_else(
                    || RuleMissingValidTargetError {
                        src: self.source.clone(),
                        broken_target_ref: node.span(),
                        position_to_insert_target: SourceOffset::from(node.span().offset()),
                    }
                )?;
                Ok(Rule {
                    name: node.name().to_string(),
                    url: self.parse_string(target.entries().first().ok_or_else(
                        || RuleMissingValidTargetError {
                            src: self.source.clone(),
                            broken_target_ref: target.span(),
                            position_to_insert_target: SourceOffset::from(target.span().offset()),
                        }
                    )?.value(), &node)?,
                    opts: RuleOptions::default(),
                })
            },
            None => {
                let name = node.name();
                let entries = node.entries();
                if entries.len() == 0 {
                    return Err(RuleMissingValidTargetError {
                        src: self.source.clone(),
                        broken_target_ref: node.span(),
                        position_to_insert_target: SourceOffset::from(node.span().offset()),
                    })?
                }

                let target = node.entry(NodeKey::Index(0)).ok_or_else(|| RuleMissingValidTargetError {
                    src: self.source.clone(),
                    broken_target_ref: node.span(),
                    position_to_insert_target: SourceOffset::from(node.span().offset()),
                })?;

                Ok(Rule {
                    name: name.to_string(),
                    url: self.parse_string(target.value(), node)?,
                    opts: self.try_parse_opts(node)?
                })
            }
        }
    }
}

pub fn parse_targets() -> miette::Result<HashMap<String, Rule>> {
    let target_text = fs::read_to_string("targets.kdl").or_else(|_| Err(CannotReadTargetManifestError {}))?;
    let src = NamedSource::new(
        "targets.kdl".to_string(),
        target_text.clone()
    );
    let doc: KdlDocument = KdlDocument::parse(&*target_text)?;

    ParseContext { source: src, document: doc }.try_parse()
}

pub fn parse_doc(source: NamedSource<String>, doc: KdlDocument) -> miette::Result<HashMap<String, Rule>> {
    ParseContext { source: source, document: doc }.try_parse()
}

// tests
#[cfg(test)]
mod tests {
    use miette::Error;
    use super::*;

    fn test_helper_parse_doc(text: &str) -> Result<HashMap<String, Rule>, Error> {
        let src = NamedSource::new("test_doc".to_string(), text.to_string());
        let doc: KdlDocument = text.parse().expect("Could not parse KDL document");
        parse_doc(src, doc)
    }

    #[test]
    fn test_parse_simple_targets() {
        let doc_string = r#"
        target1 "https://example.com/target1"
        target2 "https://example.com/target2"
        "#;
        let targets = test_helper_parse_doc(doc_string).unwrap();
        assert_eq!(targets.len(), 2);
        assert_eq!(targets.get("target1").unwrap().url, "https://example.com/target1");
        assert_eq!(targets.get("target2").unwrap().url, "https://example.com/target2");
    }

    #[test]
    fn test_parse_explicit_targets() {
        let doc_string = r#"
        target1_exp {
            target "https://example.com/target1_exp"
        }
        target2_exp {
            target "https://example.com/target2_exp"
        }
        "#;
        let targets = test_helper_parse_doc(doc_string).unwrap();
        assert_eq!(targets.len(), 2);
        assert_eq!(targets.get("target1_exp").unwrap().url, "https://example.com/target1_exp");
        assert_eq!(targets.get("target2_exp").unwrap().url, "https://example.com/target2_exp");
    }

    #[test]
        fn parse_fails_on_missing_target_url() {
            let doc_string = r#"
            target1_exp {
            }
            "#;
            let result = test_helper_parse_doc(doc_string);
            assert!(result.is_err());
        }

        #[test]
        fn parse_handles_empty_document() {
            let doc_string = r#""#;
            let targets = test_helper_parse_doc(doc_string).unwrap();
            assert!(targets.is_empty());
        }

        #[test]
        fn parse_handles_duplicate_target_names() {
            let doc_string = r#"
            target1 "https://example.com/target1"
            target1 "https://example.com/target1_duplicate"
            "#;
            let targets = test_helper_parse_doc(doc_string).unwrap();
            assert_eq!(targets.len(), 1);
            assert_eq!(targets.get("target1").unwrap().url, "https://example.com/target1_duplicate");
        }

        #[test]
        fn parse_doesnt_destroy_html_chars() {
            let doc_string = r#"
            target1 "https://example.com/target1%20with%20space"
            "#;
            let targets = test_helper_parse_doc(doc_string).unwrap();
            assert_eq!(targets.len(), 1);
            assert_eq!(targets.get("target1").unwrap().url, "https://example.com/target1%20with%20space");
        }
}