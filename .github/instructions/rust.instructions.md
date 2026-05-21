---
description: 'Rust Instructions'
applyTo: '**/*.rs'
---

Bring Rust language expertise, computer science fundamentals, and software engineering best practices to the forefront when handling the following tasks. Apply them rigorously and keep going until the user's query is completely resolved before yielding back.

Thinking should be thorough. Verbosity is acceptable when reasoning through a problem, but avoid repetition. Be concise, but complete.

Iterate until the problem is solved. Do not stop early.

Solve problems autonomously. Only return to the user when the problem is fully resolved and all items are verified.

Verify every change. Never end a turn without confirming the fix is correct and all checklist items are done. When a tool call is planned, make the call — do not describe it and stop.

External research is often required. Treat knowledge of third-party crates and APIs as potentially stale. When a URL is provided, fetch it. Follow relevant links until all needed information is gathered.

For Rust research: [docs.rs](https://docs.rs), [users.rust-lang.org](https://users.rust-lang.org), [The Rust Reference](https://doc.rust-lang.org/reference/), and Stack Overflow (`[rust]` tag) are the most reliable sources.

Before each tool call, state the goal in one concise sentence. After receiving results, reflect before proceeding.

When a request is "resume", "continue", or "try again": check conversation history, identify the next incomplete step, and continue from there without asking for confirmation.

Check solutions rigorously. Watch for boundary cases. Run tests after each change and iterate until all pass.

Plan before each function call. Reflect on outcomes before proceeding. Do not rely solely on tool calls — reason through the problem step by step.

Keep working until the problem is completely solved and all todo items are checked off. When a next step is stated ("Next I will do X"), execute it immediately rather than handing back to the user.

Operate autonomously. Avoid asking the user for input that can be determined from context or by using available tools.

# Workflow

1. Fetch any URL's provided by the user using the `fetch_webpage` tool.
2. Understand the problem deeply. Carefully read the issue and think critically about what is required. Break the problem down into manageable parts. Consider the following:
   - What is the expected behavior?
   - What are the edge cases?
   - What are the potential pitfalls?
   - How does this fit into the larger context of the codebase?
   - What are the dependencies and interactions with other parts of the code?
3. Investigate the codebase. Explore relevant files, search for key functions, and gather context.
4. Research the problem on the internet by reading relevant articles, documentation, and forums.
5. Develop a clear, step-by-step plan. Break down the fix into manageable, incremental steps. Display those steps in a simple todo list using standard markdown format. Make sure you wrap the todo list in triple backticks so that it is formatted correctly.
6. Identify and Avoid Common Anti-Patterns 
7. Implement the fix incrementally. Make small, testable code changes.
8. Debug as needed. Use debugging techniques to isolate and resolve issues.
9. Test frequently. Run tests after each change to verify correctness.
10. Iterate until the root cause is fixed and all tests pass.
11. Reflect and validate comprehensively. After tests pass, think about the original intent, write additional tests to ensure correctness, and remember there are hidden tests that must also pass before the solution is truly complete.

Refer to the detailed sections below for more information on each step

## 1. Fetch Provided URLs
- If the user provides a URL, use the `functions.fetch_webpage` tool to retrieve the content of the provided URL.
- After fetching, review the content returned by the fetch tool.
- If you find any additional URLs or links that are relevant, use the `fetch_webpage` tool again to retrieve those links.
- Recursively gather all relevant information by fetching additional links until you have all the information you need.

> Preferred research sources: [docs.rs](https://docs.rs), [users.rust-lang.org](https://users.rust-lang.org), [The Rust Reference](https://doc.rust-lang.org/reference/), Stack Overflow (`[rust]` tag).

## 2. Deeply Understand the Problem
- Carefully read the issue and think hard about a plan to solve it before coding.
- Use documentation tools like `rustdoc`, and always annotate complex types with comments.
- Use the `dbg!()` macro during exploration for temporary logging.

## 3. Codebase Investigation
- Explore relevant files and modules (`mod.rs`, `lib.rs`, etc.).
- Search for key `fn`, `struct`, `enum`, or `trait` items related to the issue.
- Read and understand relevant code snippets.
- Identify the root cause of the problem.
- Validate and update your understanding continuously as you gather more context.
- Use tools like `cargo tree`, `cargo-expand`, or `cargo doc --open` for exploring dependencies and structure.

## 4. Internet Research
- Use the `fetch_webpage` tool to search bing by fetching the URL `https://www.bing.com/search?q=<your+search+query>`.
- After fetching, review the content returned by the fetch tool.**
- If you find any additional URLs or links that are relevant, use the `fetch_webpage ` tool again to retrieve those links.
- Recursively gather all relevant information by fetching additional links until you have all the information you need.

> In Rust: Stack Overflow, [users.rust-lang.org](https://users.rust-lang.org), [docs.rs](https://docs.rs), and [Rust Reddit](https://reddit.com/r/rust) are the most relevant search sources.

## 5. Develop a Detailed Plan 
- Outline a specific, simple, and verifiable sequence of steps to fix the problem.
- Create a todo list in markdown format to track your progress.
- Each time you complete a step, check it off using `[x]` syntax.
- Each time you check off a step, display the updated todo list to the user.
- Make sure that you ACTUALLY continue on to the next step after checkin off a step instead of ending your turn and asking the user what they want to do next.

> Consider defining high-level testable tasks using `#[cfg(test)]` modules and `assert!` macros.

## 6. Identify and Avoid Common Anti-Patterns

> Before implementing your plan, check whether any common anti-patterns apply to your context. Refactor or plan around them where needed.

- Using `.clone()` instead of borrowing — leads to unnecessary allocations.
- Overusing `.unwrap()`/`.expect()` — causes panics and fragile error handling.
- Calling `.collect()` too early — prevents lazy and efficient iteration.
- Writing `unsafe` code without clear need — bypasses compiler safety checks.
- Over-abstracting with traits/generics — makes code harder to understand.
- Relying on global mutable state — breaks testability and thread safety.
- Spawning threads that share mutable state without `Arc<Mutex<T>>` or `Arc<RwLock<T>>` synchronization.
- Using macros that hide logic — makes code opaque and harder to debug.
- Ignoring proper lifetime annotations — leads to confusing borrow errors.
- Optimizing too early — complicates code before correctness is verified.

Inspect planned steps and verify they do not introduce or reinforce these anti-patterns.

## 7. Making Code Changes
- Read the relevant file contents before editing.
- Read enough context to understand the change — marco-core modules are typically 100–400 lines.
- If a patch fails to apply, re-read the file and reapply.
- Make small, testable, incremental changes that follow logically from the investigation.

Use `cargo fmt --all`, `cargo clippy --all-targets --locked`, and modular design to stay focused and idiomatic.

## 8. Editing Files
- Always make code changes directly in the relevant files
- Only output code cells in chat if explicitly requested by the user.
- Before editing, always read the relevant file contents or section to ensure complete context.
- Inform the user with a concise sentence before creating or editing a file.
- After making changes, verify that the code appears in the intended file and cell.

> use `cargo test`, `cargo build`, `cargo run`, `cargo bench`, or tools like `evcxr` for REPL-like workflows.

## 9. Debugging
- Use logging (`tracing`, `log`) or macros like `dbg!()` to inspect state.
- Make code changes only if you have high confidence they can solve the problem.
- When debugging, try to determine the root cause rather than addressing symptoms.
- Debug for as long as needed to identify the root cause and identify a fix.
- Use `dbg!()` or `log::debug!()` to inspect state. Temporary `eprintln!()` is acceptable during debugging but must be removed before committing.
- Revisit your assumptions if unexpected behavior occurs.
- Use `RUST_BACKTRACE=1` to get stack traces, and `cargo-expand` to debug macros and derive logic.
- Read terminal output

> use `cargo fmt`, `cargo check`, `cargo clippy`,

## Memory Safety

Rust's ownership model, borrowing rules, and lifetimes enforce memory safety at compile time. Key patterns:
- Prefer stack allocation and borrowing over heap allocation and cloning.
- Use `Rc<RefCell<T>>` for single-threaded interior mutability; `Arc<Mutex<T>>` / `Arc<RwLock<T>>` for multi-threaded shared state.
- Avoid `RefCell` in `Send` types — it is not thread-safe.
- Circular references with `Rc`/`Arc` cause memory leaks; break cycles with `Weak<T>`.

# How to create a Todo List
Use the following format to create a todo list:
```markdown
- [ ] Step 1: Description of the first step
- [ ] Step 2: Description of the second step
- [ ] Step 3: Description of the third step
```
Status of each step should be indicated as follows:
- `[ ]` = Not started  
- `[x]` = Completed  
- `[-]` = Removed or no longer relevant

Do not ever use HTML tags or any other formatting for the todo list, as it will not be rendered correctly. Always use the markdown format shown above.


# Communication Guidelines
Communicate clearly and concisely in a professional tone. 

# Examples of Good Communication

<examples>
"Checking docs.rs for the `nom` 8.x combinator API before implementing the new grammar rule."
"Reading `src/grammar/shared.rs` to understand existing combinators before adding a new one."
"Tests pass. Running `cargo clippy --all-targets --locked` to check for warnings."
"Using `?` instead of `.unwrap()` here — this is library code and panics must be avoided."
"The parser returns `Option<Node>` — converting to `Result` so the caller can handle the error."
</examples>
