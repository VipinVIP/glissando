#!/usr/bin/env python3
"""End-to-end tests: real Wayland compositor (headless sway), the real
keyboard binary, a text-input-v3 test client and a virtual pointer.

Every test starts a private session (own XDG dirs, runtime dir and D-Bus via
dbus-run-session) so nothing touches the developer's desktop.

    tests/e2e/e2e.py [-k NAME] [--keep] [--shots DIR] [--release]
"""
import argparse, json, os, shutil, subprocess, sys, tempfile, time, traceback

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
SWAY = os.path.join(ROOT, ".devtools/root/usr/bin/sway")
GRIM = os.path.join(ROOT, ".devtools/root/usr/bin/grim")
OUT_W, OUT_H = 360, 780


class Session:
    def __init__(self, args, purpose=None, hint=None, config=None, kb_args=()):
        self.args = args
        self.tmp = tempfile.mkdtemp(prefix="glissando-e2e-")
        self.env = dict(os.environ)
        rt = os.path.join(self.tmp, "rt")
        os.mkdir(rt, 0o700)
        for k in ["WAYLAND_DISPLAY", "DISPLAY", "SWAYSOCK"]:
            self.env.pop(k, None)
        self.env.update({
            "XDG_RUNTIME_DIR": rt,
            "XDG_CONFIG_HOME": os.path.join(self.tmp, "config"),
            "XDG_DATA_HOME": os.path.join(self.tmp, "data"),
            "XDG_STATE_HOME": os.path.join(self.tmp, "state"),
            "GLISSANDO_DATA_DIR": os.path.join(ROOT, "data"),
            "GLISSANDO_AUTOSHOW": "1",
            "WLR_BACKENDS": "headless",
            "WLR_LIBINPUT_NO_DEVICES": "1",
            "WLR_RENDERER": "pixman",
            "GLISSANDO_TK_EXTENT": f"{OUT_W}x{OUT_H}",
        })
        if config:
            os.makedirs(self.env["XDG_CONFIG_HOME"] + "/glissando", exist_ok=True)
            with open(self.env["XDG_CONFIG_HOME"] + "/glissando/config.toml", "w") as f:
                f.write(config)
        self.procs = []
        prof = "release" if args.release else "debug"
        self.kb_bin = os.path.join(ROOT, "target", prof, "glissando")
        self.tk_bin = os.path.join(ROOT, "target", prof, "glissando-testkit")
        self.geom_file = os.path.join(self.tmp, "geom.json")
        self.state_file = os.path.join(self.tmp, "state.json")
        self.spawn("sway", [SWAY, "-c", os.path.join(ROOT, "tools/devenv/sway-test.conf")])
        for _ in range(100):
            socks = [s for s in os.listdir(rt) if s.startswith("wayland-") and not s.endswith(".lock")]
            if socks:
                self.env["WAYLAND_DISPLAY"] = socks[0]
                break
            time.sleep(0.05)
        else:
            raise RuntimeError("sway did not start: " + self.log("sway"))
        self.spawn("kb", [self.kb_bin, "--no-dbus", "--dump-geometry", self.geom_file, *kb_args],
                   extra_env={"GLISSANDO_DEBUG": "1",
                              **({"GLISSANDO_DEBUG_INPUT": "1"} if os.environ.get("E2E_INPUT") else {}),
                              **({"LD_PRELOAD": os.environ["E2E_KB_PRELOAD"]} if os.environ.get("E2E_KB_PRELOAD") else {})})
        time.sleep(0.4)
        tc = [self.tk_bin, "textclient", "--state", self.state_file]
        if purpose:
            tc += ["--purpose", purpose]
        if hint:
            tc += ["--hint", hint]
        self.spawn("tc", tc)
        self.wait(lambda: os.path.exists(self.geom_file) and self.surface_height() > 0, "keyboard surface")
        time.sleep(0.3)

    def spawn(self, name, cmd, extra_env=None):
        env = dict(self.env)
        env.update(extra_env or {})
        log = open(os.path.join(self.tmp, name + ".log"), "w")
        p = subprocess.Popen(cmd, env=env, stdout=log, stderr=subprocess.STDOUT)
        self.procs.append((name, p))
        return p

    def log(self, name):
        try:
            return open(os.path.join(self.tmp, name + ".log")).read()
        except OSError:
            return ""

    def wait(self, cond, what, timeout=6.0):
        t = time.time()
        while time.time() - t < timeout:
            try:
                if cond():
                    return
            except (OSError, ValueError):
                pass
            time.sleep(0.05)
        raise AssertionError(f"timeout waiting for {what}\n--- kb log ---\n{self.log('kb')[-2000:]}")

    def geometry(self):
        with open(self.geom_file) as f:
            return json.load(f)

    def surface_height(self):
        for line in self.log("kb").splitlines()[::-1]:
            if "surface created, height" in line:
                return int(line.rsplit(" ", 1)[1])
        return 0

    def kb_height(self):
        g = self.geometry()
        return max(k["y"] + k["h"] / 2 for k in g) + 5

    def key_pos(self, name, nth=0):
        g = self.geometry()
        matches = [k for k in g if k["key"] == name]
        if not matches:
            raise KeyError(f"no key {name!r} in {[k['key'] for k in g]}")
        k = matches[nth]
        top = OUT_H - self.current_height()
        return k["x"], top + k["y"]

    def current_height(self):
        h = 0
        for line in self.log("kb").splitlines():
            if line.startswith("glissando: configure "):
                dims = line.split()[-1]
                hh = int(dims.split("x")[1])
                if hh:
                    h = hh
            if "surface created, height" in line:
                h = int(line.rsplit(" ", 1)[1])
        return h

    def state(self):
        with open(self.state_file) as f:
            return json.load(f)

    def pointer(self, script):
        subprocess.run([self.tk_bin, "pointer", script], env=self.env, check=True, timeout=30)
        time.sleep(0.15)

    def tap(self, key, nth=0):
        x, y = self.key_pos(key, nth)
        self.pointer(f"tap {x} {y}")

    def type(self, text):
        """Tap the keys for `text` (lowercase letters, space, punctuation)."""
        for ch in text:
            name = {" ": "{space}", "\n": "{enter}", "\b": "{backspace}"}.get(ch, ch)
            self.tap(name)

    def screenshot(self, name):
        if self.args.shots:
            os.makedirs(self.args.shots, exist_ok=True)
            subprocess.run([GRIM, os.path.join(self.args.shots, name + ".png")], env=self.env, timeout=10)

    def close(self):
        for name, p in reversed(self.procs):
            p.terminate()
            try:
                p.wait(timeout=3)
            except subprocess.TimeoutExpired:
                p.kill()
        if not self.args.keep:
            shutil.rmtree(self.tmp, ignore_errors=True)
        else:
            print("kept", self.tmp)


TESTS = []


def test(fn):
    TESTS.append(fn)
    return fn


@test
def keyboard_appears_on_focus(s):
    assert s.current_height() > 150, s.current_height()
    names = [k["key"] for k in s.geometry()]
    assert "q" in names and "{space}" in names, names
    s.screenshot("appears")


@test
def typing_letters(s):
    s.type("hello")
    s.wait(lambda: (s.state()["text"] + s.state()["preedit"]) == "Hello", "Hello typed")
    s.type(" ")
    s.wait(lambda: s.state()["text"] == "Hello " and s.state()["preedit"] == "", "committed with space")
    s.screenshot("typed")


@test
def backspace_and_symbols(s):
    s.type("ab")
    s.tap("{backspace}")
    s.wait(lambda: s.state()["preedit"] == "A", "backspace in composing")
    s.tap("{layer=symbols}")
    s.wait(lambda: any(k["key"] == "@" for k in s.geometry()), "symbols layer")
    s.tap("@")
    s.wait(lambda: s.state()["text"] == "A@", "symbol committed")
    s.screenshot("symbols")


@test
def glide_typing(s):
    pts = []
    for ch in "hello":
        x, y = s.key_pos(ch)
        pts += [x, y]
    pts_s = " ".join(f"{v:.1f}" for v in pts)
    s.pointer(f"swipe 450 {pts_s}")
    s.wait(lambda: (s.state()["text"] + s.state()["preedit"]).lower() == "hello", "glided word", timeout=8)
    s.screenshot("glide")


@test
def password_field_is_private(s):
    pass


def run_password(args):
    s = Session(args, purpose="password")
    try:
        s.type("helo ")
        s.wait(lambda: s.state()["text"] == "helo ", "password typed verbatim")
        assert s.state()["preedit"] == ""
        user_dir = os.path.join(s.env["XDG_DATA_HOME"], "glissando", "user")
        time.sleep(0.3)
        assert not os.path.exists(user_dir) or all(os.path.getsize(os.path.join(user_dir, f)) == 0 for f in os.listdir(user_dir))
    finally:
        s.close()


@test
def space_cursor_and_glide_delete(s):
    s.type("we go ")
    s.wait(lambda: s.state()["text"] == "We go ", "typed")
    x, y = s.key_pos("{space}")
    s.pointer(f"swipe 300 {x} {y} {x-60} {y}")
    s.wait(lambda: s.state()["cursor"] < 6, "cursor moved left")
    x, y = s.key_pos("{space}")
    s.pointer(f"swipe 300 {x-60} {y} {x+140} {y}")
    s.wait(lambda: s.state()["cursor"] == 6, "cursor back at end")
    bx, by = s.key_pos("{backspace}")
    s.pointer(f"swipe 300 {bx} {by} {bx-40} {by}")
    s.wait(lambda: s.state()["text"] == "We ", "glide delete removed a word")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("-k", default="")
    ap.add_argument("--keep", action="store_true")
    ap.add_argument("--shots", default=None)
    ap.add_argument("--release", action="store_true")
    args = ap.parse_args()
    if not os.path.exists(SWAY):
        print("sway not found; run tools/devenv/fetch-sway.sh", file=sys.stderr)
        return 2
    failed = 0
    for t in TESTS:
        if args.k not in t.__name__:
            continue
        start = time.time()
        try:
            if t.__name__ == "password_field_is_private":
                run_password(args)
            else:
                s = Session(args)
                try:
                    t(s)
                finally:
                    s.close()
            print(f"PASS {t.__name__} ({time.time()-start:.1f}s)")
        except Exception:
            failed += 1
            print(f"FAIL {t.__name__}")
            traceback.print_exc()
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
