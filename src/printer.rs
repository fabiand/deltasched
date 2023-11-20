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
    text: Vec<String>
}

impl TextSchedule {
    pub fn new() -> TextSchedule {
        TextSchedule {
            text: Vec::new()
        }
    }
    pub fn format(&mut self, doc: &Document) -> String {
        self.visit_document(&doc);
        self.text.concat()
    }

    fn visit_document(&mut self, doc: &Document) { 
        self.text.push(format!("# Kind"));
        self.text.push(format!("kind: {}\n", &doc.kind));
        self.text.push(format!("# Metadata"));
        self.text.push(format!("{}", serde_yaml::to_string(&doc.metadata).unwrap()));
        self.text.push(format!("# Spec"));
        self.visit_schedule(&doc.spec);
        self.text.push(format!("# Status"));
        self.text.push(format!("## Phases & Milestones"));
        if doc.status.is_some() {
            for p in &doc.status.as_ref().unwrap().phases {
                self.visit_phase(&p);
            }
        }
    }

    fn visit_milestone(&mut self, m: &Milestone) {
        let due_date = if m.due_date.is_none()
          { String::from("---------- ---") }
          else
          { format!("{}", m.due_date.unwrap().format("%Y-%m-%d %a")) };
          self.text.push(format!(" {}   {}   {}", due_date, m.alias, m.name));
    }

    fn visit_milestone_generator(&mut self, m: &MilestoneGenerator) {
        self.text.push(format!("{}x {}", m.count, m.name));
    }

    fn visit_phase(&mut self, p: &Phase) {
        self.text.push(format!("{}", p.name));
        for ms in &p.milestones {
            self.visit_milestone(&ms);
        };
    }

    fn visit_schedule(&mut self, s: &Schedule) {
        self.text.push(format!("## Phases & Milestones"));
        for p in &s.phases {
            self.visit_phase(&p);
        }

        self.text.push(format!("## Baseline Deltas"));
        // Note: We are reverting the order and to/from in order to
        // make it ore natural to read the output
        // The output will be from oldest to most recent
        for d in s.milestone_deltas.iter().rev() {
            self.text.push(format!("- {} {} {}: {}", d.milestone, d.is, d.target, d.by));
        }
    }
}
