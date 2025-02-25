# Notes

Here, I (*a real human*) will document my experience working with the AI tools to make this.

## 2025-02-25

Started the project. This is using Windsurf IDE with Claude 3.5 as the model. There was a moment or two I tried Deepseek, o3-mini, and others but they had fundamental issues greater than using Claude as the model.

The code committed on this day was pretty much all written by the AI.

I will document the instances of human intervention that went beyond simple prompting guidance.


### Human Interventions

1. The AI was having issues on finding out how to get the `TSType` from a `TSTupleElement` from the OXC ast. After prompting it to specifically check the documentation failed, I found the name of the method to use to do this. I just asked it to check the docs for that method, and then it was able to move forward, interestingly choosing to use a different sister method that would be more type-safe than the one I pointed it to.

2. During tests related to the return types of functions, it had tests that were correctly giving errors, but giving the wrong errors that it didn't realize.
```
Error that was found: "Type 'any' cannot be assigned to type 'string'"
Error that was expected: "Type 'number' cannot be assigned to type 'string'"
Error check used by AI to validate: error.contains("cannot be assigned to type 'string'")
```

When doing later code changes for other features, one of those tests began to fail, and the AI kept trying to fix it in the code it just wrote.

While this was inline with given guidance, due to the incorrect expectation of WHY the test case would error, it was not able to identify what was going wrong and mostly got stuck in a loop.

I identified the issue (not properly passing function param types into the symbol lookup), and directly edited the test case to check for the CORRECT error, and not allowing an incorrect error to pass.

### Are we AI yet?

Not yet.

Despite the usefullness of Tests and a verbose compiler like Rust, the AI can still end up making changes in the wrong places for the wrong reasons or getting in a loop instead of really analyzing the docs.
