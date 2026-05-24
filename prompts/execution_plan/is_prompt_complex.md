Respond **TRUE** for a message that can be considered a task with multiple phases.
Respond **FALSE** for a message that is a simple conversation, simple question or a gretting.

EXAMPLES:

- read file <filename>: returns false;
- read file <filename> and write something to <filename>: returns true.

IMPORTANT:

- if the prompt require analyzing, understanding or searching for something in the project, return true.
- respond only with true or false
