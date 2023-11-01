use std::fmt;
use std::ops;
use chrono::{Duration,NaiveDate};
use serde::{Serialize, Deserialize};
use serde_yaml;

fn NoNaiveDate() -> Option<NaiveDate> { None }
fn NotFixed() -> bool { false }

#[derive(Clone, Serialize, Deserialize)]
pub struct DraftMilestone {
    pub name: String,
    pub alias: String,
    #[serde(default = "NoNaiveDate")]
    pub due_date: Option<NaiveDate>,
    #[serde(default = "NotFixed")]
    pub fixed: bool
}
impl DraftMilestone {
    pub fn new(name: &str, alias: &str) -> DraftMilestone {
        DraftMilestone{name: String::from(name),
                       alias: String::from(alias),
                       due_date: None,
                       fixed: false}
    }
}
impl ops::Sub<Duration> for &mut DraftMilestone {
    type Output = Result<NaiveDate, String>;

    fn sub(self, _rhs: Duration) -> Result<NaiveDate, String> {
        if self.due_date.is_none() {
            return Err(String::from(format!("No due_date set for '{}'", self)))
        }
        Ok(self.due_date.unwrap() - _rhs)
    }
}
impl fmt::Display for DraftMilestone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let due_date = if self.due_date.is_none()
          { String::from("????-??-?? ???") }
          else
          { format!("{}", self.due_date.unwrap().format("%Y-%m-%d %a")) };
        if self.fixed {
            write!(f, " {}   {}   {}", due_date, self.alias, self.name)
        } else {
            write!(f, "({})  {}   {}", due_date, self.alias, self.name)
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DraftPhase {
    pub name: String,
    pub milestones: Vec<DraftMilestone>
}
impl fmt::Display for DraftPhase {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.name)?;
        for ms in &self.milestones {
            writeln!(f, "  - {}", ms)?
        };
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DraftSchedule {
    pub name: String,
    pub phases: Vec<DraftPhase>,
    pub milestone_deltas: Vec<MilestoneDelta>
}
impl DraftSchedule {
    pub fn milestone(&mut self, alias_or_name: &str) -> Result<&mut DraftMilestone, String> {
        for p in &mut self.phases {
            for m in &mut p.milestones {
                if m.alias == alias_or_name || m.name == alias_or_name {
                    return Ok(m);
                }
            }
        }
        Err(format!("Milestone '{}' not found.", alias_or_name))
    }

    pub fn as_yaml(&self) -> String {
        format!("{}", serde_yaml::to_string(&self).unwrap())
    }
}
impl fmt::Display for DraftSchedule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "# '{}' Schedule", self.name)?;

        writeln!(f, "## Timeline")?;
        for p in &self.phases {
            writeln!(f, "- {}", p)?
        }

        writeln!(f, "## Baseline Deltas")?;
        // Note: We are reverting the order and to/from in order to
        // make it ore natural to read the output
        // The output will be from oldest to most recent
        for d in self.milestone_deltas.iter().rev() {
            writeln!(f, "- {} to {}: {}", d.to, d.from, d.length)?
        }

        Ok(())
    }
}

#[derive(Serialize,Deserialize)]
enum DocumentKind {
    Schedule(DraftSchedule)
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Document {
    pub kind: String,
    pub spec: DraftSchedule,
    pub status: Option<DraftSchedule>
}

impl Document {
    pub fn from_yaml_file(filename: String) -> Document {
        let f = std::fs::File::open(filename).unwrap();
        let doc: Document = serde_yaml::from_reader(f).unwrap();
        doc
    }
    pub fn as_yaml(&self) -> String {
        format!("{}", serde_yaml::to_string(&self).unwrap())
    }
}


/*
 * Builders
 */
pub struct PhaseBuilder {}

impl PhaseBuilder {
    fn default() -> DraftPhase {
        DraftPhase {
            name: String::from(""),
            milestones: vec![]
        }
    }
    pub fn planning() -> DraftPhase {
        DraftPhase {
            name: String::from("Planning"),
            milestones: vec![DraftMilestone::new("Requirements Gathering", "RG"),
                DraftMilestone::new("Requirements Freeze", "RF")],
            ..PhaseBuilder::default()
        }
    }
    pub fn development() -> DraftPhase {
        DraftPhase {
            name: String::from("Development"),
            milestones: vec![DraftMilestone::new("Feature Start", "FS"),
                DraftMilestone::new("Feature Freeze", "FF")],
            ..PhaseBuilder::default()
        }
    }
    pub fn testing() -> DraftPhase {
        DraftPhase {
            name: String::from("Testing"),
            milestones: vec![DraftMilestone::new("Blockers Only", "BO"),
                DraftMilestone::new("Code Freeze", "CF")],
            ..PhaseBuilder::default()
        }
    }
    pub fn release() -> DraftPhase {
        DraftPhase {
            name: String::from("Release"),
            milestones: vec![DraftMilestone::new("Push to Stage", "PS"),
                DraftMilestone::new("General Availability", "GA")],
            ..PhaseBuilder::default()
        }
    }

}

pub struct ScheduleBuilder {
}

impl ScheduleBuilder {
    fn common_phases() -> Vec<DraftPhase> {
        vec![
            PhaseBuilder::planning(),
            PhaseBuilder::development(),
            PhaseBuilder::testing(),
            PhaseBuilder::release(),
        ]
    }
    fn common_deltas() -> Vec<MilestoneDelta> {
        vec![
            MilestoneDelta::new("GA", "CF", (4, "weeks")),
            MilestoneDelta::new("CF", "BO", (1, "sprint")),
            MilestoneDelta::new("BO", "FF", (1, "sprint")),
            MilestoneDelta::new("FF", "RF", (6, "sprints"))
        ]
    }
    pub fn schedule() -> DraftSchedule {
        let draft = DraftSchedule {
            name: String::from("TBD"),
            phases: ScheduleBuilder::common_phases(),
            milestone_deltas: ScheduleBuilder::common_deltas()
        };
        draft
    }
    pub fn from_yaml_file(filename: String) -> DraftSchedule {
        let f = std::fs::File::open(filename).unwrap();
        let parsed_draft: DraftSchedule = serde_yaml::from_reader(f).unwrap();
        parsed_draft
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MilestoneDelta {
    pub from: String,
    pub to: String,
    pub length: DeltaLength
}
impl MilestoneDelta {
    pub fn new(from: &str, to: &str, length: (i64, &str)) -> MilestoneDelta {
        MilestoneDelta{
            from: from.to_string(),
            to: to.to_string(),
            length: DeltaLength::new(length.0, length.1)
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DeltaLength {
    count: i64,
    unit: DeltaLengthUnit
}
impl fmt::Display for DeltaLength {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {:?}", self.count, self.unit)
    }
}


#[derive(Serialize, Deserialize, Clone, Debug)]
enum DeltaLengthUnit {
    Weeks,
    Sprints
}
impl DeltaLengthUnit {
    fn parse(unit_str: &str) -> DeltaLengthUnit {
        if unit_str.to_lowercase().starts_with("s") {
            DeltaLengthUnit::Sprints
        } else {
            DeltaLengthUnit::Weeks
        }
    }
}

impl DeltaLength {
    fn new(count: i64, unit: &str) -> DeltaLength {
        DeltaLength {
            count,
            unit: DeltaLengthUnit::parse(unit)
        }
    }
    pub fn to_duration(self) -> Duration {
        match self.unit {
            DeltaLengthUnit::Weeks => Duration::weeks(self.count),
            DeltaLengthUnit::Sprints => Duration::weeks(self.count) * 3
        }
    }
}


