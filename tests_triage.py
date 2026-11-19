import tempfile
import unittest
from contextlib import redirect_stdout
from io import StringIO
from pathlib import Path

import triage

OLD = "## a\nkind: issue\nflavor: xp\nseverity: high\nstatus: new\n"
NEW = "## b\nkind: ux\nflavor: xp\nseverity: high\ntask: X1\ncheck: I3\nstatus: new\n"
NOCHK = (
    "## c\nkind: ux\nflavor: mc\nseverity: high\ntask: M1\ncheck: none\nstatus: new\n"
)


class TriageTest(unittest.TestCase):
    def run_triage(self, files, argv=()):
        with tempfile.TemporaryDirectory() as d:
            for name, text in files.items():
                (Path(d) / name).write_text(text)
            old, triage.INBOX = triage.INBOX, Path(d)
            out = StringIO()
            try:
                with redirect_stdout(out):
                    code = triage.main(list(argv))
            finally:
                triage.INBOX = old
        return code, out.getvalue()

    def test_old_file_parses(self):
        f = next(iter(self._parse(OLD)))
        self.assertEqual((f["task"], f["check"]), ("none", "none"))

    def _parse(self, text):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d) / "x.md"
            p.write_text(text)
            return list(triage.findings(p))

    def test_grouping(self):
        _, out = self.run_triage(
            {"2026-11-09-r1-a.md": OLD + NEW + NEW.replace("## b", "## b2")}
        )
        self.assertIn("== xp / I3 (2)", out)
        self.assertIn("== xp / none (1)", out)

    def test_high_no_check(self):
        _, out = self.run_triage({"2026-11-09-r1-a.md": NOCHK + NEW})
        self.assertEqual(out.count("HIGH, NO CHECK"), 1)

    def test_format_pass_and_fail(self):
        code, _ = self.run_triage({"2026-11-20-r1-a.md": NEW}, ["--format"])
        self.assertEqual(code, 0)
        code, _ = self.run_triage({"2026-11-20-r1-a.md": OLD}, ["--format"])
        self.assertEqual(code, 1)
        code, _ = self.run_triage({"2026-11-19-r1-a.md": OLD}, ["--format"])
        self.assertEqual(code, 0)


if __name__ == "__main__":
    unittest.main()
