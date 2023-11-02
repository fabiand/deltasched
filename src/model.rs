use std::fmt;
use std::ops;
use chrono::{Local,Duration,NaiveDate};
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
    pub deltaTemplate: MilestoneRelation,
    pub milestoneTemplate: Milestone
}
impl MilestoneGenerator {
    fn generate_milestones(&self) -> (Vec<MilestoneRelation>, Vec<Milestone>) {
        debug!("Calling milestone generator {}", self.name);
        let mut milestones = Vec::new();
        let mut deltas = Vec::new();

        for counter in 1..self.count+1 {
            let alias = format!("{}{}", &self.milestoneTemplate.alias, counter);
            let name = format!("{}{}", &self.milestoneTemplate.name, counter);

            debug!("Generating milestone {}", alias);

            milestones.push(Milestone{
                alias: alias.clone(),
                due_date: None,
                name: name.clone(),
                ..self.milestoneTemplate
            });

            let delta = &self.deltaTemplate;

            deltas.push(MilestoneRelation {
                    milestone: alias.clone(),
                    is: delta.is.clone(),
                    target: delta.target.clone(),
                    by: delta.by.clone() * counter as i64
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
    fn generate_milestones(&mut self) -> Vec<MilestoneRelation>{
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
    pub milestone_deltas: Vec<MilestoneRelation>
}
impl Schedule {
    pub fn generate(&mut self) {
        debug!("Generating");
        for p in self.phases.iter_mut() {
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
            writeln!(f, "- {} {} {}: {}", d.milestone, d.is, d.target, d.by)?;
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
        let now = Local::now().date_naive();
        let mut ga = Milestone::new("General Availability", "GA");
        ga.due_date = Some(now);
        Phase {
            name: String::from("Release"),
            milestones: vec![Milestone::new("Push to Stage", "PS"), ga],
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
                    deltaTemplate: MilestoneRelation {
                        milestone: "z".to_string(),
                        is: Where::Behind,
                        target: "GA".to_string(),
                        by: SimpleDuration::Weeks{weeks: 4}
                    },
                    milestoneTemplate: Milestone {
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
    fn common_deltas() -> Vec<MilestoneRelation> {
        vec![
            MilestoneRelation::new("GA", Where::Behind, "CF", SimpleDuration::Weeks{weeks: 4}),
            MilestoneRelation::new("CF", Where::Behind, "BO", SimpleDuration::Sprints{sprints: 1}),
            MilestoneRelation::new("BO", Where::Behind, "FF", SimpleDuration::Sprints{sprints: 1}),
            MilestoneRelation::new("FF", Where::Behind, "RF", SimpleDuration::Sprints{sprints: 6}),
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
#[serde(untagged)]
pub enum SimpleDuration {
    Weeks {
        weeks: i64
    },
    Sprints {
        sprints: i64
    }
}
impl SimpleDuration {
    pub fn to_duration(self) -> Duration {
        match self {
            SimpleDuration::Weeks{weeks} => Duration::weeks(weeks),
            SimpleDuration::Sprints{sprints} => Duration::weeks(sprints*3)
        }
    }
}
impl ops::Mul<i64> for SimpleDuration {
    type Output = SimpleDuration;

    fn mul(self, _rhs: i64) -> SimpleDuration {
        match self {
            SimpleDuration::Weeks{weeks} => Self::Weeks{weeks: weeks * _rhs},
            SimpleDuration::Sprints{sprints} => Self::Sprints{sprints: sprints * _rhs},
        }
    }
}

impl fmt::Display for SimpleDuration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SimpleDuration::Weeks{weeks} => {
                write!(f, "{} weeks", weeks)
            },
            SimpleDuration::Sprints{sprints} => {
                write!(f, "{} sprints", sprints)
            }
        }
    }
} 

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a() {
        let d = MilestoneRelation::Behind{
            milestone: "bar".to_string(),
            behind: "bar".to_string(),
            by: SimpleDuration::Weeks{weeks: 4}
        };
        let n = format!("{}", serde_yaml::to_string(&d).unwrap());
        assert_eq!("{}", n);
    }
}

pub type MilestoneAliasOrName = String;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Where {
    Behind,
    AheadOf
}
impl fmt::Display for Where {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Where::Behind => "behind",
            Where::AheadOf => "aheadOf"
        })
    }
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MilestoneRelation {
    milestone: MilestoneAliasOrName,
    is: Where,
    target: MilestoneAliasOrName,
    by: SimpleDuration
}
impl MilestoneRelation {
    pub fn new(milestone: &str, is: Where, target: &str, by: SimpleDuration) -> MilestoneRelation {
        MilestoneRelation {
            milestone: milestone.to_string(),
            is: is,
            target: target.to_string(),
            by: by
        }
    }
    pub fn to_duration(self) -> Duration {
        self.by.to_duration()
    }
}


/// Plan backwards from a given target date
/// Instead of building up a schedule, we look at when we want to release the target)
/// and then backtrack the milestone due dates ahead_ofing up to this target
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
            .map(|m| (&m.milestone, &m.target))
            .collect::<HashSet<(&String, &String)>>().len();

        assert_eq!(num_deltas, num_unique_deltas,
                   "There seems to be a milestone delta dupe");
    }

    for delta in draft.milestone_deltas.clone() {
        debug!("Found delta {:?}", &delta);

        let (milestone_str, is, target_str, by) = (&delta.milestone, &delta.is, &delta.target, &delta.by);

        let target = draft.milestone(target_str)?;

        if let Some(target_due_date) = target.due_date {
            let new_due_date = if *is == Where::Behind {
                target_due_date - by.clone().to_duration()
            } else {
                target_due_date + by.clone().to_duration()
            };
            let milestone = draft.milestone(milestone_str)?;
            debug!("Computed due {} for {}", new_due_date, milestone.alias);
            if milestone.due_date.is_none() { 
                debug!("… applied");
                milestone.due_date = Some(new_due_date)
            } else {
                debug!("… not applied");
            }
        } else {
            debug!("Target {} has no du date, therefore the milestone due date can not be computed", target);
            debug!("Target has no due date");
        }
    }
    Ok(draft)
}


