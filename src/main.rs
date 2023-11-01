use chrono::NaiveDate;
use log::{debug};
use std::collections::HashSet;

mod cli;
use clap::Parser;
use crate::cli::*;

mod model;
use crate::model::*;

mod gantt;

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
