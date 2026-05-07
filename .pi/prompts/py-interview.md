---
description: Interview to define and execute a plan 
argument-hint: "<plan-name>"
---
## Procedure

This describes a procedure to interview the user, then create a plan, review it, and execute it.
The following files will be created in the process:
.py/plans/$1/INTERVIEW.md   - interview recap
.py/plans/$1/PLAN.md        - actual plan

# Objective

First ask the user for the broad objective for this plan.

Then gather the context you need to ask informed questions — stop when you have enough to start the interview.

# Interview

Then, interview the user.
Asking the user all the informations you cannot gather by yourself to reach the objective.
Continue the interview as long as you need more information or have doubts about the objective and how to reach it.

When you have enough information, ALWAYS ask this last question:
"I think I have enough information. Is there anything else you think I should know?"
If new information arise from this question, enter again the interviewing loop.

When the interview is ended, create or overwrite a detailed summary of 
everything has been decided about the objective of the plan in INTERVIEW.md.

Only write the final decisions:
- it's not important to record anytime somebody changed it's mind;
- it's important to capture what has been actually decided in the end.

Record EVERYTHING that is useful to convey the objective and the details that have been discovered in the interview. 

Then do git commit (for INTERVIEW.md file).

# Plan

Then, create a detailed plan out of this interview:
- create MICRO steps that are easy and address a single concern for what is possible;
- divide the plan in phases, each made of steps (if the plan is too easy to split, just do one phase);
- DO include (even if not mentioned in interview) final steps that verify compilation and correct test results.

If the planning runs into some serious trouble that you feel you cannot solve, 
in this case ONLY you are allowed to ask the user to help solving the issue. 
If that happens, update INTERVIEW.md as well with new findings and then try again planning.

When the plan in good, then write it into PLAN.md.

Then do git commit (for PLAN.md and / or INTERVIEW.md file).

# Review

Then, review the plan, looking for errors, problems, missing parts, inconsistencies, discrepancies with the interview, missing points from the interview:
- if the plan has some problems that you can solve independenly, update INTERVIEW.md and / or PLAN.md accordingly, and do git commit;
- if the plan has problems that need user attention, do another round of interview with the user, then plan again, then review again;
- whenever the plan is changed, do git commit (for INTERVIEW.md and PLAN.md files).

# Execution

Once you have a good, correct, reviewed plan, then you can execute it:
- execute the plan step by step;
- do git commit after each step.

## Completion criteria

[ ] INTERVIEW.md has been written with full information captured by the whole Q&A session
[ ] PLAN.md has been written with a complete, detailed, micro-stepped (single concern per-step) plan ready to be executed to obtain the objective 
[ ] Plan has been reviewed with no unresolved issues
[ ] Plan has been executed with all steps completed and all tests passing

IMPORTANT: DO NOT STOP UNTIL ALL COMPLETION CRITERIA HAS BEEN MET!!
