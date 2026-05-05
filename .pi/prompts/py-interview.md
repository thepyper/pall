---
description: Interview to define a plan 
argument-hint: "<plan-name>"
---
## Procedure

# Interview

First ask the user for the broad objective for this plan.

Then gather some sensible context.

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
Write the plan in:
.py/plans/$1/PLAN.md 

## Completion criteria

[ ] INTERVIEW.md has been written with full information captured by the whole Q&A session
[ ] PLAN.md has been written with a complete, detailed, micro-stepped (single concern per-step) plan ready to be executed to obtain the objective 

IMPORTANT: DO NOT STOP UNTIL ALL COMPLETION CRITERIA HAS BEEN MET!!
