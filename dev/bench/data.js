window.BENCHMARK_DATA = {
  "lastUpdate": 1790760057728,
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
      }
    ]
  }
}