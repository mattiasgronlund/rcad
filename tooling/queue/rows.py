"""docs/queue.md read the way `just _queue-rows` reads it, for the Python half.

`_queue-rows` is the one place the recipes learn what a row is: a line inside a
fence, five fields and then the description (§230.2). The verbs and the merge
driver have to agree with it line for line, or a verb could write a row the fill
then reads differently. So this reads exactly what that awk reads: a fence is a
line starting with three backticks, a row is any line inside one holding
something other than blanks, and a heading is `## ` outside a fence.
"""

import re

TIERS = ["1", "2", "2.5", "3", "4"]
CONFLICT_MARKERS = ("<<<<<<<", "=======", ">>>>>>>")


def heading_key(heading):
    tier = re.match(r"^Tier (1|2|2\.5|3|4)( |$)", heading)
    if tier:
        return tier.group(1)
    if re.match(r"^Closed( |$)", heading):
        return "closed"
    return ""


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

    def is_closed(self):
        return self.state.startswith(("done:", "dropped:"))

    def with_state(self, state):
        match = re.match(r"^(\s*\S+\s+\S+\s+)(\S+)(.*)$", self.text)
        assert match, f"a row of fewer than three fields reached with_state: {self.text!r}"
        return match.group(1) + state + match.group(3)


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

    def replace(self, row, text):
        self.lines[row.index] = text
        self.parse()

    def remove(self, row):
        del self.lines[row.index]
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
