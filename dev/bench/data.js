window.BENCHMARK_DATA = {
  "lastUpdate": 1790767511462,
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
      }
    ]
  }
}