
Create a schedule with basic milestones automatically.

The basics rely on two elements:

1. A target release date
2. A list of deltas between milestones

# Example

```console
$ cargo run -- -o human example
    Finished dev [unoptimized + debuginfo] target(s) in 0.06s
     Running `target/debug/deltasched -o human example`
# 'TBD' Schedule
## Timeline
- Planning
  - (????-??-?? ???)  RG   Requirements Gathering
  - (????-??-?? ???)  RF   Requirements Freeze

- Development
  - (????-??-?? ???)  FS   Feature Start
  - (????-??-?? ???)  FF   Feature Freeze

- Testing
  - (????-??-?? ???)  BO   Blockers Only
  - (????-??-?? ???)  CF   Code Freeze

- Release
  - (????-??-?? ???)  PS   Push to Stage
  - (????-??-?? ???)  GA   General Availability

## Baseline Deltas
- RF to FF: 6 Sprints
- FF to BO: 1 Sprints
- BO to CF: 1 Sprints
- CF to GA: 4 Weeks

$
```

# New

Create a new draft schedule

```console
$ cargo run -- -o human new --name kubevirt-1.1 --from-skeleton project-skeleton.yaml --with-due-date GA:2023-10-31
    Finished dev [unoptimized + debuginfo] target(s) in 0.06s
     Running `target/debug/deltasched -o human new --name kubevirt-1.1 --from-skeleton project-skeleton.yaml --with-due-date 'GA:2023-10-31'`
# 'kubevirt-1.1' Schedule
## Timeline
- Planning
  - (????-??-?? ???)  RG   Requirements Gathering
  - (2023-04-18 Tue)  RF   Requirements Freeze

- Development
  -  ????-??-?? ???   KV   KubeVirt Feature Freeze
  - (2023-08-22 Tue)  FF   Feature Freeze

- Testing
  - (2023-09-12 Tue)  BO   Blockers Only
  - (2023-10-03 Tue)  CF   Code Freeze

- Release
  - (????-??-?? ???)  PS   Push to Stage
  -  2023-10-31 Tue   GA   General Availability

## Baseline Deltas
- RF to FF: 6 Sprints
- FF to BO: 1 Sprints
- BO to CF: 1 Sprints
- CF to GA: 4 Weeks
$

# Now create a yaml in order to allow us to re-plan:
$ cargo run -- new --name kubevirt-1.1 --from-skeleton project-skeleton.yaml --with-due-date GA:2023-10-31 > draft.yaml
    Finished dev [unoptimized + debuginfo] target(s) in 0.06s
     Running `target/debug/deltasched new --name kubevirt-1.1 --from-skeleton project-skeleton.yaml --with-due-date 'GA:2023-10-31'`
$
```

# Replan

We can now take the draft/existing schedule (`draft.yaml` in the previous example).
For example let's push out the GA by 1 month.

```console
$ edit draft.yaml
# Change GA to 2023-11-30
$  cargo run -- replan --schedule draft.yaml > replanned-draft.yaml
    Finished dev [unoptimized + debuginfo] target(s) in 0.06s
     Running `target/debug/deltasched replan --schedule draft.yaml`
$ diff -u draft.yaml replanned-draft.yaml 
--- draft.yaml	2023-10-20 14:20:20.253701381 +0200
+++ replanned-draft.yaml	2023-10-20 14:21:15.978983749 +0200
@@ -8,7 +8,7 @@
     fixed: false
   - name: Requirements Freeze
     alias: RF
-    due_date: 2023-04-18
+    due_date: 2023-05-18
     fixed: false
 - name: Development
   milestones:
@@ -18,17 +18,17 @@
     fixed: true
   - name: Feature Freeze
     alias: FF
-    due_date: 2023-08-22
+    due_date: 2023-09-21
     fixed: false
 - name: Testing
   milestones:
   - name: Blockers Only
     alias: BO
-    due_date: 2023-09-12
+    due_date: 2023-10-12
     fixed: false
   - name: Code Freeze
     alias: CF
-    due_date: 2023-10-03
+    due_date: 2023-11-02
     fixed: false
 - name: Release
   milestones:
```

## Fixating milestones

In order to not re-plan a certain milestone, the milestone needs to
ne _fixated_ by setting `fixed: true`.
