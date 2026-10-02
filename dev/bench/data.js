window.BENCHMARK_DATA = {
  "lastUpdate": 1790983165377,
  "repoUrl": "https://github.com/Faicad/brepkit2",
  "entries": {
    "Boolean perf": [
      {
        "commit": {
          "author": {
            "email": "yuan_xin_yu@hotmail.com",
            "name": "Ylt",
            "username": "yuan-xy"
          },
          "committer": {
            "email": "yuan_xin_yu@hotmail.com",
            "name": "Ylt",
            "username": "yuan-xy"
          },
          "distinct": true,
          "id": "35f1a9c22a12f3cded62b00eb8f700326ac2a9f4",
          "message": "chore: fork brepkit v2.129.15 (MIT OR Apache-2.0)",
          "timestamp": "2026-09-24T11:22:05+08:00",
          "tree_id": "2e3eb92346b9432c68f6a6371dcf6580a2f21eba",
          "url": "https://github.com/Faicad/brepkit2/commit/35f1a9c22a12f3cded62b00eb8f700326ac2a9f4"
        },
        "date": 1790760056662,
        "tool": "cargo",
        "benches": [
          {
            "name": "boolean/cut_box_box",
            "value": 543089,
            "range": "± 737",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/fuse_box_box",
            "value": 603744,
            "range": "± 3798",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/intersect_box_box",
            "value": 7385,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/cut_cylinder_through_box",
            "value": 401496,
            "range": "± 1332",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/perforated_cut_36",
            "value": 15818088,
            "range": "± 34716",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "yuan_xin_yu@hotmail.com",
            "name": "Ylt",
            "username": "yuan-xy"
          },
          "committer": {
            "email": "yuan_xin_yu@hotmail.com",
            "name": "Ylt",
            "username": "yuan-xy"
          },
          "distinct": true,
          "id": "328ee9d237034335ca2cd40383c3d265c4d414df",
          "message": "docs: use brepkit2 naming in skills and guides",
          "timestamp": "2026-09-30T18:20:56+08:00",
          "tree_id": "5c9ae1c2453894afc69870012d326e5305befc8f",
          "url": "https://github.com/Faicad/brepkit2/commit/328ee9d237034335ca2cd40383c3d265c4d414df"
        },
        "date": 1790763821837,
        "tool": "cargo",
        "benches": [
          {
            "name": "boolean/cut_box_box",
            "value": 941863,
            "range": "± 7373",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/fuse_box_box",
            "value": 1022500,
            "range": "± 971",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/intersect_box_box",
            "value": 11822,
            "range": "± 81",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/cut_cylinder_through_box",
            "value": 696296,
            "range": "± 29338",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/perforated_cut_36",
            "value": 24850959,
            "range": "± 64504",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "yuan_xin_yu@hotmail.com",
            "name": "Ylt",
            "username": "yuan-xy"
          },
          "committer": {
            "email": "yuan_xin_yu@hotmail.com",
            "name": "Ylt",
            "username": "yuan-xy"
          },
          "distinct": true,
          "id": "3d55935634e94b2630812e06f0dde2f278944886",
          "message": "docs: rename CLAUDE.md to AGENTS.md and update all references\n\nAdopt the AGENTS.md standard filename for project instructions.\nUpdate doc-path checker, verify scripts, hooks, skills, and i18n\npairing record accordingly. No content changes to the instructions.",
          "timestamp": "2026-09-30T19:02:42+08:00",
          "tree_id": "e3c9177358568d0cca23135b132a4aafbaf7dea5",
          "url": "https://github.com/Faicad/brepkit2/commit/3d55935634e94b2630812e06f0dde2f278944886"
        },
        "date": 1790766355822,
        "tool": "cargo",
        "benches": [
          {
            "name": "boolean/cut_box_box",
            "value": 947458,
            "range": "± 1388",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/fuse_box_box",
            "value": 1029325,
            "range": "± 1985",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/intersect_box_box",
            "value": 12117,
            "range": "± 50",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/cut_cylinder_through_box",
            "value": 698908,
            "range": "± 973",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/perforated_cut_36",
            "value": 25249721,
            "range": "± 306022",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "yuan_xin_yu@hotmail.com",
            "name": "Ylt",
            "username": "yuan-xy"
          },
          "committer": {
            "email": "yuan_xin_yu@hotmail.com",
            "name": "Ylt",
            "username": "yuan-xy"
          },
          "distinct": true,
          "id": "3963ef381707a39e18a444a82f49063910afd6fa",
          "message": "fix(ci): restore executable bit on shell scripts lost in fork import\n\nCI jobs Layer Boundaries and Doc Paths failed with exit 126\n(Permission denied) because scripts/*.sh were committed as 100644.\nRestore 100755 on all six shell scripts.",
          "timestamp": "2026-09-30T19:22:56+08:00",
          "tree_id": "6ca08b89df0d0375d8dae52ae5fe7eb267690dbe",
          "url": "https://github.com/Faicad/brepkit2/commit/3963ef381707a39e18a444a82f49063910afd6fa"
        },
        "date": 1790767510740,
        "tool": "cargo",
        "benches": [
          {
            "name": "boolean/cut_box_box",
            "value": 794893,
            "range": "± 1345",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/fuse_box_box",
            "value": 832072,
            "range": "± 9126",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/intersect_box_box",
            "value": 10182,
            "range": "± 29",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/cut_cylinder_through_box",
            "value": 549235,
            "range": "± 6197",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/perforated_cut_36",
            "value": 20869876,
            "range": "± 467601",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "yuan_xin_yu@hotmail.com",
            "name": "yuan",
            "username": "yuan-xy"
          },
          "distinct": true,
          "id": "16b732c31fe35494f9cfa518fe8c484a3c94178e",
          "message": "chore(deps-dev): bump the npm group across 1 directory with 5 updates\n\nBumps the npm group with 5 updates in the / directory:\n\n| Package | From | To |\n| --- | --- | --- |\n| [@commitlint/cli](https://github.com/conventional-changelog/commitlint/tree/HEAD/@commitlint/cli) | `21.2.1` | `21.2.3` |\n| [@commitlint/config-conventional](https://github.com/conventional-changelog/commitlint/tree/HEAD/@commitlint/config-conventional) | `21.2.0` | `21.2.3` |\n| [jsdom](https://github.com/jsdom/jsdom) | `29.1.1` | `30.0.1` |\n| [prettier](https://github.com/prettier/prettier) | `3.9.6` | `3.9.9` |\n| [typescript](https://github.com/microsoft/TypeScript) | `5.9.3` | `7.0.2` |\n\n\n\nUpdates `@commitlint/cli` from 21.2.1 to 21.2.3\n- [Release notes](https://github.com/conventional-changelog/commitlint/releases)\n- [Changelog](https://github.com/conventional-changelog/commitlint/blob/master/@commitlint/cli/CHANGELOG.md)\n- [Commits](https://github.com/conventional-changelog/commitlint/commits/v21.2.3/@commitlint/cli)\n\nUpdates `@commitlint/config-conventional` from 21.2.0 to 21.2.3\n- [Release notes](https://github.com/conventional-changelog/commitlint/releases)\n- [Changelog](https://github.com/conventional-changelog/commitlint/blob/master/@commitlint/config-conventional/CHANGELOG.md)\n- [Commits](https://github.com/conventional-changelog/commitlint/commits/v21.2.3/@commitlint/config-conventional)\n\nUpdates `jsdom` from 29.1.1 to 30.0.1\n- [Release notes](https://github.com/jsdom/jsdom/releases)\n- [Commits](https://github.com/jsdom/jsdom/compare/v29.1.1...v30.0.1)\n\nUpdates `prettier` from 3.9.6 to 3.9.9\n- [Release notes](https://github.com/prettier/prettier/releases)\n- [Changelog](https://github.com/prettier/prettier/blob/main/CHANGELOG.md)\n- [Commits](https://github.com/prettier/prettier/compare/3.9.6...3.9.9)\n\nUpdates `typescript` from 5.9.3 to 7.0.2\n- [Release notes](https://github.com/microsoft/TypeScript/releases)\n- [Commits](https://github.com/microsoft/TypeScript/compare/v5.9.3...v7.0.2)\n\n---\nupdated-dependencies:\n- dependency-name: \"@commitlint/cli\"\n  dependency-version: 21.2.3\n  dependency-type: direct:development\n  update-type: version-update:semver-patch\n  dependency-group: npm\n- dependency-name: \"@commitlint/config-conventional\"\n  dependency-version: 21.2.3\n  dependency-type: direct:development\n  update-type: version-update:semver-patch\n  dependency-group: npm\n- dependency-name: jsdom\n  dependency-version: 30.0.1\n  dependency-type: direct:development\n  update-type: version-update:semver-major\n  dependency-group: npm\n- dependency-name: prettier\n  dependency-version: 3.9.9\n  dependency-type: direct:development\n  update-type: version-update:semver-patch\n  dependency-group: npm\n- dependency-name: typescript\n  dependency-version: 7.0.2\n  dependency-type: direct:development\n  update-type: version-update:semver-major\n  dependency-group: npm\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>",
          "timestamp": "2026-09-30T19:38:40+08:00",
          "tree_id": "074d5ab08db9a9ea1e7f8f10e4ded2a507244976",
          "url": "https://github.com/Faicad/brepkit2/commit/16b732c31fe35494f9cfa518fe8c484a3c94178e"
        },
        "date": 1790768461697,
        "tool": "cargo",
        "benches": [
          {
            "name": "boolean/cut_box_box",
            "value": 1029023,
            "range": "± 1557",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/fuse_box_box",
            "value": 1076984,
            "range": "± 2616",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/intersect_box_box",
            "value": 12890,
            "range": "± 305",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/cut_cylinder_through_box",
            "value": 714342,
            "range": "± 4761",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/perforated_cut_36",
            "value": 26126893,
            "range": "± 757624",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "yuan_xin_yu@hotmail.com",
            "name": "yuan",
            "username": "yuan-xy"
          },
          "distinct": true,
          "id": "9dd6a50d1f50059958c9122eed581199684fbcf6",
          "message": "chore(deps): update wasm-bindgen requirement from =0.2.126 to =0.2.128\n\nUpdates the requirements on [wasm-bindgen](https://github.com/wasm-bindgen/wasm-bindgen) to permit the latest version.\n- [Release notes](https://github.com/wasm-bindgen/wasm-bindgen/releases)\n- [Changelog](https://github.com/wasm-bindgen/wasm-bindgen/blob/main/CHANGELOG.md)\n- [Commits](https://github.com/wasm-bindgen/wasm-bindgen/compare/0.2.126...0.2.128)\n\n---\nupdated-dependencies:\n- dependency-name: wasm-bindgen\n  dependency-version: 0.2.128\n  dependency-type: direct:production\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>",
          "timestamp": "2026-09-30T19:38:53+08:00",
          "tree_id": "802883d51f9bac1d209db0d93ef81c685cb6f427",
          "url": "https://github.com/Faicad/brepkit2/commit/9dd6a50d1f50059958c9122eed581199684fbcf6"
        },
        "date": 1790768594364,
        "tool": "cargo",
        "benches": [
          {
            "name": "boolean/cut_box_box",
            "value": 528931,
            "range": "± 12081",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/fuse_box_box",
            "value": 572617,
            "range": "± 11786",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/intersect_box_box",
            "value": 6890,
            "range": "± 106",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/cut_cylinder_through_box",
            "value": 385068,
            "range": "± 9585",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/perforated_cut_36",
            "value": 14809057,
            "range": "± 809720",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "yuan_xin_yu@hotmail.com",
            "name": "Ylt",
            "username": "yuan-xy"
          },
          "committer": {
            "email": "yuan_xin_yu@hotmail.com",
            "name": "Ylt",
            "username": "yuan-xy"
          },
          "distinct": true,
          "id": "df21b152838d51034f08a89ff1b888da3d40f5f2",
          "message": "fix(ci): commit Cargo.lock and clear stale criterion cache for bench\n\n- OSV Scan failed because Cargo.lock was gitignored, so the scanner\n  could not find the lockfile on the runner. Commit it (also unblocks\n  reproducible builds and the Security Audit job).\n- Benchmark publish failed because rust-cache restored a partial\n  target/criterion baseline, making criterion emit ERROR lines into\n  the bencher output. Wipe target/criterion before running benches.",
          "timestamp": "2026-09-30T20:01:59+08:00",
          "tree_id": "91fab4875676fc7faf2741da4b0eb9bd470208f2",
          "url": "https://github.com/Faicad/brepkit2/commit/df21b152838d51034f08a89ff1b888da3d40f5f2"
        },
        "date": 1790769906340,
        "tool": "cargo",
        "benches": [
          {
            "name": "boolean/cut_box_box",
            "value": 988508,
            "range": "± 5019",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/fuse_box_box",
            "value": 1076484,
            "range": "± 2725",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/intersect_box_box",
            "value": 12981,
            "range": "± 24",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/cut_cylinder_through_box",
            "value": 708748,
            "range": "± 5389",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/perforated_cut_36",
            "value": 26464786,
            "range": "± 122350",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "yuan_xin_yu@hotmail.com",
            "name": "Ylt",
            "username": "yuan-xy"
          },
          "committer": {
            "email": "yuan_xin_yu@hotmail.com",
            "name": "Ylt",
            "username": "yuan-xy"
          },
          "distinct": true,
          "id": "f8f9b569216109fa8335ad4ecf5b1016561a40ec",
          "message": "fix(security): bump lodash-es to 4.18.1 and allowlist unmaintained ttf-parser\n\n- lodash-es 4.17.23 had two advisories (GHSA-f23m-r3pf-42rh,\n  GHSA-r5fr-rjxr-66jc), fixed in 4.18.0; pin ^4.18.1 in devDependencies\n  so chevrotain/mermaid resolve the patched copy.\n- RUSTSEC-2026-0192 (ttf-parser unmaintained) has no fixed version; the\n  crate is only reachable via winit -> sctk-adwaita -> ab_glyph behind\n  brepkit-render's window feature and never parses untrusted input, so\n  allowlist it in deny.toml until upstream moves to skrifa.",
          "timestamp": "2026-09-30T20:16:25+08:00",
          "tree_id": "82bf36585853a81dfa49af2b0a80374ff2a29ae0",
          "url": "https://github.com/Faicad/brepkit2/commit/f8f9b569216109fa8335ad4ecf5b1016561a40ec"
        },
        "date": 1790770927751,
        "tool": "cargo",
        "benches": [
          {
            "name": "boolean/cut_box_box",
            "value": 949208,
            "range": "± 20523",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/fuse_box_box",
            "value": 1033299,
            "range": "± 1963",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/intersect_box_box",
            "value": 11975,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/cut_cylinder_through_box",
            "value": 702610,
            "range": "± 10234",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/perforated_cut_36",
            "value": 24945648,
            "range": "± 44326",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "yuan_xin_yu@hotmail.com",
            "name": "Ylt",
            "username": "yuan-xy"
          },
          "committer": {
            "email": "yuan_xin_yu@hotmail.com",
            "name": "Ylt",
            "username": "yuan-xy"
          },
          "distinct": true,
          "id": "e442bd3f0fe8ba43552daf92ff8c6b8135d12f3b",
          "message": "fix(security): override lodash-es to ^4.18.1 for chevrotain chain\n\nchevrotain 11.1.2 (pinned by mermaid 12) hard-depends on\nlodash-es 4.17.23, which has GHSA-f23m-r3pf-42rh and\nGHSA-r5fr-rjxr-66jc. The devDependency bump alone did not dedupe it;\nadd an npm override so the whole tree resolves to 4.18.1.",
          "timestamp": "2026-09-30T20:27:07+08:00",
          "tree_id": "abeb424afc0095e9da41306f03709b5578c06073",
          "url": "https://github.com/Faicad/brepkit2/commit/e442bd3f0fe8ba43552daf92ff8c6b8135d12f3b"
        },
        "date": 1790771446542,
        "tool": "cargo",
        "benches": [
          {
            "name": "boolean/cut_box_box",
            "value": 920433,
            "range": "± 15561",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/fuse_box_box",
            "value": 1001672,
            "range": "± 25129",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/intersect_box_box",
            "value": 11355,
            "range": "± 250",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/cut_cylinder_through_box",
            "value": 685577,
            "range": "± 15017",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/perforated_cut_36",
            "value": 24798131,
            "range": "± 181854",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "yuan_xin_yu@hotmail.com",
            "name": "Ylt",
            "username": "yuan-xy"
          },
          "committer": {
            "email": "yuan_xin_yu@hotmail.com",
            "name": "Ylt",
            "username": "yuan-xy"
          },
          "distinct": true,
          "id": "7efc6ddee97f337563e07b8a999e595656322193",
          "message": "fix(ci): add osv-scanner.toml to allowlist unmaintained ttf-parser\n\nosv-scanner does not read deny.toml, so RUSTSEC-2026-0192 kept failing\nthe blocking scan despite the cargo-deny allowlist. Mirror the ignore\nin osv-scanner.toml at the repo root (same scope as Cargo.lock).",
          "timestamp": "2026-09-30T20:31:44+08:00",
          "tree_id": "4412bffcb59745f3e6a5d487dea2417271860df0",
          "url": "https://github.com/Faicad/brepkit2/commit/7efc6ddee97f337563e07b8a999e595656322193"
        },
        "date": 1790771752076,
        "tool": "cargo",
        "benches": [
          {
            "name": "boolean/cut_box_box",
            "value": 973613,
            "range": "± 12719",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/fuse_box_box",
            "value": 1055826,
            "range": "± 14863",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/intersect_box_box",
            "value": 11982,
            "range": "± 97",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/cut_cylinder_through_box",
            "value": 716805,
            "range": "± 4754",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/perforated_cut_36",
            "value": 24901235,
            "range": "± 50190",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "yuan_xin_yu@hotmail.com",
            "name": "Ylt",
            "username": "yuan-xy"
          },
          "committer": {
            "email": "yuan_xin_yu@hotmail.com",
            "name": "Ylt",
            "username": "yuan-xy"
          },
          "distinct": true,
          "id": "b4893347f622ec43e1e0127b77f8a69395a49994",
          "message": "docs(i18n): expand terminology glossary with per-area term tables",
          "timestamp": "2026-10-01T10:33:36+08:00",
          "tree_id": "528216e44b0c3bd0813bdf5e2523c94f6539a822",
          "url": "https://github.com/Faicad/brepkit2/commit/b4893347f622ec43e1e0127b77f8a69395a49994"
        },
        "date": 1790822172481,
        "tool": "cargo",
        "benches": [
          {
            "name": "boolean/cut_box_box",
            "value": 963512,
            "range": "± 2100",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/fuse_box_box",
            "value": 1044112,
            "range": "± 3714",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/intersect_box_box",
            "value": 11959,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/cut_cylinder_through_box",
            "value": 700764,
            "range": "± 3354",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/perforated_cut_36",
            "value": 24947819,
            "range": "± 196765",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "yuan_xin_yu@hotmail.com",
            "name": "Ylt",
            "username": "yuan-xy"
          },
          "committer": {
            "email": "yuan_xin_yu@hotmail.com",
            "name": "Ylt",
            "username": "yuan-xy"
          },
          "distinct": true,
          "id": "3d8743e124d1d595b2fc5513ca3edf9380c2e093",
          "message": "fix(blend): give fillet_v2 the correct box-fillet volume\n\nC-01, engine half. On a 10^3 cube with all 12 edges filleted the walking\nengine now reproduces Steiner's closed form at every radius:\n\n    r      closed    before      after\n    0.1   999.744   (no datum)  999.744\n    0.5   993.729    977.163    993.692\n    1.0   975.587    912.675    975.332\n    2.0   907.705    690.772    905.746\n\nThe r=2 residual of -1.959 belongs to the ruler, not the geometry:\nfillet_rolling_ball, a separate solver, measures 905.7461 under the same\ndeflection against fillet_v2's 905.7464.\n\nThree independent defects, all fixed here:\n\n1. Stripes spanned the whole edge. A stripe must stop one radius short of\n   each end at a vertex where three or more filleted edges meet, because the\n   spherical corner patch owns that material. New `setback` module solves the\n   hold-back analytically -- the ball tangent to the two adjacent faces sits\n   at w = r (n1+n2)/(1+n1.n2) and its clearance from a third face varies\n   linearly along the edge, so the setback is (r - w.n3)/(u.n3) -- and\n   `Spine::window` restricts a stripe to the surviving sub-interval. Fewer\n   than three filleted edges at a vertex gets no setback: no corner patch is\n   coming to take that material over.\n\n2. The corner patch's interior control point sat on the rolling-ball sphere.\n   A degree-(2,2) rational patch over a wide spherical triangle then sags\n   inward across its middle: sampled at (0.5,0.5) it lay 13.5% of R inside\n   the sphere, gouging material out of the corner. Moving the apex to the\n   tangent-cone apex (the un-normalised sum of the unit radial directions,\n   which overshoots by sqrt(3) for an orthogonal corner and lands exactly on\n   the box corner) brings all eight patches onto the same surface the\n   rolling-ball engine already used.\n\n3. Corner patches came out wound the wrong way: 6 of 8 had their tessellated\n   normals pointing into the material, against 0 of 8 in the control engine.\n   `VertexContactData::is_convex` selects vertex -/+ sum(normals)*r in\n   `compute_sphere_center`, and it reports false for all eight of these\n   convex corners, so reading it as \"is convex\" when picking the outward\n   direction inverted the answer. The flag now drives the radial sign\n   consistently, and the face carries the result in its `reversed` flag,\n   which both the tessellator and the volume integral follow. The test is the\n   (u,v) grid cross product rather than `surface.normal`, because a\n   tensor-product patch whose control grid is transposed disagrees with its\n   own parametric normal while the tessellator walks the grid.\n\nEach fix was verified by disabling it: without the orientation correction the\nvolume returns to exactly 912.6748 at r=1, with the on-sphere apex r=2 is\n5.78 off, and without the setback the bands span the full edge again.\n\nTwo of the box tickets in tests/fillet_box_volume.rs graduate from ignored to\nenforced, and two in regress_fillet_cascade stop being tickets. Those two\nasserted the volume stayed within 1.0 of the un-filleted 1000 -- an\nexpectation no correct engine can meet (a box fillet loses\n2.5752*(lx+ly+lz)*r^2, 24.4 at r=1) -- so they now assert the closed form.\n\nRemaining, documented but not fixed: the shell is still not a closed\n2-manifold. `corner::compute_corners` mints fresh vertices and edges for each\npatch boundary instead of sharing the stripe ends' arcs, so 76 edge-uses in\nthe r=1 result have a single face (control engine: 0). The volume is\nunaffected, but validate_shell_closed rejects it, which is what keeps the\nwasm cascade from accepting fillet_v2.\n\nblend: 99 tests pass. operations: 1033 tests pass. clippy --all-targets\nclean, check-boundaries.sh clean.",
          "timestamp": "2026-10-02T08:33:39+08:00",
          "tree_id": "9873ac1fa03790e4a1d18c8251262021ffc9fb17",
          "url": "https://github.com/Faicad/brepkit2/commit/3d8743e124d1d595b2fc5513ca3edf9380c2e093"
        },
        "date": 1790901395753,
        "tool": "cargo",
        "benches": [
          {
            "name": "boolean/cut_box_box",
            "value": 507817,
            "range": "± 5518",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/fuse_box_box",
            "value": 555500,
            "range": "± 1607",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/intersect_box_box",
            "value": 6872,
            "range": "± 384",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/cut_cylinder_through_box",
            "value": 373209,
            "range": "± 1049",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/perforated_cut_36",
            "value": 15160336,
            "range": "± 422191",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "yuan_xin_yu@hotmail.com",
            "name": "Ylt",
            "username": "yuan-xy"
          },
          "committer": {
            "email": "yuan_xin_yu@hotmail.com",
            "name": "Ylt",
            "username": "yuan-xy"
          },
          "distinct": true,
          "id": "8292ca9fbfbdb285487d6a665e202c366750af41",
          "message": "chore(release): v2.129.16\n\nAuto-computed from git history: two blend fixes (42539e2, 3d8743e) ->\npatch bump. CHANGELOG block generated under Keep a Changelog format;\npublished to npm as @faicad/brepkit2-wasm@2.129.16 (tag: next).",
          "timestamp": "2026-10-02T17:22:28+08:00",
          "tree_id": "fb8f417daa218568c2f618cbe5d504b6b56085d3",
          "url": "https://github.com/Faicad/brepkit2/commit/8292ca9fbfbdb285487d6a665e202c366750af41"
        },
        "date": 1790933131005,
        "tool": "cargo",
        "benches": [
          {
            "name": "boolean/cut_box_box",
            "value": 946018,
            "range": "± 5816",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/fuse_box_box",
            "value": 1028917,
            "range": "± 16171",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/intersect_box_box",
            "value": 11844,
            "range": "± 414",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/cut_cylinder_through_box",
            "value": 705957,
            "range": "± 13355",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/perforated_cut_36",
            "value": 24821336,
            "range": "± 138079",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "yuan_xin_yu@hotmail.com",
            "name": "Ylt",
            "username": "yuan-xy"
          },
          "committer": {
            "email": "yuan_xin_yu@hotmail.com",
            "name": "Ylt",
            "username": "yuan-xy"
          },
          "distinct": true,
          "id": "b0928173ce9fceeec2bccde9cade8fb407af518b",
          "message": "fix(shell): register cavity faces as an inner shell and classify through it (B-04)\n\n- shell() closed path put the 6 outer faces and 6 cavity faces into a\n  single disconnected shell and never registered an inner shell. Split\n  connected components via face_components: largest stays the outer\n  shell, the rest are registered as cavity (inner) shells.\n- classify_point / compute_winding_number only traversed the outer\n  shell, so a cavity center classified Inside once cavity faces moved\n  to the inner shell. Face set is now outer + inner shells; ray parity\n  gives the correct sign automatically.\n- visibility of face_components widened pub(super) -> pub(crate).\n- add regression tests: sense/connectivity check with a plain-box\n  control case, and cavity/wall point classification for both closed\n  and open-top variants. Plan doc updated with the round record.",
          "timestamp": "2026-10-02T17:48:24+08:00",
          "tree_id": "b37e47a8c25abf58508080ffb360ed6978b3a542",
          "url": "https://github.com/Faicad/brepkit2/commit/b0928173ce9fceeec2bccde9cade8fb407af518b"
        },
        "date": 1790934855447,
        "tool": "cargo",
        "benches": [
          {
            "name": "boolean/cut_box_box",
            "value": 519646,
            "range": "± 5431",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/fuse_box_box",
            "value": 578068,
            "range": "± 10271",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/intersect_box_box",
            "value": 7286,
            "range": "± 136",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/cut_cylinder_through_box",
            "value": 386389,
            "range": "± 5645",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/perforated_cut_36",
            "value": 15341257,
            "range": "± 295577",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "yuan_xin_yu@hotmail.com",
            "name": "Ylt",
            "username": "yuan-xy"
          },
          "committer": {
            "email": "yuan_xin_yu@hotmail.com",
            "name": "Ylt",
            "username": "yuan-xy"
          },
          "distinct": true,
          "id": "e3fc3f0b910560b7193082df9b781a86b71f2c89",
          "message": "feat(io,ci): fix cone STEP semi-angle and add OCCT cross-kernel compat gate\n\n- STEP writer now emits CONICAL_SURFACE semi-angle measured from the\n  axis (ISO 10303); the reader converts it back to the internal\n  radial-plane definition. Fixes OCCT misreading cone volume\n  (1651.0 instead of 612.6) found by the faijs cross-kernel probe.\n- Add scripts/compare-occt.mjs: builds 10 reference solids in both\n  brepkit2 (local wasm build) and occt-wasm, exports STEP from each,\n  and gates on volume/area/bbox/surface-census agreement plus OCCT\n  reading brepkit2 STEP back with matching volume. Sphere topology\n  difference (two hemispheres vs seam) is a documented exemption.\n- New kernel-compat CI job on main pushes and a weekly schedule;\n  occt-wasm is installed ad hoc inside the job and is not a project\n  dependency.",
          "timestamp": "2026-10-03T07:17:26+08:00",
          "tree_id": "3aa4a4db36b67d3b71adb55f273f2df1767f0ff4",
          "url": "https://github.com/Faicad/brepkit2/commit/e3fc3f0b910560b7193082df9b781a86b71f2c89"
        },
        "date": 1790983164460,
        "tool": "cargo",
        "benches": [
          {
            "name": "boolean/cut_box_box",
            "value": 527098,
            "range": "± 6852",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/fuse_box_box",
            "value": 587617,
            "range": "± 15874",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/intersect_box_box",
            "value": 7258,
            "range": "± 90",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/cut_cylinder_through_box",
            "value": 384131,
            "range": "± 5170",
            "unit": "ns/iter"
          },
          {
            "name": "boolean/perforated_cut_36",
            "value": 15251132,
            "range": "± 247178",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}