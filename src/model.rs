use std::fmt;
use std::ops;
use chrono::{Duration,NaiveDate};
use serde::{Serialize, Deserialize};
use serde_yaml;
use log::{debug};
use std::collections::HashMap;
use std::collections::HashSet;

fn no_naivedate() -> Option<NaiveDate> { None }

#[derive(Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub name: String,
    pub alias: String,
    #[serde(default = "no_naivedate", skip_serializing_if = "Option::is_none")]
    pub due_date: Option<NaiveDate>,
}
impl Milestone {
    pub fn new(name: &str, alias: &str) -> Milestone {
        Milestone{name: String::from(name),
                       alias: String::from(alias),
                       due_date: None}
    }
}
impl ops::Sub<Duration> for &mut Milestone {
    type Output = Result<NaiveDate, String>;

    fn sub(self, _rhs: Duration) -> Result<NaiveDate, String> {
        if self.due_date.is_none() {
            return Err(String::from(format!("No due_date set for '{}'", self)))
        }
        Ok(self.due_date.unwrap() - _rhs)
    }
}
impl fmt::Display for Milestone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let due_date = if self.due_date.is_none()
          { String::from("????-??-?? ???") }
          else
          { format!("{}", self.due_date.unwrap().format("%Y-%m-%d %a")) };
          write!(f, " {}   {}   {}", due_date, self.alias, self.name)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MilestoneGenerator {
    pub name: String,
    pub count: u32,
    pub cadence: DeltaLength,
    pub template: Milestone
}
impl MilestoneGenerator {
    fn generate_milestones(&self) -> (Vec<MilestoneDelta>, Vec<Milestone>) {
        debug!("Calling milestone generator {}", self.name);
        let mut milestones = Vec::new();
        let mut deltas = Vec::new();

        for counter in 1..self.count+1 {
            let alias = format!("{}{}", self.template.alias.clone(), counter);

            debug!("Generating milestone {}", alias);

            milestones.push(Milestone{
                alias: alias.clone(),
                due_date: None,
                name: "".to_string(),
                ..self.template
            });

            deltas.push(MilestoneDelta{
                from: "GA".to_string(),
                to: alias.clone(),
                length: DeltaLength {
                    count: self.cadence.count * counter as i64 * -1,
                    unit: self.cadence.unit.clone()
                }
            });
        }
        (deltas, milestones)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Phase {
    pub name: String,
    pub milestones: Vec<Milestone>,
    #[serde(skip_serializing_if = "Option::is_none", alias = "milestoneGenerators")]
    pub milestone_generators: Option<Vec<MilestoneGenerator>>,
}
impl Phase {
    fn generate_milestones(&mut self) -> Vec<MilestoneDelta>{
        debug!("Generating milestone of phase {}", self.name);

        //if self.milestones_generated { return
        let mut milestones = Vec::new();
        let mut more_deltas = Vec::new();
        
        if self.milestone_generators.is_none() {
            debug!("No milestone generator");
            return more_deltas;
        };

        for g in self.milestone_generators.as_ref().unwrap().iter() {
            let (deltas, more_milestones) = g.generate_milestones();
            milestones.extend(more_milestones);
            more_deltas.extend(deltas);
        }

        self.milestones.extend(milestones);

        debug!("More deltas {:?}", more_deltas);
        more_deltas
    }
}
impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.name)?;
        for ms in &self.milestones {
            writeln!(f, "  - {}", ms)?
        };
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Schedule {
    pub phases: Vec<Phase>,
    pub milestone_deltas: Vec<MilestoneDelta>
}
impl Schedule {
    pub fn generate(&mut self) {
        debug!("Generating");
        for mut p in self.phases.iter_mut() {
            let more_deltas = p.generate_milestones();
            self.milestone_deltas.extend(more_deltas);
        }
    }

    pub fn milestone(&mut self, alias_or_name: &str) -> Result<&mut Milestone, String> {
        for p in &mut self.phases {
            for m in &mut p.milestones {
                if m.alias == alias_or_name || m.name == alias_or_name {
                    return Ok(m);
                }
            }
        }
        Err(format!("Milestone '{}' not found.", alias_or_name))
    }
}
impl fmt::Display for Schedule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "## Phases & Milestones")?;
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

#[derive(Serialize, Deserialize, Clone)]
pub struct Document {
    pub kind: String,
    pub metadata: HashMap<String, String>,
    pub spec: Schedule,
    pub status: Option<DocumentStatus>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DocumentStatus {
    pub phases: Vec<Phase>
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
    pub fn example() -> Document {
        Document {
            kind: "Schedule".to_string(),
            metadata: HashMap::from([
                ("name".to_string(), "example".to_string())
                ]),
            spec: ScheduleBuilder::schedule(),
            status: None
        }
    }
    pub fn replan(&mut self, target: Option<(&str, NaiveDate)>) {
        let new_sched = plan_backwards(&self.spec, target).unwrap();
        let computed_phases = new_sched.phases;
        self.status = Some(DocumentStatus {
            phases: computed_phases
        })
    }
}
impl fmt::Display for Document {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "# Kind")?;
        writeln!(f, "kind: {}\n", &self.kind)?;
        writeln!(f, "# Metadata")?;
        writeln!(f, "{}", serde_yaml::to_string(&self.metadata).unwrap())?;
        writeln!(f, "# Spec")?;
        writeln!(f, "{}", &self.spec)?;
        writeln!(f, "# Status")?;
        writeln!(f, "## Phases & Milestones")?;
        for p in &self.status.as_ref().unwrap().phases {
            writeln!(f, "- {}", p)?
        }
        Ok(())
    }
}

/*
 * Builders
 */
pub struct PhaseBuilder {}

impl PhaseBuilder {
    fn default() -> Phase {
        Phase {
            name: String::from(""),
            milestones: vec![],
            milestone_generators: None
        }
    }
    pub fn planning() -> Phase {
        Phase {
            name: String::from("Planning"),
            milestones: vec![Milestone::new("Requirements Gathering", "RG"),
                Milestone::new("Requirements Freeze", "RF")],
            ..PhaseBuilder::default()
        }
    }
    pub fn development() -> Phase {
        Phase {
            name: String::from("Development"),
            milestones: vec![Milestone::new("Feature Start", "FS"),
                Milestone::new("Feature Freeze", "FF")],
            ..PhaseBuilder::default()
        }
    }
    pub fn testing() -> Phase {
        Phase {
            name: String::from("Testing"),
            milestones: vec![Milestone::new("Blockers Only", "BO"),
                Milestone::new("Code Freeze", "CF")],
            ..PhaseBuilder::default()
        }
    }
    pub fn release() -> Phase {
        Phase {
            name: String::from("Release"),
            milestones: vec![Milestone::new("Push to Stage", "PS"),
                Milestone::new("General Availability", "GA")],
            ..PhaseBuilder::default()
        }
    }
    pub fn maintenance() -> Phase {
        Phase {
            name: String::from("Maintenance"),
            milestones: vec![],
            milestone_generators: Some(vec![
                MilestoneGenerator {
                    name: "zstreams".to_string(),
                    count: 10,
                    cadence: DeltaLength::new(4, "weeks"),
                    template: Milestone {
                        name: "zstream".to_string(),
                        alias: "z".to_string(),
                        due_date: None
                    }
                }
                ]),
            ..PhaseBuilder::default()
        }
    }

}

pub struct ScheduleBuilder {
}

impl ScheduleBuilder {
    fn common_phases() -> Vec<Phase> {
        vec![
            PhaseBuilder::planning(),
            PhaseBuilder::development(),
            PhaseBuilder::testing(),
            PhaseBuilder::release(),
            PhaseBuilder::maintenance(),
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
    pub fn schedule() -> Schedule {
        let draft = Schedule {
            phases: ScheduleBuilder::common_phases(),
            milestone_deltas: ScheduleBuilder::common_deltas()
        };
        draft
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

/// Plan backwards from a given target date
/// Instead of building up a schedule, we look at when we want to release the target)
/// and then backtrack the milestone due dates leading up to this target
fn plan_backwards(sched: &Schedule, target: Option<(&str, NaiveDate)>) -> Result<Schedule, String> {
    let mut draft = sched.clone();
    //debug!("Planning backwards of {}", draft.metadata.entry("name"));

    draft.generate();

    if let Some((milestone_alias, due_date)) = target {
        debug!("Target is provided: {} on {}", milestone_alias, due_date);
        let target = draft.milestone(&milestone_alias)?;
        target.due_date = Some(due_date);
    } else {
        debug!("No target was provided");
    }

    {
        // Ensure that we don't have deltas for the same
        // MS combo twice
        let num_deltas = draft.milestone_deltas.len();
        let num_unique_deltas = draft.milestone_deltas
            .iter()
            .map(|m| (&m.from, &m.to))
            .collect::<HashSet<(&String, &String)>>().len();

        assert_eq!(num_deltas, num_unique_deltas,
                   "There seems to be a milestone delta dupe");
    }

    for delta in draft.milestone_deltas.clone() {
        debug!("Found delta {:?}", &delta);
        let from = draft.milestone(&delta.from)?;
        if let Some(from_due_date) = from.due_date {
            let to_due_date = from_due_date - delta.length.to_duration();
            let to_milestone = draft.milestone(&delta.to)?;
            debug!("Computed due {} for {}", to_due_date, to_milestone.alias);
            if to_milestone.due_date.is_none() { 
                debug!("… applied");
                to_milestone.due_date = Some(to_due_date)
            } else {
                debug!("… not applied");
            }
        } else {
            debug!("No due date");
        }
    }
    Ok(draft)
}


