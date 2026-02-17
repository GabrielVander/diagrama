use pest::{
    Parser,
    error::Error,
    iterators::{Pair, Pairs},
};

use crate::infra::pest::plantuml_pest_parser::{PlantumlPestParser, Rule};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PlantUmlAst {
    pub header: Option<UmlHeader>,
    pub statements: Vec<UmlStatement>,
}

impl PlantUmlAst {
    pub fn from_raw(input: &str) -> Result<PlantUmlAst, Error<Rule>> {
        let mut file_pairs: Pairs<Rule> = PlantumlPestParser::parse(Rule::file, input)?;

        let mut statements: Vec<UmlStatement> = Vec::new();

        if let Some(file_pair) = file_pairs.next() {
            Self::process_file_pairs(file_pair, &mut statements);
        }

        // We currently do not parse Title or Direction, so Header defaults to None
        let header: Option<UmlHeader> = None;

        Ok(PlantUmlAst { header, statements })
    }

    fn process_file_pairs(file_pair: Pair<Rule>, statements: &mut Vec<UmlStatement>) {
        for pair in file_pair.into_inner() {
            match pair.as_rule() {
                Rule::statement => Self::process_statement(pair, statements),
                Rule::EOI | Rule::start_uml | Rule::end_uml => {} // Safely ignore
                _ => unreachable!("Unexpected rule at file level: {:?}", pair.as_rule()),
            }
        }
    }

    fn process_statement(pair: Pair<Rule>, statements: &mut Vec<UmlStatement>) {
        let stmt_pair: Pair<Rule> = pair.into_inner().next().unwrap();

        match stmt_pair.as_rule() {
            // Safely ignore these if they still exist in the pest grammar
            Rule::skinparam | Rule::hide_show => {}
            _ => statements.push(Self::build_statement(stmt_pair)),
        }
    }

    fn build_statement(pair: Pair<Rule>) -> UmlStatement {
        match pair.as_rule() {
            Rule::class_def => UmlStatement::Element(Self::build_element(pair)),
            Rule::relation_def => UmlStatement::Relation(Self::build_relation(pair)),
            Rule::package_def => UmlStatement::Package(Self::build_package(pair)),
            Rule::note_def => UmlStatement::Note(Self::build_note(pair)),
            _ => unreachable!("Unexpected statement rule: {:?}", pair.as_rule()),
        }
    }

    fn build_element(pair: Pair<Rule>) -> UmlElement {
        let mut inner: Pairs<Rule> = pair.into_inner();

        let kind: UmlElementKind = Self::map_element_kind(inner.next().unwrap().as_str());
        let (id, display_name): (UmlId, Option<String>) =
            Self::build_identifier(inner.next().unwrap());

        let mut element: UmlElement = UmlElement {
            kind,
            id,
            display_name,
            alias: None,
            stereotype: None,
            members: Vec::new(),
            modifiers: Vec::new(),
        };

        for component in inner {
            Self::apply_element_component(&mut element, component);
        }

        element
    }

    fn apply_element_component(element: &mut UmlElement, component: Pair<Rule>) {
        match component.as_rule() {
            Rule::alias => {
                element.alias = Some(component.into_inner().next().unwrap().as_str().to_string());
            }
            Rule::stereotype => {
                let name: String = component
                    .as_str()
                    .trim_matches(|c| c == '<' || c == '>')
                    .to_string();
                element.stereotype = Some(UmlStereotype { name });
            }
            Rule::body_block => {
                for member_pair in component.into_inner() {
                    if member_pair.as_rule() == Rule::member {
                        let line: &str = member_pair.into_inner().next().unwrap().as_str();
                        element.members.push(Self::parse_member_line(line));
                    }
                }
            }
            Rule::empty_decl => {}
            _ => {}
        }
    }

    fn map_element_kind(kind_str: &str) -> UmlElementKind {
        match kind_str {
            "class" => UmlElementKind::Class,
            "interface" => UmlElementKind::Interface,
            "abstract class" => UmlElementKind::AbstractClass,
            "enum" => UmlElementKind::Enum,
            "component" => UmlElementKind::Component,
            "actor" => UmlElementKind::Actor,
            "database" => UmlElementKind::Database,
            _ => unreachable!("Unknown element kind: {}", kind_str),
        }
    }

    fn build_relation(pair: Pair<Rule>) -> UmlRelation {
        let mut inner: Pairs<Rule> = pair.into_inner();

        let (from_id, _): (UmlId, Option<String>) = Self::build_identifier(inner.next().unwrap());
        let arrow: UmlArrow = Self::build_arrow(inner.next().unwrap());
        let (to_id, _): (UmlId, Option<String>) = Self::build_identifier(inner.next().unwrap());

        let label: Option<String> = inner
            .next()
            .map(|p: Pair<Rule>| p.as_str().trim_start_matches(':').trim().to_string());

        UmlRelation {
            from: from_id,
            to: to_id,
            arrow,
            label,
        }
    }

    fn build_arrow(pair: Pair<Rule>) -> UmlArrow {
        let inner: Pairs<Rule> = pair.into_inner();
        let mut left: UmlArrowEnd = UmlArrowEnd::None;
        let mut right: UmlArrowEnd = UmlArrowEnd::None;
        let mut line: UmlLineStyle = UmlLineStyle::Solid;

        for component in inner {
            match component.as_rule() {
                Rule::arrow_head_left => left = Self::map_arrow_head(component.as_str()),
                Rule::arrow_head_right => right = Self::map_arrow_head(component.as_str()),
                Rule::line_style => {
                    line = if component.as_str().contains('.') {
                        UmlLineStyle::Dotted
                    } else {
                        UmlLineStyle::Solid
                    };
                }
                _ => {}
            }
        }

        Self::adjust_arrow_for_dependencies(&mut left, &mut right, &line);

        UmlArrow { line, left, right }
    }

    fn adjust_arrow_for_dependencies(
        left: &mut UmlArrowEnd,
        right: &mut UmlArrowEnd,
        line: &UmlLineStyle,
    ) {
        if *line == UmlLineStyle::Dotted {
            if *left == UmlArrowEnd::Association {
                *left = UmlArrowEnd::Dependency;
            }
            if *right == UmlArrowEnd::Association {
                *right = UmlArrowEnd::Dependency;
            }
        }
    }

    fn map_arrow_head(s: &str) -> UmlArrowEnd {
        match s {
            "<|" | "|>" => UmlArrowEnd::Inheritance,
            "*" => UmlArrowEnd::Composition,
            "o" => UmlArrowEnd::Aggregation,
            "<" | ">" => UmlArrowEnd::Association,
            _ => UmlArrowEnd::None,
        }
    }

    fn build_package(pair: Pair<Rule>) -> UmlPackage {
        let mut inner: Pairs<Rule> = pair.into_inner();

        let kind: UmlPackageKind = Self::map_package_kind(inner.next().unwrap().as_str());
        let (id, display_name): (UmlId, Option<String>) =
            Self::build_identifier(inner.next().unwrap());

        let mut children: Vec<UmlStatement> = Vec::new();
        for component in inner {
            if component.as_rule() == Rule::statement {
                children.push(Self::build_statement(
                    component.into_inner().next().unwrap(),
                ));
            }
        }

        UmlPackage {
            kind,
            id,
            display_name,
            children,
        }
    }

    fn map_package_kind(kind_str: &str) -> UmlPackageKind {
        match kind_str {
            "package" => UmlPackageKind::Package,
            "namespace" => UmlPackageKind::Namespace,
            "node" => UmlPackageKind::Node,
            "folder" => UmlPackageKind::Folder,
            "rectangle" => UmlPackageKind::Rectangle,
            "frame" => UmlPackageKind::Frame,
            _ => unreachable!("Unknown package kind: {}", kind_str),
        }
    }

    fn build_note(pair: Pair<Rule>) -> UmlNote {
        let mut inner: Pairs<Rule> = pair.into_inner();
        let first: Pair<Rule> = inner.next().unwrap();

        if first.as_rule() == Rule::position {
            Self::build_positional_note(first, inner)
        } else {
            Self::build_floating_note(first, inner)
        }
    }

    fn build_positional_note(position_pair: Pair<Rule>, mut remaining: Pairs<Rule>) -> UmlNote {
        let position: UmlNotePosition = Self::map_note_position(position_pair.as_str());
        let target_id: String = remaining.next().unwrap().as_str().to_string();
        let text: String = remaining.next().unwrap().as_str().to_string();

        UmlNote {
            text,
            position,
            target: Some(UmlId(target_id)),
        }
    }

    fn build_floating_note(text_pair: Pair<Rule>, mut remaining: Pairs<Rule>) -> UmlNote {
        let text: String = text_pair.as_str().to_string();
        let alias: String = remaining.next().unwrap().as_str().to_string();

        UmlNote {
            text,
            position: UmlNotePosition::Floating,
            target: Some(UmlId(alias)),
        }
    }

    fn map_note_position(pos_str: &str) -> UmlNotePosition {
        match pos_str {
            "right" => UmlNotePosition::Right,
            "left" => UmlNotePosition::Left,
            "top" => UmlNotePosition::Top,
            "bottom" => UmlNotePosition::Bottom,
            "over" => UmlNotePosition::Over,
            _ => UmlNotePosition::Floating,
        }
    }

    fn build_identifier(pair: Pair<Rule>) -> (UmlId, Option<String>) {
        let text: &str = pair.as_str();

        if text.starts_with('"') && text.ends_with('"') {
            let inner_str: String = text[1..text.len() - 1].to_string();
            (UmlId(inner_str.clone()), Some(inner_str))
        } else {
            (UmlId(text.to_string()), None)
        }
    }

    fn parse_member_line(line: &str) -> UmlMember {
        let trimmed: &str = line.trim();

        if trimmed.is_empty() {
            return UmlMember::Raw(trimmed.to_string());
        }

        let (visibility, rest): (Option<Visibility>, &str) = Self::extract_visibility(trimmed);
        let signature: &str = rest.trim();

        if signature.contains('(') && signature.contains(')') {
            Self::parse_method(visibility, signature)
        } else {
            Self::parse_field(visibility, signature)
        }
    }

    fn extract_visibility(trimmed_line: &str) -> (Option<Visibility>, &str) {
        match trimmed_line.chars().next().unwrap() {
            '+' => (Some(Visibility::Public), &trimmed_line[1..]),
            '-' => (Some(Visibility::Private), &trimmed_line[1..]),
            '#' => (Some(Visibility::Protected), &trimmed_line[1..]),
            '~' => (Some(Visibility::Package), &trimmed_line[1..]),
            _ => (None, trimmed_line),
        }
    }

    fn parse_method(visibility: Option<Visibility>, signature: &str) -> UmlMember {
        let paren_start: usize = signature.find('(').unwrap();
        let paren_end: usize = signature.rfind(')').unwrap();

        let name: String = signature[..paren_start].trim().to_string();
        let params_str: &str = signature[paren_start + 1..paren_end].trim();

        let parameters: Vec<UmlParameter> = Self::parse_parameters(params_str);
        let return_type: Option<String> = Self::parse_return_type(signature, paren_end);

        UmlMember::Method(UmlMethod {
            visibility,
            name,
            parameters,
            return_type,
        })
    }

    fn parse_parameters(params_str: &str) -> Vec<UmlParameter> {
        if params_str.is_empty() {
            return Vec::new();
        }

        params_str
            .split(',')
            .map(|p: &str| {
                let p_trimmed: &str = p.trim();
                if let Some((pname, ptype)) = p_trimmed.split_once(':') {
                    UmlParameter {
                        name: pname.trim().to_string(),
                        param_type: Some(ptype.trim().to_string()),
                    }
                } else {
                    UmlParameter {
                        name: p_trimmed.to_string(),
                        param_type: None,
                    }
                }
            })
            .collect()
    }

    fn parse_return_type(signature: &str, paren_end: usize) -> Option<String> {
        if signature.len() > paren_end + 1 {
            let ret_part: &str = signature[paren_end + 1..].trim();
            ret_part.strip_prefix(':').map(|ret| ret.trim().to_string())
        } else {
            None
        }
    }

    fn parse_field(visibility: Option<Visibility>, signature: &str) -> UmlMember {
        if let Some((name, ftype)) = signature.split_once(':') {
            UmlMember::Field(UmlField {
                visibility,
                name: name.trim().to_string(),
                field_type: Some(ftype.trim().to_string()),
            })
        } else {
            UmlMember::Field(UmlField {
                visibility,
                name: signature.to_string(),
                field_type: None,
            })
        }
    }
}

// ---------------------------------------------------------
// Structs and Enums
// ---------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct UmlHeader {
    pub title: Option<String>,
    pub direction: Option<LayoutDirection>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum UmlStatement {
    Element(UmlElement),
    Relation(UmlRelation),
    Package(UmlPackage),
    Note(UmlNote),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct UmlElement {
    pub kind: UmlElementKind,
    pub id: UmlId,
    pub display_name: Option<String>,
    pub alias: Option<String>,
    pub stereotype: Option<UmlStereotype>,
    pub members: Vec<UmlMember>,
    pub modifiers: Vec<UmlModifier>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum UmlElementKind {
    Class,
    Interface,
    AbstractClass,
    Enum,
    Component,
    Actor,
    Database,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct UmlId(pub String);

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum UmlMember {
    Field(UmlField),
    Method(UmlMethod),
    Raw(String),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct UmlField {
    pub visibility: Option<Visibility>,
    pub name: String,
    pub field_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct UmlMethod {
    pub visibility: Option<Visibility>,
    pub name: String,
    pub parameters: Vec<UmlParameter>,
    pub return_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Visibility {
    Public,
    Private,
    Protected,
    Package,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct UmlRelation {
    pub from: UmlId,
    pub to: UmlId,
    pub arrow: UmlArrow,
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct UmlArrow {
    pub line: UmlLineStyle,
    pub left: UmlArrowEnd,
    pub right: UmlArrowEnd,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum UmlLineStyle {
    Solid,
    Dotted,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum UmlArrowEnd {
    None,
    Association,
    Inheritance,
    Composition,
    Aggregation,
    Dependency,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum UmlPackageKind {
    Package,
    Namespace,
    Node,
    Folder,
    Rectangle,
    Frame,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct UmlNote {
    pub text: String,
    pub position: UmlNotePosition,
    pub target: Option<UmlId>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum UmlNotePosition {
    Left,
    Right,
    Top,
    Bottom,
    Over,
    Floating,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum LayoutDirection {
    LeftToRight,
    TopToBottom,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct UmlPackage {
    pub kind: UmlPackageKind,
    pub id: UmlId,
    pub display_name: Option<String>,
    pub children: Vec<UmlStatement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UmlStereotype {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum UmlModifier {
    Abstract,
    Static,
    Final,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UmlParameter {
    pub name: String,
    pub param_type: Option<String>,
}

// ---------------------------------------------------------
// Unit Tests
// ---------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_empty_file() {
        let input: &str = "@startuml\n@enduml";
        let ast: PlantUmlAst = PlantUmlAst::from_raw(input).unwrap();

        assert!(ast.statements.is_empty());
        assert!(ast.header.is_none());
    }

    #[test]
    fn test_parse_simple_class() {
        let input: &str = "@startuml\nclass User\n@enduml";
        let ast: PlantUmlAst = PlantUmlAst::from_raw(input).unwrap();

        assert_eq!(ast.statements.len(), 1);
        if let UmlStatement::Element(elem) = &ast.statements[0] {
            assert_eq!(elem.kind, UmlElementKind::Class);
            assert_eq!(elem.id, UmlId("User".to_string()));
            assert!(elem.members.is_empty());
        } else {
            panic!("Expected Element");
        }
    }

    #[test]
    fn test_parse_interface_and_enum() {
        let input: &str = "@startuml\ninterface Repository\nenum Status\n@enduml";
        let ast: PlantUmlAst = PlantUmlAst::from_raw(input).unwrap();

        assert_eq!(ast.statements.len(), 2);

        if let UmlStatement::Element(elem) = &ast.statements[0] {
            assert_eq!(elem.kind, UmlElementKind::Interface);
            assert_eq!(elem.id, UmlId("Repository".to_string()));
        } else {
            panic!("Expected Interface");
        }

        if let UmlStatement::Element(elem) = &ast.statements[1] {
            assert_eq!(elem.kind, UmlElementKind::Enum);
            assert_eq!(elem.id, UmlId("Status".to_string()));
        } else {
            panic!("Expected Enum");
        }
    }

    #[test]
    fn test_parse_class_with_members_and_visibility() {
        let input: &str = r#"
        @startuml
        class User {
            -id: Int
            #password_hash: String
            ~session_token: String
            +getName(includeLastName: Boolean): String
        }
        @enduml
        "#;
        let ast: PlantUmlAst = PlantUmlAst::from_raw(input).unwrap();

        if let UmlStatement::Element(elem) = &ast.statements[0] {
            assert_eq!(elem.members.len(), 4);

            if let UmlMember::Field(f) = &elem.members[0] {
                assert_eq!(f.visibility, Some(Visibility::Private));
                assert_eq!(f.name, "id");
            } else {
                panic!("Expected Field");
            }

            if let UmlMember::Field(f) = &elem.members[1] {
                assert_eq!(f.visibility, Some(Visibility::Protected));
            } else {
                panic!("Expected Field");
            }

            if let UmlMember::Field(f) = &elem.members[2] {
                assert_eq!(f.visibility, Some(Visibility::Package));
            } else {
                panic!("Expected Field");
            }

            if let UmlMember::Method(m) = &elem.members[3] {
                assert_eq!(m.name, "getName");
                assert_eq!(m.return_type.as_deref(), Some("String"));
                assert_eq!(m.visibility, Some(Visibility::Public));
                assert_eq!(m.parameters.len(), 1);
                assert_eq!(m.parameters[0].name, "includeLastName");
                assert_eq!(m.parameters[0].param_type.as_deref(), Some("Boolean"));
            } else {
                panic!("Expected Method");
            }
        }
    }

    #[test]
    fn test_parse_relations_and_dependencies() {
        // Composition (solid) and Dependency (dotted association)
        let input: &str = "@startuml\nUser *-- Profile : has\nAuth ..> User\n@enduml";
        let ast: PlantUmlAst = PlantUmlAst::from_raw(input).unwrap();

        assert_eq!(2, ast.statements.len());

        // Composition
        assert_eq!(
            ast.statements[0],
            UmlStatement::Relation(UmlRelation {
                from: UmlId("User".to_string()),
                to: UmlId("Profile".to_string()),
                arrow: UmlArrow {
                    line: UmlLineStyle::Solid,
                    left: UmlArrowEnd::Composition,
                    right: UmlArrowEnd::None
                },
                label: Some("has".to_owned())
            })
        );

        // Dependency
        assert_eq!(
            ast.statements[1],
            UmlStatement::Relation(UmlRelation {
                from: UmlId("Auth".to_string()),
                to: UmlId("User".to_string()),
                arrow: UmlArrow {
                    line: UmlLineStyle::Dotted,
                    left: UmlArrowEnd::None,
                    right: UmlArrowEnd::Dependency
                },
                label: None
            })
        );
    }

    #[test]
    fn test_parse_package() {
        let input: &str = "package \"Auth Module\" {\n class User \n}";
        let ast: PlantUmlAst = PlantUmlAst::from_raw(input).unwrap();

        if let UmlStatement::Package(pkg) = &ast.statements[0] {
            assert_eq!(pkg.kind, UmlPackageKind::Package);
            assert_eq!(pkg.id, UmlId("Auth Module".to_string()));
            assert_eq!(pkg.display_name.as_deref(), Some("Auth Module"));
            assert_eq!(pkg.children.len(), 1);
        } else {
            panic!("Expected Package");
        }
    }

    #[test]
    fn test_parse_positional_note() {
        let input: &str = "note right of User: This is a note";
        let ast: PlantUmlAst = PlantUmlAst::from_raw(input).unwrap();

        assert_eq!(
            ast.statements[0],
            UmlStatement::Note(UmlNote {
                position: UmlNotePosition::Right,
                target: Some(UmlId("User".to_string())),
                text: "This is a note".to_string(),
            })
        );
    }

    #[test]
    fn test_parse_floating_note() {
        let input: &str = "note \"Floating text\" as N1";
        let ast: PlantUmlAst = PlantUmlAst::from_raw(input).unwrap();

        assert_eq!(
            ast.statements[0],
            UmlStatement::Note(UmlNote {
                position: UmlNotePosition::Floating,
                target: Some(UmlId("N1".to_string())),
                text: "Floating text".to_string(),
            })
        );
    }
}
