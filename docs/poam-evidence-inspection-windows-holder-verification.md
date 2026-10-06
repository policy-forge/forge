# Evidence inspection: Windows holder correction and current local measurement

This supplement records the narrow platform holder correction and a fresh local macOS LLVM measurement of the complete 13-file evidence-inspection cohort. It does not establish Windows test execution or native platform acceptance.

The selected comparison remains frozen base `0006cd28c5f1bed9ebdd80f443dacd7014f38335`. The authentic measurement ran at unchanged head `e3b32c353c95810ccf1422fd4265a9409106c8b4` with the current captured source bytes. Those are separate identities. Both job source generations match every source pin below.

## Narrow correction

`RootGeneration` retains the same owned `Vec<File>` in exactly one platform field: `directories` on Unix and `_directories` on non-Unix. The non-Unix handles remain held for their original lifetime. The patch introduces no clone, early release, permission change, fallback or new admission. Immediate-predecessor comparison retains all 50 function signatures and docblocks and 49 literal bodies; only the `qualify_root` initializer body changes. The non-Unix field has adjacent documentation, and the Unix field documentation and cfg are explicit.

The authentic Windows run **37197264431**, job **111421597552**, failed during **Lint** because the original field was never read on non-Unix. The retained log contains compilation errors before any test summaries. Exit zero on the separate log-capture command means that capture succeeded; it is not a Windows job pass. A new Windows run and platform qualification remain pending.

## Current documentation cohort

| Literal declaration dimension | Selected documented/total | Whole 13 files documented/total |
|---|---:|---:|
| Functions | 216/216 | 404/592 |
| Types | 44/44 | 93/121 |
| Members | 201/214 | 604/877 |
| Modules | 9/10 | 60/81 |

There are 214 new selected functions and 2 changed existing functions relative to the original comparison base. The latter are `crate::cli::execute` and `crate::linkage::has_normalized_path_spelling`. The visibility/doc change to the path-spelling helper retains its body. These are current counts; historical counts are preserved in their original records.

The lexical selected scopes contain 145 production-labelled, 55 cfg-test and 16 integration-test declarations. The production label is not a cfg evaluation. In particular, **13 captured-context compound `#[cfg(all(test, unix))]` occurrences** are nine registered controls and four helpers, not production functions. The reader retains **27 qualified selected cfg rows** in total, including these 13 and 14 other platform-qualified occurrences; the full cohort retains 52 such qualifications. Adjacent module declaration docs and inner module docs remain separate dimensions. The selected `workflow_evidence_cli::tests` outer declaration lacks a directly adjacent outer doc even though its inner doc is retained. All selected and whole member/module/type/function omission locators remain in the machine dataset.

## Authentic current measurement

The fresh whole-workspace, all-feature LLVM job completed with **3,262 passed, 0 failed and 3 ignored** across **70 summaries**, exit zero and unchanged source. The exact raw export is `bc0699b01d1fe8c78a2418b8df6d7f38b74134ac69a7a095f7da003e1852da2c` (22,433,197 bytes).

The separate current format check and strict all-target, all-feature Clippy job passed with unchanged source. The Clippy receipt matches all 13 current pins before and after its command. Clippy is a lint result and adds no test outcomes. A subsequent enabled hook remains pending; this supplement makes no claim about a future commit hook.

| Raw whole-export dimension | Positive/total |
|---|---:|
| Lines | 83,005/90,866 (91.3487993309%) |
| Functions | 7,404/8,801 |
| Instantiations | 10,429/15,639 |
| Regions | 145,397/160,370 |

The export contains 204 files and 19,545 raw function records, including all **6,410 zero-count records**. These raw/generated/test denominators differ from named source declarations. Branch and MC/DC denominators are zero and therefore unavailable.

Exact filename, literal header line and UTF-8 column are matched only to the **first regular CodeRegion** of each emitted record. Among the **216 selected headers**, **207 are positive, 2 are zero and 7 are unmapped**. All **408 aliases** are retained: 380 positive and 28 zero. Across all 592 named headers, 567 are positive, 8 zero and 17 unmapped; all 971 aliases remain, including 100 zero aliases.

The two selected zero headers are `CountingWriter::flush` and `BoundedWriter::flush`. The seven selected unmapped occurrences are platform versions of `directory_identity`, `open_windows`, `open_root` and `open_local`, retained separately with their exact cfg/header locations. Captured-context `plan_tasks` has exact alias counts **[0, 6]**; a positive alias does not erase a zero alias. The machine dataset also retains all 1109 scoped non-header first-region records and every raw zero index. No nearby generated closure or name-only substitute earns a header observation.

Positive header entry does not prove full body, line, branch, every alias, Windows execution or consumer correctness. The original full S4 records, intermediate failures and earlier coverage remain unchanged. No future enabled-hook result is claimed here.

## Exact current source pins

- `src/evidence_capture.rs` — `e607a7dd3450ea8ed72fd63b621eaf18c4f39add1c3421bc2abab5100a72523b` (36,951 bytes)
- `src/linkage/fresh.rs` — `205180dbb6c96fd4d3624a9c195d324c544a1a7198c44dcf837e64f210ece9d0` (73,067 bytes)
- `src/mapping/inventory.rs` — `ed3c69b14ba5125bb7dc55d95644c43af955f4b20302de073ae4f86706efe8a8` (33,712 bytes)
- `src/poam/source.rs` — `2b9513309900b916c32ec608a44cd59fd1a93c84da3ba001c8a1989a8971607b` (76,773 bytes)
- `src/poam/workflow_evidence.rs` — `60ce4bf61fc56d95d7cdc4d94d161c50abd5daa6b9d1969abbcbef8fdc55403c` (58,054 bytes)
- `src/assessment_results/context.rs` — `0aa316507e1ce203834043d38fedabf24c4927276bbc0ca84fbd98b9aeabf2a7` (46,770 bytes)
- `src/assessment_results/context_captured.rs` — `58d649e65fc0693ab5645cee20123fabbb864baee2852e6765d213c0c2ff6a02` (49,097 bytes)
- `src/cli/mod.rs` — `3b0eb41fd51284003906510d554c2b84d0bf796ae0f3071913d7f28dc3b3f24d` (116,718 bytes)
- `src/lib.rs` — `bca366cb5192c08123e86f1eb6a83eaff005d8a9e09c88e0a5f3db980eda8151` (5,090 bytes)
- `src/linkage/mod.rs` — `3fb1e0829922ed5fc94b7cf99266dafff43b1441d5e2426507ce35186cc8a9d0` (92,837 bytes)
- `src/poam/mod.rs` — `59a089fa227c78a4b04d8cb2c66721c532eb4613de5c67e83c2f4f862b440ff1` (14,061 bytes)
- `src/poam/workflow_evidence_cli.rs` — `52eb2533fa0c4e59c5446eaff351e70667d5641bcda371a2904e71d15a42080f` (8,372 bytes)
- `tests/poam_foundation_test.rs` — `d029bb7bbe7b178721a8b0f8e931a1d98e4074681a52d3fe01c79dd9edd08550` (194,211 bytes)

## Review limits and remaining gates

This is a pure source/metadata readback. Historical authored context, helper, wrapper, control, CLI and verifier correctness is excluded from independent approval. The associated local job is Root-owned; this reader ran no candidate, compiler, control or native code. Windows execution, Linux/Windows S4 qualification, D064 terminal authority, human and consumer acceptance, and the final goal-wide documentation review remain separate open gates. Root must record the actual subsequent enabled hook and hosted Windows outcome in successors.
