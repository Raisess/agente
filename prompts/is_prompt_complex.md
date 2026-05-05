If the next prompt is has more than one action, return true, otherwise return false.

EXAMPLES:

- read file <filename>: returns false;
- read file <filename> and write something to <filename>: returns true.

IMPORTANT:

- if the prompt is a error message or a piece of code, just returns false;
- if the prompt is a question, just return false.
