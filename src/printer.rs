use serde_yaml;

use crate::model::*;

pub struct ScheduleFormatter{}

impl ScheduleFormatter {
    pub fn as_yaml(&self, sched: &Document) -> String {
        format!("{}", serde_yaml::to_string(&sched).unwrap())
    }

    pub fn as_text(&self, sched: &Document) -> String {
        let mut text = TextSchedule::new();
        format!("{}", text.format(sched))
    }
}

pub struct TextSchedule {
}

impl TextSchedule {
    pub fn new() -> TextSchedule {
        TextSchedule {
        }
    }
    pub fn format(&mut self, doc: &Document) -> String {
        self.visit_document(&doc)
    }

    fn visit_document(&mut self, doc: &Document) -> String {
        let mut text = Vec::new();
        text.push(format!("# Kind"));
        text.push(format!("kind: {}\n", &doc.kind));
        text.push(format!("# Metadata"));
        text.push(format!("{}", serde_yaml::to_string(&doc.metadata).unwrap()));
        text.push(format!("# Spec"));
        text.push(self.visit_schedule(&doc.spec));
        text.push(format!("\n# Status"));
        text.push(format!("## Phases & Milestones"));
        if doc.status.is_some() {
            for p in &doc.status.as_ref().unwrap().phases {
                text.push(self.visit_phase(&p));
                text.push(String::from(""));
            }
        }
        text.join("\n")
    }

    fn visit_milestone(&mut self, m: &Milestone) -> String {
        let due_date = if m.due_date.is_none()
          { String::from("????-??-?? ???") }
          else
          { format!("{}", m.due_date.unwrap().format("%Y-%m-%d %a")) };
        format!("  -  {}   {}   {}", due_date, m.alias, m.name)
    }

    fn visit_milestone_generator(&mut self, m: &MilestoneGenerator) -> String {
        format!("{}x {}", m.count, m.name)
    }

    fn visit_phase(&mut self, p: &Phase) -> String {
        let mut text = Vec::new();
        text.push(format!("- {}", p.name));
        for ms in &p.milestones {
            text.push(self.visit_milestone(&ms));
        };
        text.join("\n")
    }

    fn visit_schedule(&mut self, s: &Schedule) -> String {
        let mut text = Vec::new();
        text.push(format!("## Phases & Milestones"));
        for p in &s.phases {
            text.push(self.visit_phase(&p));
            text.push(String::from(""));
        }

        text.push(format!("## Baseline Deltas"));
        // Note: We are reverting the order and to/from in order to
        // make it ore natural to read the output
        // The output will be from oldest to most recent
        for d in s.milestone_deltas.iter().rev() {
            text.push(format!("- {} {} {}: {}", d.milestone, d.is, d.target, d.by));
        }
        text.join("\n")
    }
}
