import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { copyFileSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import test from "node:test";

const root = resolve(import.meta.dirname, "..");

function fixture(run) {
  const temporaryRoot = resolve(tmpdir());
  const directory = mkdtempSync(join(temporaryRoot, "kraqel-hooks-"));
  const checkout = join(directory, "checkout with spaces");
  const config = join(directory, "empty.gitconfig");
  writeFileSync(config, "");
  mkdirSync(join(checkout, "scripts"), { recursive: true });
  mkdirSync(join(checkout, ".githooks"));
  copyFileSync(
    join(root, "scripts", "install-git-hooks.mjs"),
    join(checkout, "scripts", "install-git-hooks.mjs"),
  );
  copyFileSync(join(root, "scripts", "pre-push.mjs"), join(checkout, "scripts", "pre-push.mjs"));
  copyFileSync(join(root, ".githooks", "pre-push"), join(checkout, ".githooks", "pre-push"));
  const env = { ...process.env, CI: "", GIT_CONFIG_GLOBAL: config, GIT_CONFIG_NOSYSTEM: "1" };
  function command(executable, args, extra = {}) {
    const result = spawnSync(executable, args, { cwd: checkout, encoding: "utf8", env, ...extra });
    assert.ifError(result.error);
    return result;
  }
  function git(args) {
    const result = command("git", args);
    assert.equal(result.status, 0, result.stderr);
    return result.stdout.trim();
  }
  function install(...args) {
    return command(process.execPath, ["scripts/install-git-hooks.mjs", ...args]);
  }
  try {
    git(["init", "-q", "--initial-branch=main"]);
    return run({ checkout, directory, env, command, git, install });
  } finally {
    assert.equal(dirname(directory), temporaryRoot);
    rmSync(directory, { recursive: true, force: true });
  }
}

test("pre-push runs the check and blocks a failed check before changing the remote", () => {
  fixture(({ checkout, directory, env, command, git, install }) => {
    assert.equal(install().status, 0);
    assert.equal(install().status, 0, "installation is idempotent");
    assert.equal(git(["config", "--local", "--get", "core.hooksPath"]), ".githooks");
    git(["add", "."]);
    git([
      "-c",
      "user.name=Hook Test",
      "-c",
      "user.email=hook@example.invalid",
      "commit",
      "-qm",
      "fixture",
    ]);
    const remote = join(directory, "remote.git");
    git(["init", "--bare", "-q", remote]);
    git(["remote", "add", "origin", remote]);

    const bin = join(directory, "bin");
    const log = join(directory, "check.log");
    mkdirSync(bin);
    writeFileSync(
      join(bin, process.platform === "win32" ? "pnpm.cmd" : "pnpm"),
      process.platform === "win32"
        ? '@echo off\r\n> "%KRAQEL_HOOK_TEST_LOG%" echo %*\r\nexit /b %KRAQEL_HOOK_TEST_EXIT%\r\n'
        : '#!/bin/sh\nprintf "%s\\n" "$*" > "$KRAQEL_HOOK_TEST_LOG"\nexit "$KRAQEL_HOOK_TEST_EXIT"\n',
      { mode: 0o755 },
    );
    const pathKey = Object.keys(env).find((key) => key.toLowerCase() === "path") ?? "PATH";
    const testEnv = {
      ...env,
      [pathKey]: `${bin}${process.platform === "win32" ? ";" : ":"}${env[pathKey] ?? ""}`,
      KRAQEL_HOOK_TEST_LOG: log,
      KRAQEL_HOOK_TEST_EXIT: "17",
    };
    const push = () =>
      command("git", ["push", "origin", "HEAD:refs/heads/main"], {
        cwd: join(checkout, "scripts"),
        env: testEnv,
      });
    const failed = push();
    assert.notEqual(failed.status, 0, failed.stderr);
    assert.equal(readFileSync(log, "utf8").trim(), "check");
    assert.equal(git(["ls-remote", "origin", "refs/heads/main"]), "");
    testEnv.KRAQEL_HOOK_TEST_EXIT = "0";
    const passed = push();
    assert.equal(passed.status, 0, passed.stderr);
    assert.equal(
      git(["ls-remote", "origin", "refs/heads/main"]).split(/\s/)[0],
      git(["rev-parse", "HEAD"]),
    );
  });
});

test("installer preserves custom hook paths and automatic setup only warns", () => {
  fixture(({ git, install }) => {
    git(["config", "--local", "core.hooksPath", "custom-hooks"]);
    assert.equal(install().status, 1);
    assert.equal(install("--auto").status, 0);
    assert.equal(git(["config", "--local", "--get", "core.hooksPath"]), "custom-hooks");
  });
});

test("installer preserves existing hooks in the default directory", () => {
  fixture(({ checkout, command, install }) => {
    const hook = join(checkout, ".git", "hooks", "pre-push");
    writeFileSync(hook, "existing hook\n");
    assert.equal(install().status, 1);
    assert.equal(readFileSync(hook, "utf8"), "existing hook\n");
    assert.equal(command("git", ["config", "--local", "--get", "core.hooksPath"]).status, 1);
  });
});

test("automatic setup leaves CI configuration untouched", () => {
  fixture(({ command, env }) => {
    const result = command(process.execPath, ["scripts/install-git-hooks.mjs", "--auto"], {
      env: { ...env, CI: "true" },
    });
    assert.equal(result.status, 0);
    assert.equal(command("git", ["config", "--local", "--get", "core.hooksPath"]).status, 1);
  });
});
