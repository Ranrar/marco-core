# Security Policy

## Supported versions

Security fixes are issued for the latest minor release on the `1.x` series of
`marco-core`. Older minor versions receive fixes only at the maintainer's
discretion.

| Version  | Supported          |
| -------- | ------------------ |
| `1.x`    | :white_check_mark: |
| `< 1.0`  | :x:                |

## Reporting a vulnerability

Please **do not** open public GitHub issues for security problems.

Report vulnerabilities privately via GitHub's
[Security Advisories](https://github.com/Ranrar/marco-core/security/advisories/new)
form on this repository. Include:

- the affected version(s) of `marco-core`,
- a minimal Markdown input that reproduces the issue,
- the resulting HTML / panic / behavior,
- your assessment of impact (e.g. XSS via rendered HTML, denial of service
  through pathological input, infinite recursion, panics in library code).

You should receive an acknowledgement within a few business days. Once the
issue is confirmed, a fix and coordinated disclosure timeline will be agreed
in the advisory thread before any public commit or release.

## Scope

In scope:

- The Markdown parser (`marco_core::parse`).
- The HTML renderer (`marco_core::render`) — including how it escapes
  user-controlled URLs, attributes, and inline HTML.
- The intelligence layer (highlights / diagnostics / completions / hover).
- Input sanitization (`marco_core::sanitize_input`).

Out of scope:

- Vulnerabilities in third-party renderers (KaTeX, Mermaid, syntect) reached
  through their published APIs — please report those upstream.
- Issues only reproducible in the consuming editor
  ([Marco](https://github.com/Ranrar/Marco)) — report those in the editor
  repository.
