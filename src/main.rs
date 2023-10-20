use std::fmt;
use std::ops;
use chrono::{Duration,NaiveDate};
use serde::{Serialize, Deserialize};
use serde_yaml;
use clap::{Parser, Subcommand, ValueEnum};
use log::{debug};
use env_logger;
use std::collections::HashSet;

mod gantt;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, default_value="yaml")]
    output: OutputFormat,

    #[command(subcommand)]
    command: Commands
}

#[derive(ValueEnum, Debug, Clone)]
enum OutputFormat {
    Yaml,
    Human/*,
    MermaidGantt*/
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Output an example schedule skeleton
    Example {},

    /// Create a new schedule
    New {
        /// Name of the project release
        #[arg(long)]
        name: String,
        /// The schedule skeleton to use
        #[arg(long)]
        from_skeleton: String,
        /// The known dates, a tuple of <milestone alias>:<yyyy-mm-dd>
        #[arg(long)]
        with_due_date: String
    },

    /// Replan an existing schedule
    /// It is expected that to be replanned dates are null
    Replan {
        #[arg(long)]
        schedule: String
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct DraftMilestone {
    name: String,
    alias: String,
    due_date: Option<NaiveDate>,
    fixed: bool
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
struct DraftPhase {
    name: String,
    milestones: Vec<DraftMilestone>
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

#[derive(Serialize, Deserialize)]
struct DraftSchedule {
    name: String,
    phases: Vec<DraftPhase>,
    milestone_deltas: Vec<MilestoneDelta>
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


/*
 * Builders
 */
struct PhaseBuilder {}

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

struct ScheduleBuilder {
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
struct MilestoneDelta {
    from: String,
    to: String,
    length: DeltaLength
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
struct DeltaLength {
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
    fn to_duration(self) -> Duration {
        match self.unit {
            DeltaLengthUnit::Weeks => Duration::weeks(self.count),
            DeltaLengthUnit::Sprints => Duration::weeks(self.count) * 3
        }
    }
}

/// Plan backwards from a given target date
/// Instead of building up a schedule, we look at when we want to release the target)
/// and then backtrack the milestone due dates leading up to this target
fn plan_backwards(draft: &mut DraftSchedule, target: Option<(&str, NaiveDate)>) -> Result<(), String> {
    debug!("Planning backwards of {}", draft.name);

    if let Some((milestone_alias, due_date)) = target {
        debug!("Target is provided: {} on {}", milestone_alias, due_date);
        let target = draft.milestone(&milestone_alias)?;
        target.due_date = Some(due_date);
        target.fixed = true;
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
            if !to_milestone.fixed || to_milestone.due_date.is_none() { 
                to_milestone.due_date = Some(to_due_date)
            } else {
                debug!("New computed due was not applied");
            }
        }
    }
    Ok(())
}


fn main() {
    env_logger::init();

    let args = Args::parse();

    let print_schedule = |sched: &DraftSchedule| {
        let output;
        match args.output {
            OutputFormat::Yaml => {
                output = format!("{}", &sched.as_yaml());
            },
            OutputFormat::Human => {
                output = format!("{}", &sched);
            }/*,
            OutputFormat::MermaidGantt => {
                output = format!("{}", gantt::MermaidGanttPrinter(&sched));
            }*/
        }
        println!("{}", output);
    };

    let mut sched;

    match &args.command {
        Commands::Example {} => {
            sched = ScheduleBuilder::schedule();
        }
        Commands::New { name, from_skeleton, with_due_date } => {
            let mut draft = ScheduleBuilder::from_yaml_file(from_skeleton.to_string());
            draft.name = name.to_string();
            let (milestone_alias, due_date) = {
                let Some((raw_ms, raw_due_date)) = with_due_date.split_once(":") else { todo!() };
                (raw_ms, NaiveDate::parse_from_str(raw_due_date, "%Y-%m-%d").unwrap())
            };
            let _ = plan_backwards(&mut draft, Some((milestone_alias, due_date)));
            sched = draft;
        }
        Commands::Replan { schedule } => {
            sched = ScheduleBuilder::from_yaml_file(schedule.to_string());
            let _ = plan_backwards(&mut sched, None);
        }
    }
    print_schedule(&sched);
}
