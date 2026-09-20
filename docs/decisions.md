# Design decisions

The decisions live in `docs/decisions.d/`, one file per section, named by the
number it is cited as. Citations keep saying `docs/decisions.md §N` — the number
is the address either way, and respelling several hundred of them would have
collided with every session in flight to no reader's benefit. §179 is why.

    just decisions-toc        the map: every section and what it costs to read
    just decision 49          §49's prose and the list of its subsections
    just decision 49.6        one subsection in full
    just decisions-find ...   which sections mention this, in section numbers
    just decisions-all        every section concatenated, as this file used to be
    just claim-section '...'  take the next number before you write the section

**Do not append a section here.** A `## N.` heading in this file is addressed by
nothing and reaches no reader; write `docs/decisions.d/NNN-<slug>.md` instead.
`just check-decisions` fails on one, and it is in `just gate`.
