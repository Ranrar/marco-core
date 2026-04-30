---
name: Bug report
about: Report a parser, renderer, or intelligence bug in marco-core
title: ''
labels: bug
assignees: Ranrar

---

**Describe the bug**
A clear and concise description of what the bug is (parse error, wrong AST, wrong HTML, wrong highlight/diagnostic, panic, etc.).

**Minimal Markdown input**
The smallest possible Markdown snippet that reproduces the issue.

```markdown
<!-- paste input here -->
```

**Expected output**
What `marco_core::parse` / `marco_core::render` (or the relevant intelligence function) should produce.

```text
<!-- expected AST debug, expected HTML, or expected highlights/diagnostics -->
```

**Actual output**
What you actually get.

```text
<!-- actual output, error message, or panic backtrace -->
```

**Reproducer (optional)**
A short Rust snippet calling the public API:

```rust
use marco_core::{parse, render, RenderOptions};

let doc = parse("...").unwrap();
let html = render(&doc, &RenderOptions::default()).unwrap();
println!("{html}");
```

**Environment**
- `marco-core` version: [e.g. 1.0.2]
- Rust version (`rustc --version`): [e.g. 1.94.1]
- OS: [e.g. Linux x86_64, Windows 11]

**Additional context**
Any other context, links to spec sections (CommonMark / GFM), or related issues.
