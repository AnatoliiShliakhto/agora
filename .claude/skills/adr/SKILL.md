---
name: adr
description: Record an architecture decision in docs/adr using the template and update the index
disable-model-invocation: true
---

# ADR

1. Next number: `ls docs/adr | tail -1`. File `docs/adr/NNNN-kebab-title.md` from
   `0000-template.md`.
2. Context: the forces, in the language of `docs/DOMAIN.md`/`docs/ARCHITECTURE.md`. Cite the
   plan task that triggered it.
3. Decision: what we do; concrete enough that a reviewer can check a PR against it.
4. Consequences: what gets easier, harder, what is given up. Include measured numbers when the
   decision was performance-driven.
5. Alternatives considered: each with the one reason it lost.
6. Status `accepted` once merged; a superseded ADR gets `Status: superseded by NNNN` and nothing
   else changes.
7. Add the row to `docs/adr/README.md`; reference the ADR from the code (`// See ADR-NNNN.`)
   where the decision is enforced.

Keep it under a page. An ADR is a note to a future engineer, not a design document.
