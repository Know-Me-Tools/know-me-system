//! # knowme_hands — the Hands system (FUNC-SPEC §06)
//!
//! A Hand is an autonomous agent that works *for* the user, as opposed to a
//! chat agent the user works *with*. It runs on a schedule, executes within a
//! Cedar policy, and writes its outputs to the user's memory regardless of
//! whether the user is watching.
//!
//! ## What is real in this skeleton
//! The domain model: templates, the three-step creation flow (Identity →
//! Configuration → Deployment), schedules, run records, and deployment
//! targets. These types are the contract the desktop `features/hands` UI and
//! the mobile Hands feature serialize against.
//!
//! ## What is stubbed
//! Execution. The scheduler lives in UAR (`gen_ui_agent` owns the PMPO loop);
//! wiring the `HandDefinition` into a scheduled UAR session, enforcing the
//! Cedar policy per skill invocation, and appending to the audit log are
//! tracked in `docs/implementation-roadmap.md` (M1).
//!
//! Spec anchors: FUNC-SPEC §06 (Hands System), §09 (first-use consent),
//! UX-MOOD Screens 03–04.

use serde::{Deserialize, Serialize};

/// The six built-in template archetypes (FUNC-SPEC §06, "The six built-in templates").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HandTemplate {
    /// Watches a topic. Writes a daily digest with sources. `web-fetch · wikipedia`
    Researcher,
    /// Triages messages. Surfaces what needs attention. `email-imap`
    InboxTriage,
    /// Watches a repo. Summarizes diffs, flags risky changes. `filesystem · shell-exec`
    CodeWatcher,
    /// Tracks tickers. Daily delta summary with context. `web-fetch`
    MarketPulse,
    /// Asks a check-in question. Builds private memory. `surreal-memory`
    DailyJournal,
    /// Captures URLs, summarizes, files them by topic. `web-fetch · memory`
    ReadingList,
}

impl HandTemplate {
    /// Default cadence per the spec table.
    pub fn default_schedule(&self) -> HandSchedule {
        match self {
            Self::Researcher => HandSchedule::Cron("0 9 * * *".into()),
            Self::InboxTriage => HandSchedule::Cron("0 */2 * * *".into()),
            Self::CodeWatcher => HandSchedule::Event("on-commit".into()),
            Self::MarketPulse => HandSchedule::Cron("0 17 * * *".into()),
            Self::DailyJournal => HandSchedule::Cron("0 20 * * *".into()),
            Self::ReadingList => HandSchedule::Event("on-capture".into()),
        }
    }

    /// Skills the template requires, granted per-Hand and per-skill — never blanket.
    pub fn required_skills(&self) -> &'static [&'static str] {
        match self {
            Self::Researcher => &["web-fetch", "wikipedia"],
            Self::InboxTriage => &["email-imap"],
            Self::CodeWatcher => &["filesystem", "shell-exec"],
            Self::MarketPulse => &["web-fetch"],
            Self::DailyJournal => &["surreal-memory"],
            Self::ReadingList => &["web-fetch", "surreal-memory"],
        }
    }
}

/// Schedule by cron, event, or manual trigger (FUNC-SPEC §06 key features).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "kind", content = "value")]
pub enum HandSchedule {
    /// Standard 5-field cron expression, evaluated by the UAR scheduler.
    Cron(String),
    /// Named event trigger (e.g. "on-commit", "on-capture").
    Event(String),
    /// Only runs when the user presses "run now".
    Manual,
}

/// Where a Hand executes (creation Step 3 · Deployment).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "kind")]
pub enum DeploymentTarget {
    /// This device — local, always available, battery aware.
    Local,
    /// A connected BossFang instance, addressed by its instance id.
    BossFang { instance_id: String },
}

/// Step 1 of 3 — Identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandIdentity {
    pub name: String,
    pub description: String,
    pub category: String,
}

/// Step 2 of 3 — Configuration. Model, schedule, and the explicit skill grants.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandConfiguration {
    /// Model id as registered in the Models library (local or remote).
    pub model: String,
    pub schedule: HandSchedule,
    /// Every grant is per-Hand and per-skill. A Hand cannot use a skill not listed here.
    pub granted_skills: Vec<String>,
}

/// Step 3 of 3 — Deployment, including the Cedar policy reviewed before activation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandDeployment {
    pub target: DeploymentTarget,
    /// Cedar policy source attached at creation. The policy travels with the
    /// Hand wherever it runs and is evaluated by the Rust core after the model
    /// decides to call a tool but before the tool executes.
    pub cedar_policy: String,
}

/// A fully specified Hand — the product of the three-step wizard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandDefinition {
    pub id: String,
    pub template: Option<HandTemplate>,
    pub identity: HandIdentity,
    pub configuration: HandConfiguration,
    pub deployment: HandDeployment,
    pub status: HandStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HandStatus {
    Active,
    Paused,
    Draft,
}

/// One execution of a Hand, persisted to SurrealDB. Run history is per-instance
/// (FUNC-SPEC §07 — definitions sync, run history does not by default).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandRun {
    pub hand_id: String,
    pub started_at_ms: u64,
    pub finished_at_ms: Option<u64>,
    pub ran_on: DeploymentTarget,
    /// Every skill call recorded — the per-Hand audit log.
    pub skill_invocations: Vec<SkillInvocation>,
    pub outcome: RunOutcome,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInvocation {
    pub skill: String,
    pub input_digest: String,
    pub output_digest: String,
    pub cedar_decision: CedarDecision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CedarDecision {
    Allow,
    Deny,
    /// First-use: execution paused until the user grants or denies.
    ConsentRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RunOutcome {
    Succeeded,
    Failed { error: String },
    Cancelled,
    InProgress,
}

#[derive(Debug, thiserror::Error)]
pub enum HandsError {
    #[error("hand not found: {0}")]
    NotFound(String),
    #[error("skill '{skill}' is not granted to hand '{hand_id}'")]
    SkillNotGranted { hand_id: String, skill: String },
    #[error("scheduler not wired yet — see docs/implementation-roadmap.md (M1)")]
    NotImplemented,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_defaults_match_spec_table() {
        assert_eq!(
            HandTemplate::Researcher.default_schedule(),
            HandSchedule::Cron("0 9 * * *".into())
        );
        assert_eq!(HandTemplate::InboxTriage.required_skills(), &["email-imap"]);
    }
}
