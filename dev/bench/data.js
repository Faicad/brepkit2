window.BENCHMARK_DATA = {
  "lastUpdate": 1790822173358,
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
      }
    ]
  }
}