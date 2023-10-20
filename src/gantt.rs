/*pub fn MermaidGanttPrinter(sched: &DraftSchedule) -> String {
    let mut gantt = String::new();
    gantt += &format!(r#"
```mermaid
gantt
    title {}
    dateFormat YYYY-MM-DD"#, sched.name);

    for phase in &sched.phases {
        gantt += &format!("
    section {}",
            phase.name);
        for m in &phase.milestones {
            if m.due_date.is_none() { continue };
            gantt += &format!("
       {}  :milestone, {}, {}, 0d", m.name, m.alias, m.due_date.unwrap())
        }
    }

    gantt += "
```";
    gantt
}
*/
