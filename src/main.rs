use chrono::NaiveDate;

mod cli;
use clap::Parser;
use crate::cli::*;

mod model;
use crate::model::*;

                use serde_yaml;
mod gantt;

use wasm_bindgen::prelude::*;
#[wasm_bindgen]
extern {
    pub fn alert(s: &str);
}
#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello, {} from delta!", name));
}


fn main() {
    env_logger::init();

    let args = Args::parse();

    let print_schedule = |sched: &Document| {
        let output;
        match args.output {
            OutputFormat::Yaml => {
                output = format!("{}", serde_yaml::to_string(&sched).unwrap())
//                output = format!("{}", &schedfmt.as_yaml(&sched));
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


#[wasm_bindgen]
pub struct WasmSched {
    doc: Document
}

#[wasm_bindgen]
impl WasmSched {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmSched {
        WasmSched {
            doc: Document::example() //from_yaml_file(schedule);
        }
    }

    pub fn replan(&mut self) {
        self.doc.replan(None);
    }

    pub fn as_json_string(&self) -> String {
        self.doc.as_json()
    }
}
