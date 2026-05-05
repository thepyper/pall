---
description: Execute a defined plan
argument-hint: "<plan-name>"
---
## Procedure

# Review

First read the plan an the interview that generated it from:
.py/plans/$1/INTERVIEW.md 
.py/plans/$1/PLAN.md 

Review the plan for errors, problems, missing parts, inconsistencies.
If the plan does not feel good enough to be executed, explain to the user and ask
if ho prefers to stop and review the plan first to solve problems, or continue with the plan as-is.

# Execution

Execute the plan using subagent worker.
You MUST USE the subagent, because you risk to get out of context window. DO USE THE SUBAGENT.

Execute the plan sequentially: NEVER launch more than one subagent, and ALWAYS wait for the subagent to finish it's work before continuing.
Execute the plan step-by-step: you can group some steps if you feel this does not make it too complicated for the subagent.
When you launch a subagent, give him context of full INTERVIEW.md and PLAN.md.

After each subagent execution, do git commit.

## Completion criteria

[ ] Plan has been initially reviewed
[ ] User has accepted execution if prompted to decide
[ ] Plan has been completely executed in all its parts

IMPORTANT: DO NOT STOP UNTIL ALL COMPLETION CRITERIA HAS BEEN MET!!
