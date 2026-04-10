use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Skill Contract v1 rule identifiers (R01..R22).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuleId {
    R01FrontmatterRequired,
    R02NameMatchesDir,
    R03AllowedTools,
    R04Category,
    R05Triggers,
    R06SiblingsExist,
    R07DescriptionLength,
    R08DescriptionFrontTriggers,
    R09DescriptionPerson,
    R10DescriptionUseDoNotUse,
    R11DescriptionMentionsSibling,
    R12SiblingBidirectional,
    R13BodyMaxLines,
    R14BodyHeadings,
    R15ReferencesHasLink,
    R16BodyTimeSensitive,
    R17RuntimeChecklist,
    R18ReferenceToc,
    R19ReferenceNesting,
    R20ClaudeSymlink,
    R21AgentsSymlink,
    R22TriggerCollision,
}

impl RuleId {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::R01FrontmatterRequired => "R01",
            Self::R02NameMatchesDir => "R02",
            Self::R03AllowedTools => "R03",
            Self::R04Category => "R04",
            Self::R05Triggers => "R05",
            Self::R06SiblingsExist => "R06",
            Self::R07DescriptionLength => "R07",
            Self::R08DescriptionFrontTriggers => "R08",
            Self::R09DescriptionPerson => "R09",
            Self::R10DescriptionUseDoNotUse => "R10",
            Self::R11DescriptionMentionsSibling => "R11",
            Self::R12SiblingBidirectional => "R12",
            Self::R13BodyMaxLines => "R13",
            Self::R14BodyHeadings => "R14",
            Self::R15ReferencesHasLink => "R15",
            Self::R16BodyTimeSensitive => "R16",
            Self::R17RuntimeChecklist => "R17",
            Self::R18ReferenceToc => "R18",
            Self::R19ReferenceNesting => "R19",
            Self::R20ClaudeSymlink => "R20",
            Self::R21AgentsSymlink => "R21",
            Self::R22TriggerCollision => "R22",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    pub rule: String,
    pub severity: Severity,
    pub skill: String,
    pub path: PathBuf,
    pub message: String,
}

impl Violation {
    pub fn new(
        rule: RuleId,
        severity: Severity,
        skill: impl Into<String>,
        path: impl AsRef<Path>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            rule: rule.as_str().to_string(),
            severity,
            skill: skill.into(),
            path: path.as_ref().to_path_buf(),
            message: message.into(),
        }
    }
}

/// Parsed YAML frontmatter for a SKILL.md file.
#[derive(Debug, Clone, Default)]
pub struct Frontmatter {
    pub name: Option<String>,
    pub description: Option<String>,
    pub allowed_tools: Option<String>,
    pub user_invocable: Option<bool>,
    pub metadata: Metadata,
    /// Raw YAML for fields linter doesn't model.
    #[allow(dead_code)]
    pub raw_yaml: String,
}

#[derive(Debug, Clone, Default)]
pub struct Metadata {
    pub author: Option<String>,
    pub version: Option<String>,
    pub category: Option<String>,
    pub triggers: Vec<String>,
    pub siblings: Vec<String>,
}

/// A loaded skill directory.
#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub dir: PathBuf,
    pub skill_md_path: PathBuf,
    pub frontmatter: Frontmatter,
    pub body: String,
    pub references: Vec<PathBuf>,
}

impl Skill {
    pub fn body_line_count(&self) -> usize {
        self.body.lines().count()
    }

    pub fn description(&self) -> &str {
        self.frontmatter.description.as_deref().unwrap_or("")
    }

    pub fn triggers(&self) -> &[String] {
        &self.frontmatter.metadata.triggers
    }

    pub fn siblings(&self) -> &[String] {
        &self.frontmatter.metadata.siblings
    }
}

/// Allowed tool tokens for unity-cli skills (Skill Contract v1, S-1.2).
pub const ALLOWED_TOOLS_BASE: &[&str] = &["Bash(unity-cli:*)", "Read", "Grep", "Glob"];
pub const ALLOWED_TOOLS_EDIT: &[&str] = &["Edit", "Write"];
pub const ALLOWED_TOOLS_LEGACY_BASH: &str = "Bash";

pub fn allowed_tool_set(skill_name: &str) -> BTreeSet<&'static str> {
    let mut set: BTreeSet<&'static str> = ALLOWED_TOOLS_BASE.iter().copied().collect();
    if skill_name == "unity-csharp-edit" {
        for token in ALLOWED_TOOLS_EDIT {
            set.insert(token);
        }
    }
    set
}

/// Categories permitted by `metadata.category` (S-1.2).
pub const ALLOWED_CATEGORIES: &[&str] = &[
    "foundation",
    "scenes",
    "assets",
    "code",
    "editor",
    "input",
    "testing",
    "prefabs",
    "ui",
];
