use chrono::NaiveDate;

mod cli;
use clap::Parser;
use crate::cli::*;

mod model;
use crate::model::*;

mod printer;

mod gantt;

fn main() {
    env_logger::init();

    let args = Args::parse();

    let print_schedule = |sched: &Document| {
        let schedfmt = printer::ScheduleFormatter{};
        let output;
        match args.output {
            OutputFormat::Yaml => {
                output = format!("{}", &schedfmt.as_yaml(&sched));
            },
            OutputFormat::Human => {
                output = format!("{}", &sched);
//                output = format!("{}", &schedfmt.as_text(&sched));
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
            sched = Document::example();
        }
        Commands::New { name, from_skeleton, with_due_date } => {
            let mut draft = Document::from_yaml_file(from_skeleton.to_string());
            draft.metadata.insert("name".to_string(), name.to_string());
            let (milestone_alias, due_date) = {
                let Some((raw_ms, raw_due_date)) = with_due_date.split_once(":") else { todo!() };
                (raw_ms, NaiveDate::parse_from_str(raw_due_date, "%Y-%m-%d").unwrap())
            };
            draft.replan(Some((milestone_alias, due_date)));
            sched = draft;
        }
        Commands::Replan { schedule } => {
            //sched = ScheduleBuilder::from_yaml_file(schedule.to_string());
            sched = Document::from_yaml_file(schedule.to_string());
            sched.replan(None);
        }
    }
    print_schedule(&sched);
}
