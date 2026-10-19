"""docs/queue.md read the way `just _queue-rows` reads it, for the Python half.

`_queue-rows` is the one place the recipes learn what a row is: a line inside a
fence, five fields and then the description (§230.2). The verbs and the merge
driver have to agree with it line for line, or a verb could write a row the fill
then reads differently. So this reads exactly what that awk reads: a fence is a
line starting with three backticks, a row is any line inside one holding
something other than blanks, and a heading is `## ` outside a fence.
"""

import re

TIERS_IN_ORDER = ["1", "2", "2.2", "2.5", "3", "4", "5"]
CONFLICT_MARKERS = ("<<<<<<<", "=======", ">>>>>>>")
TIER_HEADING = re.compile(r"^Tier (" + "|".join(re.escape(tier) for tier in TIERS_IN_ORDER) + r")( |$)")
# The first word of a description may be a tag (§352): a defect's class, or
# `small`, which a row of any tier may carry. The class decides an open row's
# tier: A and B are tier 1, C and D tier 5 (§403). A row Mattias pinned starts
# with `pinned/<n>` instead, and its tag, if it has one, comes second (§430).
CLASS = re.compile(r"^(wrong|crash|refuses)/[ABCD]$")
SMALL = "small"
PIN = re.compile(r"^pinned/([1-9][0-9]*)$")
WHAT = re.compile(r"^(\s*(?:\S+\s+){5})(\S+)(.*)$")
# A date in an `after:` state is a token, not a row id (§308): `after:Q148,2026-10-03`
# is ready once Q148 is done:, and that UTC date has come.
DATE = re.compile(r"^[0-9]{4}-[0-9]{2}-[0-9]{2}$")


def heading_key(heading):
    tier = TIER_HEADING.match(heading)
    if tier:
        return tier.group(1)
    if re.match(r"^Closed( |$)", heading):
        return "closed"
    return ""


def tier_of_class(tag):
    assert CLASS.match(tag), f"{tag!r} is not a class"
    return "5" if tag.endswith(("/C", "/D")) else "1"


# The queue page keeps lineage's summary of every commit on disk, and that
# summary is read through this: which lines are rows, a row's id and text, and
# which states close it. A change to any of those bumps `SUMMARY_SHAPE` in
# tooling/lineage/lineage.py, or the page goes on serving summaries read the
# old way (§444).
class Row:
    def __init__(self, index, text, block):
        self.index = index
        self.text = text
        self.block = block
        fields = text.split()
        self.fields = fields
        self.id = fields[0]
        self.tier = fields[1] if len(fields) > 1 else ""
        self.state = fields[2] if len(fields) > 2 else ""
        self.model = fields[3] if len(fields) > 3 else ""
        self.write_set = fields[4] if len(fields) > 4 else ""
        words = fields[5:]
        pinned = PIN.match(words[0]) if words else None
        self.pin = int(pinned.group(1)) if pinned else None
        if pinned:
            words = words[1:]
        first = words[0] if words else ""
        self.tag = first if CLASS.match(first) or first == SMALL else None

    def is_closed(self):
        return self.state.startswith(("done:", "dropped:"))

    def waits_on(self):
        """The ids an `after:` state names, less any date token, and none for any other state (§308)."""
        if not self.state.startswith("after:"):
            return []
        return [w for w in self.state[len("after:"):].split(",") if not DATE.match(w)]

    def waits_until(self):
        """The one date token an `after:` state names, or `None` (§308)."""
        if not self.state.startswith("after:"):
            return None
        dates = [w for w in self.state[len("after:"):].split(",") if DATE.match(w)]
        return dates[0] if dates else None

    def has_class(self):
        return self.tag is not None and CLASS.match(self.tag) is not None

    def with_state(self, state):
        match = re.match(r"^(\s*\S+\s+\S+\s+)(\S+)(.*)$", self.text)
        assert match, f"a row of fewer than three fields reached with_state: {self.text!r}"
        return match.group(1) + state + match.group(3)

    def with_tier(self, tier):
        """The row with its tier field replaced, the columns after it left where they stood."""
        match = re.match(r"^(\s*\S+\s+)(\S+)(\s+)(.*)$", self.text)
        assert match, f"a row of fewer than three fields reached with_tier: {self.text!r}"
        gap = max(1, len(match.group(2)) + len(match.group(3)) - len(tier))
        return match.group(1) + tier + " " * gap + match.group(4)

    def with_write_set(self, write_set):
        """The row with its write-set field replaced, the columns around it left where they stood."""
        match = re.match(r"^(\s*(?:\S+\s+){4})(\S+)(\s+)(.*)$", self.text)
        assert match, f"a row of fewer than five fields reached with_write_set: {self.text!r}"
        return match.group(1) + write_set + match.group(3) + match.group(4)

    def with_tag(self, tag):
        """The row with its tag replaced, added in front of the description, or removed when `tag` is None."""
        assert self.pin is None, f"tag reached the pinned row {self.id}, and only pin and unpin change one"
        match = WHAT.match(self.text)
        assert match, f"a row of fewer than six fields reached with_tag: {self.text!r}"
        head, first, rest = match.groups()
        if self.tag is None:
            return head + (f"{tag} " if tag else "") + first + rest
        if tag is None:
            return head + rest.lstrip()
        return head + tag + rest

    def with_pin(self, pin):
        """The row with `pinned/<pin>` in front of its description, or taken off it when `pin` is None."""
        match = WHAT.match(self.text)
        assert match, f"a row of fewer than six fields reached with_pin: {self.text!r}"
        head, first, rest = match.groups()
        if self.pin is None:
            assert pin is not None, f"unpin reached {self.id}, which is not pinned"
            return head + f"pinned/{pin} " + first + rest
        if pin is None:
            return head + rest.lstrip()
        return head + f"pinned/{pin}" + rest


class Queue:
    """The file as lines, with every row and fenced block located.

    A block is one fence pair, named by the heading above it and how many
    fences that heading has opened before it, so two fences under one heading
    stay two blocks rather than one.
    """

    def __init__(self, text):
        self.lines = text.split("\n")
        self.parse()

    def text(self):
        return "\n".join(self.lines)

    def parse(self):
        self.rows = []
        self.blocks = {}
        self.block_order = []
        self.unclosed = None
        self.conflicted = []
        self.headings = []
        head = ""
        opened_under = {}
        inside = None
        for index, line in enumerate(self.lines):
            if line.startswith("```"):
                if inside is None:
                    n = opened_under.get(head, 0)
                    opened_under[head] = n + 1
                    inside = (heading_key(head) or head, n)
                    self.blocks[inside] = [index, None]
                    self.block_order.append(inside)
                else:
                    self.blocks[inside][1] = index
                    inside = None
                continue
            if inside is not None:
                if line.strip() == "":
                    continue
                if line.startswith(CONFLICT_MARKERS):
                    self.conflicted.append(index)
                    continue
                self.rows.append(Row(index, line, inside))
                continue
            if line.startswith("## "):
                head = line[3:]
                self.headings.append((index, heading_key(head)))
        if inside is not None:
            self.unclosed = self.blocks[inside][0]

    def ids(self):
        return [row.id for row in self.rows]

    def find(self, row_id):
        found = [row for row in self.rows if row.id == row_id]
        assert len(found) <= 1, f"{row_id} is a row {len(found)} times; check-queue refuses that before any verb runs"
        return found[0] if found else None

    def block_of_key(self, key):
        for block in self.block_order:
            if block[0] == key:
                return block
        return None

    def block_for_tier(self, tier):
        """The block under `## Tier <tier>`, written first, empty, if the file has none.

        A tier with no rows may leave its heading out (§307.3), so a verb
        putting a row in one writes it. It goes just above the first heading
        that comes after it in TIERS_IN_ORDER, or above `## Closed`, which keeps
        the order `check-queue` holds. None when the file has neither.
        """
        assert tier in TIERS_IN_ORDER, f"{tier!r} is not a tier; the verb checks that before asking"
        block = self.block_of_key(tier)
        if block is not None:
            return block
        later = set(TIERS_IN_ORDER[TIERS_IN_ORDER.index(tier) + 1:]) | {"closed"}
        above = next((index for index, key in self.headings if key in later), None)
        if above is None:
            return None
        self.lines[above:above] = [f"## Tier {tier}", "", "```", "```", ""]
        self.parse()
        return self.block_of_key(tier)

    def replace(self, row, text):
        self.lines[row.index] = text
        self.parse()

    def remove(self, row):
        del self.lines[row.index]
        self.parse()

    def move_before(self, row, other):
        """Move `row`'s line to sit just before `other`'s, within the same block."""
        line = self.lines[row.index]
        del self.lines[row.index]
        other_index = other.index if other.index < row.index else other.index - 1
        self.lines.insert(other_index, line)
        self.parse()

    def move_after(self, row, other):
        """Move `row`'s line to sit just after `other`'s, within the same block."""
        line = self.lines[row.index]
        del self.lines[row.index]
        other_index = other.index if other.index < row.index else other.index - 1
        self.lines.insert(other_index + 1, line)
        self.parse()

    def append_to_block(self, block, text):
        close = self.blocks[block][1]
        assert close is not None, f"the block {block} is never closed"
        self.lines.insert(close, text)
        self.parse()


def highest_q(text):
    top = 0
    for number in re.findall(r"^\s*Q([0-9]+)\s", text, flags=re.M):
        top = max(top, int(number))
    return top
