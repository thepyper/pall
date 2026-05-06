---
description: Interview to define a plan 
argument-hint: "<plan-name>"
---
## Procedure

# Objective

First ask the user for the broad objective for this plan.

Then gather some sensible context.

# Interview

Then, interview the user.
Asking the user all the informations you cannot gather by yourself to reach the objective.
Continue the interview as long as you need more information or have doubts about the objective and how to reach it.

When you have enough information, ALWAYS ask this last question:
"I think I have enough information. Is there anything else you think I should know?"
If new information arise from this question, enter again the interviewing loop.

When the interview is ended, create or overwrite a detailed summary of everything has been decided about the objective of the plan in:
.py/plans/$1/INTERVIEW.md 

Only write the final decisions, it's not important to record anytime somebody changed it's mind, it's important to capture what has been actually decided in the end.
Record everything that is useful to convey the objective and the details that have been discovered in the interview. THIS IS YOUR ONLY TIME TO SAVE INFORMATION, THEN ALL IS LOST.

# Plan

Then, create a detailed plan out of this interview, with care to create MICRO steps that are easy and address a single concern for what is possible.
The plan should be divided in phases, each made of steps (if the plan is too easy to split, just do one phase).
The plan MUST include (even if not mentioned in interview) final steps that verify compilation and correct test results.
Write the plan in:
.py/plans/$1/PLAN.md 

# Review

Then, review the plan, looking for errors, problems, missing parts, inconsistencies, discrepancies with the interview, missing points from the interview.
If the plan does not feel good enough to be executed, explain to the user the problems and ask if ho prefers to solve the problems or continue as-is.
If the user wants to solve the problems, return to the interview phase, then integrate INTERVIEW.md, then reformulate PLAN.md, then review again.

## Completion criteria

[ ] INTERVIEW.md has been written with full information captured by the whole Q&A session
[ ] PLAN.md has been written with a complete, detailed, micro-stepped (single concern per-step) plan ready to be executed to obtain the objective 
[ ] Plan review has been done with positive outcome

IMPORTANT: DO NOT STOP UNTIL ALL COMPLETION CRITERIA HAS BEEN MET!!
