---
paths:
  - docs/**
  - README.md
---

# Documentation Instructions

Conventions for writing `README.md` and the pages under `docs/`. They are
plain Markdown read on GitHub: no site generator, no front-matter beyond what
the page declaration needs. Mechanical checks live in `docs-quality.md`; this
rule carries the voice.

---

## Narrative Structure

Every `##` section opens with two or three short paragraphs that build in
sequence:

1. **The idea** — what concept does this section cover? One-sentence frame.
2. **The problem** — why does it matter? Show concrete real-world pain, not
   hypothetical. Use examples from tools the reader knows.
3. **The solution** — how does grim-mirror address it? Short, direct.

Then subsections for depth and design decisions. No sales pitch or marketing
opener; let examples make the case.

---

## Paragraph Style

- **Short paragraphs.** One idea per paragraph, especially section intros.
- **No stop-and-go.** Each paragraph leads to the next.
- **Tables and code blocks follow prose** — set context first.
- **No command dumps** without explaining what they represent.

---

## Headers

Short headers that read as chapter titles in a rendered outline.
Bad: `### Automatic Detection of Behavior`. Good: `### Auto-Detection`.

---

## Real-World Examples and External Links

Do not describe other tools from memory; read their docs first. Use concrete
command sequences, real filenames and real repository links, not abstract
descriptions. Every external tool mentioned links to its source, every
occurrence.

---

## Precision and Nuance

Be exactly correct. State the precise behavior, not a convenient
simplification. Where a behavior is a convention rather than enforced, say so.
Read the source before documenting a behavior; never write from memory.

---

## Link Syntax

Use reference-style links, never inline `[text](url)` in the body. Collect the
definitions at the bottom of the file, grouped by comments (`<!-- external -->`,
`<!-- internal -->`). Every reference to another part of the system links, and
only to a section that exists and has content.
